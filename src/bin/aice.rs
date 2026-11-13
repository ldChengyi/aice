use aice::{config::Config, router};
use std::error::Error;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = Config::load()?.bind_addr;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| format!("cannot listen on {address}: {error}"))?;

    eprintln!("Aice listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router::build())
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("cannot listen for Ctrl+C: {error}");
            }
        })
        .await?;

    Ok(())
}
