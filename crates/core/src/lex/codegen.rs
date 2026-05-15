use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CodegenReport {
    pub output: PathBuf,
    pub structs: Vec<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CodegenLanguage {
    Rust,
    TypeScript,
}

pub fn generate_serde_models(
    input_dir: impl AsRef<Path>, output: impl AsRef<Path>,
) -> Result<CodegenReport, CodegenError> {
    generate_models(input_dir, output, CodegenLanguage::Rust)
}

pub fn generate_typescript_models(
    input_dir: impl AsRef<Path>, output: impl AsRef<Path>,
) -> Result<CodegenReport, CodegenError> {
    generate_models(input_dir, output, CodegenLanguage::TypeScript)
}

pub fn generate_models(
    input_dir: impl AsRef<Path>, output: impl AsRef<Path>, language: CodegenLanguage,
) -> Result<CodegenReport, CodegenError> {
    let input_dir = input_dir.as_ref();
    let output = output.as_ref();
    let mut files = Vec::new();
    collect_json_files(input_dir, &mut files)?;
    files.sort();

    let mut lexicons = Vec::with_capacity(files.len());
    for path in files {
        let contents =
            std::fs::read_to_string(&path).map_err(|source| CodegenError::ReadFile { path: path.clone(), source })?;
        let lexicon = serde_json::from_str::<Value>(&contents)
            .map_err(|source| CodegenError::Json { path: path.clone(), source })?;
        lexicons.push((path, lexicon));
    }

    let ref_types = collect_ref_types(&lexicons)?;
    let mut structs = Vec::new();
    let mut generated = match language {
        CodegenLanguage::Rust => String::from(
            "// This file is generated from AT Protocol Lexicon JSON. Do not edit by hand.\n\nuse serde::{Deserialize, Serialize};\n\n",
        ),
        CodegenLanguage::TypeScript => {
            let mut source =
                String::from("// This file is generated from AT Protocol Lexicon JSON. Do not edit by hand.\n\n");
            if uses_strong_ref(&lexicons) {
                source.push_str("export type AtprotoStrongRef = {\n  uri: string;\n  cid: string;\n};\n\n");
            }
            source
        }
    };

    for (path, lexicon) in lexicons {
        match language {
            CodegenLanguage::Rust => generate_rust_lexicon(&lexicon, &path, &ref_types, &mut generated, &mut structs)?,
            CodegenLanguage::TypeScript => {
                generate_typescript_lexicon(&lexicon, &path, &ref_types, &mut generated, &mut structs)?;
            }
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| CodegenError::CreateDir { path: parent.to_path_buf(), source })?;
    }

    let generated = format!("{}\n", generated.trim_end());
    std::fs::write(output, generated)
        .map_err(|source| CodegenError::WriteFile { path: output.to_path_buf(), source })?;

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
    for entry in std::fs::read_dir(dir).map_err(|source| CodegenError::ReadDir { path: dir.to_path_buf(), source })? {
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

fn generate_rust_lexicon(
    lexicon: &Value, path: &Path, ref_types: &BTreeMap<String, String>, output: &mut String, structs: &mut Vec<String>,
) -> Result<(), CodegenError> {
    let id = lexicon
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "id" })?;
    let defs = lexicon
        .get("defs")
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "defs" })?;
    let prefix = safe_type_name(&lexicon_prefix(id));

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
                emit_struct(output, structs, ref_types, &prefix, &struct_name, Some(id), record)?;
            }
            "object" => {
                let struct_name =
                    if def_name == "main" { prefix.clone() } else { format!("{prefix}{}", pascal_case(def_name)) };
                emit_struct(output, structs, ref_types, &prefix, &struct_name, None, def)?;
            }
            "query" | "procedure" => {
                if let Some(parameters) = def.get("parameters") {
                    let struct_name = format!("{prefix}Params");
                    emit_struct(output, structs, ref_types, &prefix, &struct_name, None, parameters)?;
                }

                if let Some(schema) = def.get("output").and_then(|output| output.get("schema")) {
                    let struct_name = format!("{prefix}Output");
                    emit_struct(output, structs, ref_types, &prefix, &struct_name, None, schema)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn generate_typescript_lexicon(
    lexicon: &Value, path: &Path, ref_types: &BTreeMap<String, String>, output: &mut String, structs: &mut Vec<String>,
) -> Result<(), CodegenError> {
    let id = lexicon
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "id" })?;
    let defs = lexicon
        .get("defs")
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "defs" })?;
    let prefix = safe_type_name(&lexicon_prefix(id));

    for (def_name, def) in defs {
        let Some(kind) = def.get("type").and_then(Value::as_str) else {
            continue;
        };

        match kind {
            "record" => {
                let record = def
                    .get("record")
                    .ok_or_else(|| CodegenError::MissingField { path: path.to_path_buf(), field: "record" })?;
                emit_typescript_type(output, structs, ref_types, &prefix, &prefix, Some(id), record)?;
            }
            "object" => {
                let type_name =
                    if def_name == "main" { prefix.clone() } else { format!("{prefix}{}", pascal_case(def_name)) };
                emit_typescript_type(output, structs, ref_types, &prefix, &type_name, None, def)?;
            }
            "query" | "procedure" => {
                if let Some(parameters) = def.get("parameters") {
                    let type_name = format!("{prefix}Params");
                    emit_typescript_type(output, structs, ref_types, &prefix, &type_name, None, parameters)?;
                }

                if let Some(schema) = def.get("output").and_then(|output| output.get("schema")) {
                    let type_name = format!("{prefix}Output");
                    emit_typescript_type(output, structs, ref_types, &prefix, &type_name, None, schema)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn emit_struct(
    output: &mut String, structs: &mut Vec<String>, ref_types: &BTreeMap<String, String>, lexicon_prefix: &str,
    struct_name: &str, record_type: Option<&str>, object: &Value,
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
        let field_type = rust_type(schema, ref_types, lexicon_prefix);
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

fn rust_type(schema: &Value, ref_types: &BTreeMap<String, String>, lexicon_prefix: &str) -> String {
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "std::string::String".to_string(),
        Some("integer") => "i64".to_string(),
        Some("boolean") => "bool".to_string(),
        Some("array") => {
            let item_type = schema
                .get("items")
                .map(|items| rust_type(items, ref_types, lexicon_prefix))
                .unwrap_or_else(|| "serde_json::Value".to_string());
            format!("Vec<{item_type}>")
        }
        Some("ref") => schema
            .get("ref")
            .and_then(Value::as_str)
            .and_then(|reference| ref_type(reference, ref_types, lexicon_prefix))
            .unwrap_or_else(|| "serde_json::Value".to_string()),
        Some("object") => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}

fn emit_typescript_type(
    output: &mut String, structs: &mut Vec<String>, ref_types: &BTreeMap<String, String>, lexicon_prefix: &str,
    type_name: &str, record_type: Option<&str>, object: &Value,
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

    output.push_str(&format!("export type {type_name} = {{\n"));
    if let Some(record_type) = record_type {
        output.push_str(&format!("  '$type'?: '{}';\n", typescript_string_literal(record_type)));
    }

    let sorted = properties.into_iter().collect::<BTreeMap<_, _>>();
    for (name, schema) in &sorted {
        let optional = if required.contains(name.as_str()) { "" } else { "?" };
        let field_type = typescript_type(schema, ref_types, lexicon_prefix);
        output.push_str(&format!(
            "  {}{}: {};\n",
            typescript_property_name(name),
            optional,
            field_type
        ));
    }

    output.push_str("};\n\n");
    structs.push(type_name.to_string());

    Ok(())
}

fn typescript_type(schema: &Value, ref_types: &BTreeMap<String, String>, lexicon_prefix: &str) -> String {
    if let Some(known_values) = schema.get("knownValues").and_then(Value::as_array) {
        let values = known_values
            .iter()
            .filter_map(Value::as_str)
            .map(|value| format!("'{}'", typescript_string_literal(value)))
            .collect::<Vec<_>>();
        if !values.is_empty() {
            return values.join(" | ");
        }
    }

    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "string".to_string(),
        Some("integer") => "number".to_string(),
        Some("boolean") => "boolean".to_string(),
        Some("array") => {
            let item_type = schema
                .get("items")
                .map(|items| typescript_type(items, ref_types, lexicon_prefix))
                .unwrap_or_else(|| "unknown".to_string());
            format!("{item_type}[]")
        }
        Some("ref") => schema
            .get("ref")
            .and_then(Value::as_str)
            .and_then(|reference| typescript_ref_type(reference, ref_types, lexicon_prefix))
            .unwrap_or_else(|| "unknown".to_string()),
        Some("union") => schema
            .get("refs")
            .and_then(Value::as_array)
            .map(|refs| {
                refs.iter()
                    .filter_map(Value::as_str)
                    .filter_map(|reference| typescript_ref_type(reference, ref_types, lexicon_prefix))
                    .collect::<Vec<_>>()
            })
            .filter(|refs| !refs.is_empty())
            .map(|refs| refs.join(" | "))
            .unwrap_or_else(|| "unknown".to_string()),
        Some("object") => "Record<string, unknown>".to_string(),
        _ => "unknown".to_string(),
    }
}

fn typescript_ref_type(reference: &str, ref_types: &BTreeMap<String, String>, lexicon_prefix: &str) -> Option<String> {
    if reference == "com.atproto.repo.strongRef" || reference == "com.atproto.repo.strongRef#main" {
        return Some("AtprotoStrongRef".to_string());
    }

    ref_type(reference, ref_types, lexicon_prefix)
}

fn ref_type(reference: &str, ref_types: &BTreeMap<String, String>, lexicon_prefix: &str) -> Option<String> {
    if let Some(rust_type) = ref_types.get(reference) {
        return Some(rust_type.clone());
    }

    reference.strip_prefix('#').map(|name| {
        if name == "main" {
            safe_type_name(lexicon_prefix)
        } else {
            safe_type_name(&format!("{lexicon_prefix}{}", pascal_case(name)))
        }
    })
}

fn collect_ref_types(lexicons: &[(PathBuf, Value)]) -> Result<BTreeMap<String, String>, CodegenError> {
    let mut refs = BTreeMap::new();

    for (path, lexicon) in lexicons {
        let id = lexicon
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| CodegenError::MissingField { path: path.clone(), field: "id" })?;
        let defs = lexicon
            .get("defs")
            .and_then(Value::as_object)
            .ok_or_else(|| CodegenError::MissingField { path: path.clone(), field: "defs" })?;
        let prefix = safe_type_name(&lexicon_prefix(id));

        for (def_name, def) in defs {
            let Some(kind) = def.get("type").and_then(Value::as_str) else {
                continue;
            };
            let rust_type = match kind {
                "record" => prefix.clone(),
                "object" => {
                    if def_name == "main" {
                        prefix.clone()
                    } else {
                        format!("{prefix}{}", pascal_case(def_name))
                    }
                }
                _ => continue,
            };
            refs.insert(format!("{id}#{def_name}"), rust_type);
        }
    }

    Ok(refs)
}

fn uses_strong_ref(lexicons: &[(PathBuf, Value)]) -> bool {
    lexicons.iter().any(|(_, lexicon)| value_contains_strong_ref(lexicon))
}

fn value_contains_strong_ref(value: &Value) -> bool {
    match value {
        Value::String(value) => value == "com.atproto.repo.strongRef" || value == "com.atproto.repo.strongRef#main",
        Value::Array(values) => values.iter().any(value_contains_strong_ref),
        Value::Object(values) => values.values().any(value_contains_strong_ref),
        _ => false,
    }
}

fn safe_type_name(name: &str) -> String {
    match name {
        "String" => "StringRecord".to_string(),
        _ => name.to_string(),
    }
}

fn typescript_property_name(name: &str) -> String {
    if is_typescript_identifier(name) && !typescript_reserved_word(name) {
        name.to_string()
    } else {
        format!("'{}'", typescript_string_literal(name))
    }
}

fn is_typescript_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn typescript_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

fn typescript_string_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn lexicon_prefix(id: &str) -> String {
    let segments = id.split('.').collect::<Vec<_>>();
    let Some(last) = segments.last().copied() else {
        return pascal_case(id);
    };

    if last == "defs" && segments.len() >= 2 {
        format!("{}{}", pascal_case(segments[segments.len() - 2]), pascal_case(last))
    } else {
        pascal_case(last)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_typescript_records_unions_and_strong_refs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = dir.path().join("lexicons");
        std::fs::create_dir(&input).expect("create input");
        std::fs::write(
            input.join("card.json"),
            r##"{
              "lexicon": 1,
              "id": "network.cosmik.card",
              "defs": {
                "main": {
                  "type": "record",
                  "record": {
                    "type": "object",
                    "required": ["type", "content"],
                    "properties": {
                      "type": { "type": "string", "knownValues": ["URL", "NOTE"] },
                      "content": { "type": "union", "refs": ["#urlContent", "#noteContent"] },
                      "parentCard": { "type": "ref", "ref": "com.atproto.repo.strongRef" }
                    }
                  }
                },
                "urlContent": {
                  "type": "object",
                  "required": ["url"],
                  "properties": { "url": { "type": "string" } }
                },
                "noteContent": {
                  "type": "object",
                  "required": ["text"],
                  "properties": { "text": { "type": "string" } }
                }
              }
            }"##,
        )
        .expect("write lexicon");

        let output = dir.path().join("generated.ts");
        let report = generate_typescript_models(&input, &output).expect("generate typescript");
        let source = std::fs::read_to_string(output).expect("read generated typescript");

        assert_eq!(report.structs, ["Card", "CardNoteContent", "CardUrlContent"]);
        assert!(source.contains("export type AtprotoStrongRef = {"));
        assert!(source.contains("'$type'?: 'network.cosmik.card';"));
        assert!(source.contains("content: CardUrlContent | CardNoteContent;"));
        assert!(source.contains("parentCard?: AtprotoStrongRef;"));
        assert!(source.contains("type: 'URL' | 'NOTE';"));
    }
}
