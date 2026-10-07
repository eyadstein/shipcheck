use std::process::ExitCode;

use shipcheck_api::config::Config;
use shipcheck_api::{build_router, db, with_cors, AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;
    let router = build_router(AppState::new(pool, &config.api_key));
    let app = with_cors(router, &config.cors_origins);
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(
        addr = %config.bind_addr,
        cors_origins = config.cors_origins.len(),
        "listening"
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
