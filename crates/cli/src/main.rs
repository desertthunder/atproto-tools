use atp_tools_core::{ActorRepoInfo, AppConfig, AtprotoClient, LexiconSyncSpec, generate_serde_models, sync_lexicons};
use atp_tools_margin::{SourceNotesDocument, export_notes, export_source_notes};
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

mod echo;

#[derive(Debug, Parser)]
#[command(name = "atp")]
#[command(about = "Tools for working with AT Protocol apps and repositories")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetch profile metadata and repository information for an actor.
    #[command(alias = "i")]
    Info {
        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Print the complete response as formatted JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read or update CLI configuration.
    #[command(alias = "conf")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Sync Lexicon JSON and generate Rust models.
    #[command(alias = "lex")]
    Lexicons {
        #[command(subcommand)]
        command: LexiconCommands,
    },
    /// Work with at.margin records.
    Margin {
        #[command(subcommand)]
        command: MarginCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommands {
    /// Set a config field.
    Set {
        /// Field to set.
        #[arg(value_parser = AppConfig::FIELD_NAMES)]
        field: String,

        /// New field value.
        value: String,
    },
}

#[derive(Debug, Subcommand)]
enum LexiconCommands {
    /// Pull selected Lexicon JSON files from a GitHub repository at a pinned commit.
    Sync {
        /// GitHub repository in owner/name form.
        #[arg(long, default_value = "margin-at/margin")]
        repo: String,

        /// Commit hash to fetch from.
        #[arg(long)]
        commit: String,

        /// Directory inside the source repository that contains the lexicons.
        #[arg(long, default_value = "lexicons/at/margin")]
        source_path: String,

        /// Local destination directory.
        #[arg(long, default_value = "lexicons/at/margin")]
        dest: PathBuf,

        /// Lexicon filename to sync. Repeat to override the Margin defaults.
        #[arg(long = "file", default_values = ["collection.json", "collectionItem.json", "like.json", "note.json"])]
        files: Vec<String>,
    },
    /// Generate serde-compatible Rust models from local Lexicon JSON.
    Generate {
        /// Tool crate to generate models for.
        #[arg(value_parser = ["margin"])]
        tool: String,

        /// Local directory containing Lexicon JSON files.
        #[arg(long)]
        input: Option<PathBuf>,

        /// Generated Rust output file.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum MarginCommands {
    /// Export notes as Obsidian/GFM-compatible Markdown documents.
    Export {
        /// Source URL to export. When omitted, exports one document per source.
        #[arg(long)]
        source: Option<String>,

        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Output directory for generated Markdown files.
        #[arg(long, default_value = ".")]
        output_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.clone();
    let mut config = AppConfig::load(config_path.clone())?;

    match cli.command {
        Commands::Info { actor, json } => {
            let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
            let client = AtprotoClient::new(config.services)?;
            let info = client.actor_repo_info(&actor).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                print_info(&info);
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::Set { field, value } => {
                config.set_field(&field, value)?;
                let path = config.save(config_path)?;
                echo::pair("config", path.display());
                echo::pair(&field, config.get_field(&field)?);
            }
        },
        Commands::Lexicons { command } => match command {
            LexiconCommands::Sync { repo, commit, source_path, dest, files } => {
                let report =
                    sync_lexicons(LexiconSyncSpec { repo, commit, source_path, dest_dir: dest, files }).await?;

                echo::pair("commit", report.commit);
                let written = report
                    .written
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>();
                echo::list("written", &written);
            }
            LexiconCommands::Generate { tool, input, output } => {
                let input = input.unwrap_or_else(|| default_lexicon_input(&tool));
                let output = output.unwrap_or_else(|| default_generated_output(&tool));
                let report = generate_serde_models(input, output)?;
                echo::pair("output", report.output.display());
                echo::list("structs", &report.structs);
            }
        },
        Commands::Margin { command } => match command {
            MarginCommands::Export { source, actor, output_dir } => {
                let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
                let client = AtprotoClient::new(config.services)?;

                let documents = if let Some(source) = source {
                    if let Some(document) = export_source_notes(&client, &actor, &source).await? {
                        vec![document]
                    } else {
                        Vec::new()
                    }
                } else {
                    export_notes(&client, &actor).await?
                };

                fs::create_dir_all(&output_dir)?;
                let written = write_margin_documents(&output_dir, &documents)?;
                echo::pair("documents", written.len());
                echo::list("written", &written);
            }
        },
    }

    Ok(())
}

fn write_margin_documents(output_dir: &PathBuf, documents: &[SourceNotesDocument]) -> anyhow::Result<Vec<String>> {
    let mut written = Vec::with_capacity(documents.len());

    for document in documents {
        let path = output_dir.join(document.filename());
        fs::write(&path, document.to_markdown()?)?;
        written.push(path.display().to_string());
    }

    Ok(written)
}

fn default_lexicon_input(tool: &str) -> PathBuf {
    match tool {
        "margin" => PathBuf::from("lexicons/at/margin"),
        _ => unreachable!("clap validates tool names"),
    }
}

fn default_generated_output(tool: &str) -> PathBuf {
    match tool {
        "margin" => PathBuf::from("crates/margin/src/generated.rs"),
        _ => unreachable!("clap validates tool names"),
    }
}

fn print_info(info: &ActorRepoInfo) {
    echo::pair("did", &info.profile.did);
    echo::pair("handle", &info.profile.handle);

    echo::opt("displayName", info.profile.display_name.as_deref());
    echo::opt("description", info.profile.description.as_deref());
    echo::opt("pronouns", info.profile.pronouns.as_deref());
    echo::opt("website", info.profile.website.as_deref());
    echo::opt("avatar", info.profile.avatar.as_deref());
    echo::opt("banner", info.profile.banner.as_deref());
    echo::opt("followersCount", info.profile.followers_count);
    echo::opt("followsCount", info.profile.follows_count);
    echo::opt("postsCount", info.profile.posts_count);
    echo::opt("indexedAt", info.profile.indexed_at.as_deref());
    echo::opt("createdAt", info.profile.created_at.as_deref());

    echo::pair("repoHandle", &info.repo.handle);
    echo::pair("repoDid", &info.repo.did);
    echo::pair("handleIsCorrect", info.repo.handle_is_correct);
    echo::pair("collectionsCount", info.repo.collections.len());
    echo::list("collections", &info.repo.collections);
}
