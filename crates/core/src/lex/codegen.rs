use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CodegenReport {
    pub output: PathBuf,
    pub structs: Vec<String>,
}

pub fn generate_serde_models(
    input_dir: impl AsRef<Path>, output: impl AsRef<Path>,
) -> Result<CodegenReport, CodegenError> {
    let input_dir = input_dir.as_ref();
    let output = output.as_ref();
    let mut files = Vec::new();
    collect_json_files(input_dir, &mut files)?;
    files.sort();

    let mut structs = Vec::new();
    let mut generated = String::from(
        "// This file is generated from AT Protocol Lexicon JSON. Do not edit by hand.\n\nuse serde::{Deserialize, Serialize};\n\n",
    );

    for path in files {
        let contents =
            fs::read_to_string(&path).map_err(|source| CodegenError::ReadFile { path: path.clone(), source })?;
        let lexicon = serde_json::from_str::<Value>(&contents)
            .map_err(|source| CodegenError::Json { path: path.clone(), source })?;
        generate_lexicon(&lexicon, &path, &mut generated, &mut structs)?;
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|source| CodegenError::CreateDir { path: parent.to_path_buf(), source })?;
    }

    let generated = format!("{}\n", generated.trim_end());
    fs::write(output, generated).map_err(|source| CodegenError::WriteFile { path: output.to_path_buf(), source })?;

    Ok(CodegenReport { output: output.to_path_buf(), structs })
}

#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("failed to read lexicon directory at {path}: {source}")]
    ReadDir { path: PathBuf, source: std::io::Error },
    #[error("failed to read lexicon at {path}: {source}")]
    ReadFile { path: PathBuf, source: std::io::Error },
    #[error("failed to parse lexicon JSON at {path}: {source}")]
    Json { path: PathBuf, source: serde_json::Error },
    #[error("lexicon at {path} is missing {field}")]
    MissingField { path: PathBuf, field: &'static str },
    #[error("failed to create generated source directory at {path}: {source}")]
    CreateDir { path: PathBuf, source: std::io::Error },
    #[error("failed to write generated source at {path}: {source}")]
    WriteFile { path: PathBuf, source: std::io::Error },
}

fn collect_json_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), CodegenError> {
    for entry in fs::read_dir(dir).map_err(|source| CodegenError::ReadDir { path: dir.to_path_buf(), source })? {
        let path = entry
            .map_err(|source| CodegenError::ReadDir { path: dir.to_path_buf(), source })?
            .path();

        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }

    Ok(())
}

fn generate_lexicon(
    lexicon: &Value, path: &Path, output: &mut String, structs: &mut Vec<String>,
) -> Result<(), CodegenError> {
    let id = lexicon
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "id" })?;
    let defs = lexicon
        .get("defs")
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "defs" })?;
    let prefix = safe_type_name(&pascal_case(id.rsplit('.').next().unwrap_or(id)));

    for (def_name, def) in defs {
        let Some(kind) = def.get("type").and_then(Value::as_str) else {
            continue;
        };

        match kind {
            "record" => {
                let record = def
                    .get("record")
                    .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "record" })?;
                let struct_name = prefix.clone();
                emit_struct(output, structs, &prefix, &struct_name, Some(id), record)?;
            }
            "object" => {
                let struct_name =
                    if def_name == "main" { prefix.clone() } else { format!("{prefix}{}", pascal_case(def_name)) };
                emit_struct(output, structs, &prefix, &struct_name, None, def)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn emit_struct(
    output: &mut String, structs: &mut Vec<String>, lexicon_prefix: &str, struct_name: &str, record_type: Option<&str>,
    object: &Value,
) -> Result<(), CodegenError> {
    let required = object
        .get("required")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let properties = object
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    output.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]\n");
    output.push_str("#[serde(rename_all = \"camelCase\")]\n");
    output.push_str(&format!("pub struct {struct_name} {{\n"));

    if record_type.is_some() {
        let default_fn = format!("default_{}_type", snake_case(struct_name));
        output.push_str("    #[serde(rename = \"$type\", default = \"");
        output.push_str(&default_fn);
        output.push_str("\")]\n");
        output.push_str("    pub r#type: std::string::String,\n");
    }

    let mut sorted = properties.into_iter().collect::<BTreeMap<_, _>>();
    for (name, schema) in &mut sorted {
        let field_name = rust_field_name(name);
        let field_type = rust_type(schema, lexicon_prefix);
        let is_required = required.contains(name.as_str());
        if !is_required {
            output.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
        }
        if field_name != *name {
            output.push_str(&format!("    #[serde(rename = \"{name}\")]\n"));
        }
        let ty = if is_required { field_type } else { format!("Option<{field_type}>") };
        output.push_str(&format!("    pub {field_name}: {ty},\n"));
    }

    output.push_str("}\n\n");
    if let Some(record_type) = record_type {
        let default_fn = format!("default_{}_type", snake_case(struct_name));
        output.push_str(&format!(
            "fn {default_fn}() -> std::string::String {{\n    \"{record_type}\".to_string()\n}}\n\n"
        ));
    }
    structs.push(struct_name.to_string());

    Ok(())
}

fn rust_type(schema: &Value, lexicon_prefix: &str) -> String {
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "std::string::String".to_string(),
        Some("integer") => "i64".to_string(),
        Some("boolean") => "bool".to_string(),
        Some("array") => {
            let item_type = schema
                .get("items")
                .map(|items| rust_type(items, lexicon_prefix))
                .unwrap_or_else(|| "serde_json::Value".to_string());
            format!("Vec<{item_type}>")
        }
        Some("ref") => schema
            .get("ref")
            .and_then(Value::as_str)
            .and_then(|reference| local_ref_type(reference, lexicon_prefix))
            .unwrap_or_else(|| "serde_json::Value".to_string()),
        Some("object") => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}

fn local_ref_type(reference: &str, lexicon_prefix: &str) -> Option<String> {
    reference.strip_prefix('#').map(|name| {
        if name == "main" {
            safe_type_name(lexicon_prefix)
        } else {
            safe_type_name(&format!("{lexicon_prefix}{}", pascal_case(name)))
        }
    })
}

fn safe_type_name(name: &str) -> String {
    match name {
        "String" => "StringRecord".to_string(),
        _ => name.to_string(),
    }
}

fn rust_field_name(name: &str) -> String {
    match name {
        "type" => "r#type".to_string(),
        _ => snake_case(name),
    }
}

fn pascal_case(value: &str) -> String {
    words(value)
        .into_iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

fn snake_case(value: &str) -> String {
    words(value).join("_")
}

fn words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lowercase = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && previous_lowercase && !current.is_empty() {
                words.push(current);
                current = String::new();
            }
            current.push(ch.to_ascii_lowercase());
            previous_lowercase = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else if !current.is_empty() {
            words.push(current);
            current = String::new();
            previous_lowercase = false;
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}
