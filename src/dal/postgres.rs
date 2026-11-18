//! PostgreSQL 连接池配置，连接参数由 SQLx 从 PG 系列环境变量读取。

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::time::Duration;

/// 建立最多五个连接的共享连接池，获取连接的等待时间上限为五秒。
pub(crate) async fn connect() -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(PgConnectOptions::new())
        .await
}
