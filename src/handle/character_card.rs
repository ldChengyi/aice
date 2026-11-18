//! 人物卡 HTTP 接口：列出模板、读取字段、直接导出及下载 MCP 暂存文件。

use crate::service::character_card::{
    CardChanges, CardField, CardTemplate, CharacterCard, CharacterCardDocument, TEMPLATES,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

/// 返回内置模板的 ID 和名称，文件字节不参与 JSON 序列化。
pub(crate) async fn templates() -> Json<&'static [CardTemplate]> {
    Json(TEMPLATES)
}

/// 按导出 ID 下载文件；不存在或过期返回 404，并禁止响应缓存。
pub(crate) async fn download(
    State(state): State<crate::init::AppState>,
    Path(id): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let bytes = state
        .card_exports
        .download(&id)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "人物卡不存在或下载链接已过期。".into(),
        ))?;
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
            (header::CACHE_CONTROL, "no-store"),
        ],
        bytes,
    )
        .into_response())
}

pub(crate) async fn default_card() -> Result<Json<CharacterCard>, (StatusCode, String)> {
    read(&TEMPLATES[0]).await
}

/// 按模板 ID 读取已保存的单元格值，不执行 Excel 公式。
pub(crate) async fn template_card(
    Path(id): Path<String>,
) -> Result<Json<CharacterCard>, (StatusCode, String)> {
    let template = TEMPLATES
        .iter()
        .find(|template| template.id == id)
        .ok_or((StatusCode::NOT_FOUND, "未找到该人物卡模版。".into()))?;
    read(template).await
}

/// 返回模板允许修改的字段，供调用方生成合法的修改集合。
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

/// 从模板新建独立人物卡，应用字段修改并直接返回 Excel；不加入 MCP 暂存。
pub(crate) async fn export(
    Path(id): Path<String>,
    Json(changes): Json<CardChanges>,
) -> Result<Response, (StatusCode, String)> {
    let template = TEMPLATES
        .iter()
        .find(|template| template.id == id)
        .ok_or((StatusCode::NOT_FOUND, "未找到该人物卡模版。".into()))?;
    // 解压和改写 Excel 属于阻塞工作，交给阻塞线程池处理。
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

/// 在线程池读取模板，避免同步解析 Excel 占用异步执行线程。
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
