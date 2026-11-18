//! 数据库启用策略：只有配置 PGHOST 时才建立连接池。

use crate::dal::postgres;
use sqlx::PgPool;
use std::{env, io};

/// 未设置 PGHOST 时返回 None；设置后连接失败则返回启动错误。
pub async fn connect() -> io::Result<Option<PgPool>> {
    if env::var_os("PGHOST").is_none() {
        return Ok(None);
    }

    let pool = postgres::connect()
        .await
        .map_err(|error| io::Error::other(format!("cannot connect to PostgreSQL: {error}")))?;
    eprintln!("Connected to PostgreSQL");
    Ok(Some(pool))
}
