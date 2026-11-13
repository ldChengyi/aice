use crate::dal::postgres;
use sqlx::PgPool;
use std::{env, io};

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
