use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Health {
    status: &'static str,
}

pub(crate) async fn check() -> Json<Health> {
    Json(Health { status: "ok" })
}
