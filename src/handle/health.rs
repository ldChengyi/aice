//! 进程存活检查，不探测数据库或模型供应商的可用性。

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Health {
    status: &'static str,
}

pub(crate) async fn check() -> Json<Health> {
    Json(Health { status: "ok" })
}
