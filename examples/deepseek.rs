use aice::{config::Config, infra::llm};
use rig_core::completion::CompletionRequest;
use std::{env, error::Error, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if prompt.trim().is_empty() {
        return Err("usage: cargo run --example deepseek -- <prompt>".into());
    }

    let config = Config::load()?;
    let llm_config = config.llm.ok_or("configure llm in config.yaml first")?;
    let model = llm::build_model(&llm_config).map_err(|error| error as Box<dyn Error>)?;
    let response = tokio::time::timeout(
        Duration::from_secs(120),
        model.complete(CompletionRequest::new(prompt)),
    )
    .await?
    .map_err(|error| error as Box<dyn Error>)?;
    let reply = response.text();
    if reply.trim().is_empty() {
        return Err("DeepSeek returned no text".into());
    }
    println!("{reply}");
    Ok(())
}
