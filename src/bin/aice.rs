use aice::{config::Config, infra::llm::build_model, router};
use std::{error::Error, io, sync::Arc};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;
    let model = config
        .llm
        .as_ref()
        .map(build_model)
        .transpose()
        .map_err(io::Error::other)?;
    let address = config.bind_addr;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| format!("cannot listen on {address}: {error}"))?;

    eprintln!("Aice listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router::build(model.map(Arc::from)))
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("cannot listen for Ctrl+C: {error}");
            }
        })
        .await?;

    Ok(())
}
