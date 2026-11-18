use crate::service::character_card::{
    CardChanges, CardField, CardTemplate, CharacterCard, CharacterCardDocument, TEMPLATES,
};
use axum::{
    Json,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

pub(crate) async fn templates() -> Json<&'static [CardTemplate]> {
    Json(TEMPLATES)
}

pub(crate) async fn default_card() -> Result<Json<CharacterCard>, (StatusCode, String)> {
    read(&TEMPLATES[0]).await
}

pub(crate) async fn template_card(
    Path(id): Path<String>,
) -> Result<Json<CharacterCard>, (StatusCode, String)> {
    let template = TEMPLATES
        .iter()
        .find(|template| template.id == id)
        .ok_or((StatusCode::NOT_FOUND, "未找到该人物卡模版。".into()))?;
    read(template).await
}

pub(crate) async fn fields(
    Path(id): Path<String>,
) -> Result<Json<Vec<CardField>>, (StatusCode, String)> {
    let template = TEMPLATES
        .iter()
        .find(|template| template.id == id)
        .ok_or((StatusCode::NOT_FOUND, "未找到该人物卡模版。".into()))?;
    tokio::task::spawn_blocking(move || template.create().map(|card| card.fields().to_vec()))
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))
}

/// Create an independent card, apply a field patch, and return its Excel file.
pub(crate) async fn export(
    Path(id): Path<String>,
    Json(changes): Json<CardChanges>,
) -> Result<Response, (StatusCode, String)> {
    let template = TEMPLATES
        .iter()
        .find(|template| template.id == id)
        .ok_or((StatusCode::NOT_FOUND, "未找到该人物卡模版。".into()))?;
    let bytes = tokio::task::spawn_blocking(move || {
        let mut card = template
            .create()
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?;
        card.update(&changes)
            .map_err(|error| (StatusCode::BAD_REQUEST, error))?;
        card.export()
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))
    })
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))??;
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=character-card.xlsx",
            ),
        ],
        bytes,
    )
        .into_response())
}

async fn read(
    template: &'static CardTemplate,
) -> Result<Json<CharacterCard>, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || template.read())
        .await
        .map_err(|error| {
            eprintln!("character card reader failed: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "人物卡读取失败，请稍后再试。".into(),
            )
        })?
        .map(Json)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))
}
