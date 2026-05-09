use atp_tools_core::{ActorRepoInfo, AppConfig, AtprotoClient};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    Info {
        /// Handle or DID to inspect. Defaults to identity.identifier in config.toml.
        #[arg(long, value_name = "HANDLE_OR_DID")]
        actor: Option<String>,

        /// Print the complete response as formatted JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read or update CLI configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
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
    }

    Ok(())
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
