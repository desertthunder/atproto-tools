use atp_tools_core::{AtprotoClient, ServiceConfig};
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod args;
mod db;
mod server;
mod worker;

use args::{Cli, Commands};

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
