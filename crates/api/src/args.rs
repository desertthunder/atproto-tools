use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};

pub const DEFAULT_DB_PATH: &str = "atp-tools-api.sqlite3";
pub const DEFAULT_PUBLIC_API_BASE: &str = "https://public.api.bsky.app";
pub const DEFAULT_PLC_DIRECTORY_BASE: &str = "https://plc.directory";
pub const DEFAULT_SEED_ACTOR: &str = "desertthunder.dev";
pub const DEFAULT_BIND: &str = "127.0.0.1:3000";
pub const DEFAULT_JETSTREAM_ENDPOINT: &str = "wss://jetstream2.us-west.bsky.network/subscribe";

#[derive(Debug, Parser)]
#[command(name = "atp-tools-api")]
#[command(about = "Serve and backfill AT Protocol enrichment data")]
#[command(long_about = "\
Serve the local AT Protocol enrichment API and run the Jetstream worker that keeps its SQLite database warm.

The API stores graph and feed-derived data in a local SQLite database. Use the server command for HTTP/XRPC access and the worker command for continuous Jetstream ingestion.")]
pub struct Cli {
    /// SQLite database path shared by the server and worker.
    #[arg(long, global = true, value_name = "PATH", default_value = DEFAULT_DB_PATH)]
    pub database: PathBuf,

    /// Base URL for the public Bluesky API used for profile, graph, and feed lookups.
    #[arg(long, global = true, default_value = DEFAULT_PUBLIC_API_BASE)]
    pub public_api_base: String,

    /// Base URL for the PLC directory used when resolving DID documents.
    #[arg(long, global = true, default_value = DEFAULT_PLC_DIRECTORY_BASE)]
    pub plc_directory_base: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Serve the Axum XRPC API and run a bounded author-feed backfill on startup.
    Server {
        /// Socket address for the HTTP server to bind.
        #[arg(long, default_value = DEFAULT_BIND)]
        bind: SocketAddr,

        /// Actor handle or DID used to seed the initial follow graph backfill.
        #[arg(long, default_value = DEFAULT_SEED_ACTOR)]
        seed_actor: String,
    },
    /// Run the Jetstream indexer worker for continuous event ingestion.
    Worker {
        /// Jetstream WebSocket endpoint to subscribe to.
        #[arg(long, default_value = DEFAULT_JETSTREAM_ENDPOINT)]
        jetstream_endpoint: String,

        /// Actor handle or DID used to seed graph data before live ingestion.
        #[arg(long, default_value = DEFAULT_SEED_ACTOR)]
        seed_actor: String,

        /// Seconds between worker heartbeat log messages.
        #[arg(long, default_value_t = 30)]
        heartbeat_seconds: u64,

        /// Resume from the latest persisted cursor. Pass --resume=false to replay from the endpoint default.
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        resume: bool,
    },
}
