use crate::frontend::{PERSONAS_CSS, pages::personas};
use axum::{http::header, response::Html};

pub(crate) async fn page() -> Html<String> {
    Html(personas::render())
}

pub(crate) async fn stylesheet() -> ([(header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        PERSONAS_CSS,
    )
}
