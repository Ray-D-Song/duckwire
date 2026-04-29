mod backend;
mod errors;
mod protocol;
mod rewrite;
mod types;

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use backend::connection::DuckDBConnection;
use protocol::simple_query::DuckWireHandlerFactory;

#[derive(Parser)]
#[command(name = "duckwire", about = "DuckDB PostgreSQL wire protocol proxy")]
struct Args {
    #[arg(short, long, help = "DuckDB database file path (default: in-memory)")]
    db: Option<String>,

    #[arg(short, long, default_value = "5433", help = "Listen port")]
    port: u16,

    #[arg(short = 'H', long, default_value = "0.0.0.0", help = "Listen address")]
    host: String,

    #[arg(short, long, help = "Log file path (directory creates duckwire.log inside)")]
    logfile: Option<String>,
}

fn init_logging(logfile: Option<&str>) {
    let env_filter = EnvFilter::from_default_env()
        .add_directive("duckwire=debug".parse().unwrap());

    if let Some(log) = logfile {
        let path = PathBuf::from(log);
        let log_path = if path.is_dir() {
            path.join("duckwire.log")
        } else {
            path
        };
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let file = std::fs::File::create(&log_path).expect("Failed to create log file");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file);

        let stdout_layer = tracing_subscriber::fmt::layer();
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();

        info!("Logging to file: {}", log_path.display());
        std::mem::forget(_guard);
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    init_logging(args.logfile.as_deref());

    let connection = Arc::new(
        DuckDBConnection::open(args.db.as_deref()).expect("Failed to open DuckDB"),
    );
    let factory = Arc::new(DuckWireHandlerFactory::new(connection));

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    info!("DuckWire listening on {addr}");

    loop {
        let (socket, _) = listener.accept().await.expect("Failed to accept");
        let factory_ref = factory.clone();
        tokio::spawn(async move {
            let _ = pgwire::tokio::process_socket(socket, None, factory_ref).await;
        });
    }
}