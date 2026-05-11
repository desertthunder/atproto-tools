use atp_tools_core::{AtprotoClient, ServiceConfig};
use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod server;
mod worker;

const DEFAULT_DB_PATH: &str = "atp-tools-api.sqlite3";
const DEFAULT_PUBLIC_API_BASE: &str = "https://public.api.bsky.app";
const DEFAULT_PLC_DIRECTORY_BASE: &str = "https://plc.directory";
const DEFAULT_SEED_ACTOR: &str = "desertthunder.dev";

#[derive(Debug, Parser)]
#[command(name = "atp-tools-api")]
#[command(about = "Indexer API for AT Protocol enrichment data")]
struct Cli {
    #[arg(long, global = true, value_name = "PATH", default_value = DEFAULT_DB_PATH)]
    database: PathBuf,

    #[arg(long, global = true, default_value = DEFAULT_PUBLIC_API_BASE)]
    public_api_base: String,

    #[arg(long, global = true, default_value = DEFAULT_PLC_DIRECTORY_BASE)]
    plc_directory_base: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Serve the Axum XRPC API and bounded author-feed backfill worker.
    Server {
        #[arg(long, default_value = server::DEFAULT_BIND)]
        bind: SocketAddr,

        #[arg(long, default_value = DEFAULT_SEED_ACTOR)]
        seed_actor: String,
    },
    /// Run the Jetstream indexer worker.
    Worker {
        #[arg(long, default_value = worker::DEFAULT_JETSTREAM_ENDPOINT)]
        jetstream_endpoint: String,

        #[arg(long, default_value = DEFAULT_SEED_ACTOR)]
        seed_actor: String,

        #[arg(long, default_value_t = 30)]
        heartbeat_seconds: u64,

        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        resume: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();
    let db = Arc::new(db::open_database(args.database).await?);
    let conf = ServiceConfig { public_api_base: args.public_api_base, plc_directory_base: args.plc_directory_base };
    let client = AtprotoClient::new(conf)?;

    match args.command {
        Commands::Server { bind, seed_actor } => server::run_server(bind, db, client, seed_actor).await,
        Commands::Worker { jetstream_endpoint, seed_actor, heartbeat_seconds, resume } => {
            worker::run_worker(db, client, jetstream_endpoint, seed_actor, heartbeat_seconds, resume).await
        }
    }
}
