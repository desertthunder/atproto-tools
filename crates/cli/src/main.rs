use atp_tools_bsky::{
    FollowLastPost, FollowsOptions, FollowsProgress, FollowsSort, FollowsSortDirection, FollowsSortField,
    fetch_follows_report_with_progress,
};
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FollowsSortFieldArg {
    Handle,
    Did,
    #[value(alias = "profileUrl")]
    ProfileUrl,
    #[value(alias = "lastPostAt")]
    LastPostAt,
    #[value(alias = "lastPostRkey")]
    LastPostRkey,
    #[value(alias = "lastPostUrl")]
    LastPostUrl,
}

impl From<FollowsSortFieldArg> for FollowsSortField {
    fn from(field: FollowsSortFieldArg) -> Self {
        match field {
            FollowsSortFieldArg::Handle => Self::Handle,
            FollowsSortFieldArg::Did => Self::Did,
            FollowsSortFieldArg::ProfileUrl => Self::ProfileUrl,
            FollowsSortFieldArg::LastPostAt => Self::LastPostAt,
            FollowsSortFieldArg::LastPostRkey => Self::LastPostRkey,
            FollowsSortFieldArg::LastPostUrl => Self::LastPostUrl,
        }
    }
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
    /// Fetch follows and their latest posts.
    Follows {
        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Only inspect the first N follows.
        #[arg(long, value_name = "N")]
        limit: Option<usize>,

        /// Sort rows by field. Defaults to last-post-at.
        #[arg(long, value_enum, value_name = "FIELD", conflicts_with_all = ["sort_ascending", "sort_descending"])]
        sort: Option<FollowsSortFieldArg>,

        /// Sort in ascending order. This is the default direction.
        #[arg(long, conflicts_with_all = ["desc", "sort_ascending", "sort_descending"])]
        asc: bool,

        /// Sort in descending order.
        #[arg(long, conflicts_with_all = ["asc", "sort_ascending", "sort_descending"])]
        desc: bool,

        /// Sort rows by field in ascending order.
        #[arg(long = "sa", value_enum, value_name = "FIELD", conflicts_with_all = ["sort", "asc", "desc", "sort_descending"])]
        sort_ascending: Option<FollowsSortFieldArg>,

        /// Sort rows by field in descending order.
        #[arg(long = "sd", value_enum, value_name = "FIELD", conflicts_with_all = ["sort", "asc", "desc", "sort_ascending"])]
        sort_descending: Option<FollowsSortFieldArg>,

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
            BskyCommands::Follows { actor, limit, sort, asc, desc, sort_ascending, sort_descending, refresh, json } => {
                let actor = actor.unwrap_or_else(|| config.identity.identifier.clone());
                let client = AtprotoClient::new(config.services)?;
                let options = follows_options(limit, sort, asc, desc, sort_ascending, sort_descending);
                let report =
                    fetch_follows_report_with_progress(&client, &actor, refresh, options, print_follows_progress)
                        .await?;
                echo::clear_status();

                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    echo::pair("actor", &report.actor);
                    echo::pair("did", &report.actor_did);
                    echo::pair("follows", report.follows.len());
                    echo::pair("followsHash", &report.follows_hash);
                    echo::pair("cache", report.cache_path.display());
                    print_follows(&report.follows);
                }
            }
        },
    }

    Ok(())
}

fn follows_options(
    limit: Option<usize>, sort: Option<FollowsSortFieldArg>, asc: bool, desc: bool,
    sort_ascending: Option<FollowsSortFieldArg>, sort_descending: Option<FollowsSortFieldArg>,
) -> FollowsOptions {
    let (field, direction) = if let Some(field) = sort_ascending {
        (field, FollowsSortDirection::Asc)
    } else if let Some(field) = sort_descending {
        (field, FollowsSortDirection::Desc)
    } else {
        (
            sort.unwrap_or(FollowsSortFieldArg::LastPostAt),
            if desc && !asc { FollowsSortDirection::Desc } else { FollowsSortDirection::Asc },
        )
    };

    FollowsOptions { limit, sort: FollowsSort { field: field.into(), direction }, ..FollowsOptions::default() }
}

fn print_follows(follows: &[FollowLastPost]) {
    let rows = follows
        .iter()
        .map(|follow| {
            [
                follow.handle.as_str(),
                follow.did.as_str(),
                follow.profile_url.as_str(),
                follow.last_post_at.as_deref().unwrap_or(""),
                follow.last_post_url.as_deref().unwrap_or(""),
            ]
        })
        .collect::<Vec<_>>();
    let widths = column_widths(&["handle", "did", "profile", "lastPostAt"], &rows);

    println!(
        "{:<handle_width$}  {:<did_width$}  {:<profile_width$}  {:<last_post_at_width$}  lastPost",
        "handle",
        "did",
        "profile",
        "lastPostAt",
        handle_width = widths[0],
        did_width = widths[1],
        profile_width = widths[2],
        last_post_at_width = widths[3],
    );

    for row in rows {
        println!(
            "{:<handle_width$}  {:<did_width$}  {:<profile_width$}  {:<last_post_at_width$}  {}",
            row[0],
            row[1],
            row[2],
            row[3],
            row[4],
            handle_width = widths[0],
            did_width = widths[1],
            profile_width = widths[2],
            last_post_at_width = widths[3],
        );
    }
}

fn column_widths(headers: &[&str; 4], rows: &[[&str; 5]]) -> [usize; 4] {
    let mut widths = headers.map(str::len);

    for row in rows {
        for index in 0..widths.len() {
            widths[index] = widths[index].max(row[index].len());
        }
    }

    widths
}

fn print_follows_progress(progress: FollowsProgress) {
    match progress {
        FollowsProgress::ResolvingActor => echo::status("resolving actor"),
        FollowsProgress::CheckingCache { path } => echo::status(format!("checking cache {}", path.display())),
        FollowsProgress::CacheHit { path, count } => {
            echo::status(format!("cache hit: {count} follows from {}", path.display()));
        }
        FollowsProgress::CacheStale { path, generated_at_unix } => {
            echo::status(format!(
                "cache stale: {} generated at {generated_at_unix}",
                path.display()
            ));
        }
        FollowsProgress::ApplyingLimit { limit } => echo::status(format!("limiting to {limit} follows")),
        FollowsProgress::FetchingFollowsPage { page } => {
            echo::status(format!("fetching follows page {page}"));
        }
        FollowsProgress::FetchedFollowsPage { page: _, total } => {
            echo::status(format!("fetched {total} follows"));
        }
        FollowsProgress::FetchingLastPosts { completed, total } => {
            if completed == 1 || completed == total || completed % 25 == 0 {
                echo::progress("latest posts", completed, total);
            }
        }
        FollowsProgress::WritingCache { path } => echo::status(format!("writing cache {}", path.display())),
        FollowsProgress::WroteCache { path } => echo::status(format!("wrote cache {}", path.display())),
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
                "graph/getFollows.json".to_string(),
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
