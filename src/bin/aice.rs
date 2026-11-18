//! HTTP 服务入口：加载配置、初始化共享状态，并响应 Ctrl+C 优雅退出。

use aice::{config::Config, init::AppState, router};
use std::error::Error;
use tokio::net::TcpListener;

/// 完成初始化后开始监听；启动失败直接返回错误，避免带故障继续提供服务。
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;
    let state = AppState::new(&config).await?;
    let address = config.bind_addr;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| format!("cannot listen on {address}: {error}"))?;

    eprintln!("Aice listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router::build(state))
        // Ctrl+C 后停止接收新请求，并等待现有请求结束。
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("cannot listen for Ctrl+C: {error}");
            }
        })
        .await?;

    Ok(())
}
