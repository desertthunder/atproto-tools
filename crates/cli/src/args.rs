use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub const CONFIG_FIELD_NAMES: [&str; 7] = [
    "identity.identifier",
    "link-digest.follow-poll-cron",
    "link-digest.limit",
    "link-digest.min-score",
    "link-digest.min-shares",
    "services.public-api-base",
    "services.plc-directory-base",
];

#[derive(Debug, Parser)]
#[command(name = "atp")]
#[command(about = "Inspect and maintain AT Protocol data from a POSIX-friendly CLI")]
#[command(long_about = "\
Inspect AT Protocol actors, synchronize Lexicon schemas, export at.margin notes, and build Bluesky follow reports.

The CLI reads configuration from ~/.config/atproto-tools/config.toml by default. Pass --config to use another file for a single invocation.")]
pub struct Cli {
    /// Read configuration from PATH instead of ~/.config/atproto-tools/config.toml.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Fetch profile metadata, DID document data, and repository information for an actor.
    #[command(alias = "i")]
    Info {
        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Print the complete response as formatted JSON instead of a compact terminal summary.
        #[arg(long)]
        json: bool,
    },
    /// Read or update CLI configuration values.
    #[command(alias = "conf")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Sync Lexicon JSON and generate serde-compatible Rust models.
    #[command(alias = "lex")]
    Lexicons {
        #[command(subcommand)]
        command: LexiconCommands,
    },
    /// Export and transform at.margin records.
    Margin {
        #[command(subcommand)]
        command: MarginCommands,
    },
    /// Fetch, cache, and analyze Bluesky app data.
    Bsky {
        #[command(subcommand)]
        command: BskyCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Set a supported config field and persist it to disk.
    Set {
        /// Config field to update.
        ///
        /// Supported fields are identity.identifier, link-digest.follow-poll-cron,
        /// link-digest.limit, link-digest.min-score, link-digest.min-shares,
        /// services.public-api-base, and services.plc-directory-base.
        #[arg(value_parser = CONFIG_FIELD_NAMES)]
        field: String,

        /// New value for FIELD. Numeric fields are validated before the config file is written.
        value: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Tool {
    Bsky,
    Margin,
    #[value(alias = "tngl")]
    Tangled,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FollowsSortFieldArg {
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

#[derive(Debug, Subcommand)]
pub enum LexiconCommands {
    /// Pull selected Lexicon JSON files from a git repository at a pinned commit.
    Sync {
        /// Tool preset to sync. The preset selects default repo, source path, destination, and files.
        tool: Tool,

        /// Git repository URL, host/path, or GitHub owner/name. Overrides the tool preset source repo.
        #[arg(long)]
        repo: Option<String>,

        /// Commit hash to fetch from. Pinning the commit keeps generated models reproducible.
        #[arg(long)]
        commit: String,

        /// Directory inside the source repository that contains Lexicon JSON files.
        #[arg(long)]
        source_path: Option<String>,

        /// Local destination directory for synced Lexicon JSON files.
        #[arg(long)]
        dest: Option<PathBuf>,

        /// Lexicon filename to sync. Repeat to replace the preset file list.
        #[arg(long = "file")]
        files: Vec<String>,
    },
    /// Generate serde-compatible Rust models from local Lexicon JSON.
    Generate {
        /// Tool crate to generate models for.
        tool: Tool,

        /// Local directory containing Lexicon JSON files. Defaults to the selected tool's lexicon directory.
        #[arg(long)]
        input: Option<PathBuf>,

        /// Generated Rust output file. Defaults to the selected tool crate's generated.rs.
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum MarginCommands {
    /// Export notes as Obsidian/GFM-compatible Markdown documents with TOML frontmatter.
    Export {
        /// Source URL to export. When omitted, exports one document per source found for the actor.
        #[arg(long)]
        source: Option<String>,

        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Output directory for generated Markdown files. The directory is created when missing.
        #[arg(long, default_value = ".")]
        output_dir: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum BskyCommands {
    /// Generate a Markdown digest of external links shared by followed accounts.
    LinkDigest {
        /// Handle or DID whose follows should be inspected. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Only include posts at or after this ISO datetime.
        #[arg(long, value_name = "ISO_DATETIME")]
        since: Option<String>,

        /// Only include posts before this ISO datetime.
        #[arg(long, value_name = "ISO_DATETIME")]
        until: Option<String>,

        /// Maximum links to include. Defaults to link-digest.limit.
        #[arg(long, value_name = "N")]
        limit: Option<usize>,

        /// Minimum bookmark + repost + like score required for a link. Defaults to link-digest.min-score.
        #[arg(long, value_name = "N")]
        min_score: Option<i64>,

        /// Minimum distinct followed accounts that must share the same link. Defaults to link-digest.min-shares.
        #[arg(long, value_name = "N")]
        min_shares: Option<usize>,

        /// Number of author-feed posts to fetch per page.
        #[arg(long, default_value_t = 100, value_name = "N")]
        feed_limit: u16,

        /// Maximum author-feed pages to fetch per follow.
        #[arg(long, default_value_t = 5, value_name = "N")]
        max_pages: usize,

        /// Ignore the cached follow list and fetch fresh follows before building the digest.
        #[arg(long)]
        refresh_follows: bool,
    },
    /// Fetch followed accounts and each account's latest original post.
    Follows {
        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Only inspect the first N follows after fetching or loading the follow list.
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

        /// Sort rows by FIELD in ascending order. Shorthand for --sort FIELD --asc.
        #[arg(long = "sa", value_enum, value_name = "FIELD", conflicts_with_all = ["sort", "asc", "desc", "sort_descending"])]
        sort_ascending: Option<FollowsSortFieldArg>,

        /// Sort rows by FIELD in descending order. Shorthand for --sort FIELD --desc.
        #[arg(long = "sd", value_enum, value_name = "FIELD", conflicts_with_all = ["sort", "asc", "desc", "sort_ascending"])]
        sort_descending: Option<FollowsSortFieldArg>,

        /// Ignore any matching cache file and fetch fresh data from Bluesky.
        #[arg(long)]
        refresh: bool,

        /// Print the complete cached report as formatted JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
}
