use atp_tools_bsky::{FollowerLastPost, fetch_followers_report};
use atp_tools_core::{ActorRepoInfo, AppConfig, AtprotoClient, LexiconSyncSpec, generate_serde_models, sync_lexicons};
use atp_tools_margin::{SourceNotesDocument, export_notes, export_source_notes};
use clap::{Parser, Subcommand, ValueEnum};
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
    /// Work with Bluesky app data.
    Bsky {
        #[command(subcommand)]
        command: BskyCommands,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Tool {
    Bsky,
    Margin,
    #[value(alias = "tngl")]
    Tangled,
}

#[derive(Debug, Subcommand)]
enum LexiconCommands {
    /// Pull selected Lexicon JSON files from a git repository at a pinned commit.
    Sync {
        /// Tool preset to sync.
        tool: Tool,

        /// Git repository URL, host/path, or GitHub owner/name.
        #[arg(long)]
        repo: Option<String>,

        /// Commit hash to fetch from.
        #[arg(long)]
        commit: String,

        /// Directory inside the source repository that contains the lexicons.
        #[arg(long)]
        source_path: Option<String>,

        /// Local destination directory.
        #[arg(long)]
        dest: Option<PathBuf>,

        /// Lexicon filename to sync. Repeat to override the tool defaults.
        #[arg(long = "file")]
        files: Vec<String>,
    },
    /// Generate serde-compatible Rust models from local Lexicon JSON.
    Generate {
        /// Tool crate to generate models for.
        tool: Tool,

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

#[derive(Debug, Subcommand)]
enum BskyCommands {
    /// Fetch followers and their latest posts.
    #[command(alias = "follows")]
    Followers {
        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Ignore any matching cache file and fetch fresh data.
        #[arg(long)]
        refresh: bool,

        /// Print the complete cached report as formatted JSON.
        #[arg(long)]
        json: bool,
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
            LexiconCommands::Sync { tool, repo, commit, source_path, dest, files } => {
                let mut spec = default_lexicon_sync_spec(tool, commit);
                if let Some(repo) = repo {
                    spec.repo = repo;
                }
                if let Some(source_path) = source_path {
                    spec.source_path = source_path;
                }
                if let Some(dest) = dest {
                    spec.dest_dir = dest;
                }
                if !files.is_empty() {
                    spec.files = files;
                }
                let report = sync_lexicons(spec).await?;

                echo::pair("commit", report.commit);
                let written = report
                    .written
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>();
                echo::list("written", &written);
            }
            LexiconCommands::Generate { tool, input, output } => {
                let input = input.unwrap_or_else(|| default_lexicon_input(tool));
                let output = output.unwrap_or_else(|| default_generated_output(tool));
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
        Commands::Bsky { command } => match command {
            BskyCommands::Followers { actor, refresh, json } => {
                let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
                let client = AtprotoClient::new(config.services)?;
                let report = fetch_followers_report(&client, &actor, refresh).await?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    echo::pair("actor", &report.actor);
                    echo::pair("did", &report.actor_did);
                    echo::pair("followers", report.followers.len());
                    echo::pair("cache", report.cache_path.display());
                    print_followers(&report.followers);
                }
            }
        },
    }

    Ok(())
}

fn print_followers(followers: &[FollowerLastPost]) {
    println!("handle\tdid\tprofile\tlastPostAt\tlastPost");
    for follower in followers {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            follower.handle,
            follower.did,
            follower.profile_url,
            follower.last_post_at.as_deref().unwrap_or(""),
            follower.last_post_url.as_deref().unwrap_or("")
        );
    }
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

fn default_lexicon_input(tool: Tool) -> PathBuf {
    match tool {
        Tool::Bsky => PathBuf::from("lexicons/app/bsky"),
        Tool::Margin => PathBuf::from("lexicons/at/margin"),
        Tool::Tangled => PathBuf::from("lexicons/sh/tangled"),
    }
}

fn default_generated_output(tool: Tool) -> PathBuf {
    match tool {
        Tool::Bsky => PathBuf::from("crates/bsky/src/generated.rs"),
        Tool::Margin => PathBuf::from("crates/margin/src/generated.rs"),
        Tool::Tangled => PathBuf::from("crates/tngl/src/generated.rs"),
    }
}

fn default_lexicon_sync_spec(tool: Tool, commit: String) -> LexiconSyncSpec {
    match tool {
        Tool::Bsky => LexiconSyncSpec {
            repo: "bluesky-social/atproto".to_string(),
            commit,
            source_path: "lexicons/app/bsky".to_string(),
            dest_dir: PathBuf::from("lexicons/app/bsky"),
            files: vec![
                "actor/defs.json".to_string(),
                "feed/defs.json".to_string(),
                "feed/getAuthorFeed.json".to_string(),
                "feed/post.json".to_string(),
                "graph/getFollowers.json".to_string(),
            ],
            preserve_paths: true,
        },
        Tool::Margin => LexiconSyncSpec {
            repo: "margin-at/margin".to_string(),
            commit,
            source_path: "lexicons/at/margin".to_string(),
            dest_dir: PathBuf::from("lexicons/at/margin"),
            files: vec![
                "collection.json".to_string(),
                "collectionItem.json".to_string(),
                "like.json".to_string(),
                "note.json".to_string(),
            ],
            preserve_paths: false,
        },
        Tool::Tangled => LexiconSyncSpec {
            repo: "tangled.org/tangled.org/core".to_string(),
            commit,
            source_path: "lexicons".to_string(),
            dest_dir: PathBuf::from("lexicons/sh/tangled"),
            files: vec![
                "string/string.json".to_string(),
                "repo/repo.json".to_string(),
                "issue/issue.json".to_string(),
            ],
            preserve_paths: false,
        },
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
