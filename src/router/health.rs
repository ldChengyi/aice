use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct Health {
    status: &'static str,
}

pub(super) async fn check() -> Json<Health> {
    Json(Health { status: "ok" })
}
