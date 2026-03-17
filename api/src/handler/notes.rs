use axum::{
    extract::State,
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone, Default)]
pub struct AppState;

#[derive(Serialize, ToSchema)]
pub struct ViewerContext {
    pub subject: Option<String>,
    pub auth_type: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct NoteSummary {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
}

#[derive(Serialize, ToSchema)]
pub struct NotesResponse {
    pub viewer: ViewerContext,
    pub notes: Vec<NoteSummary>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateNote {
    pub title: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateNoteResponse {
    pub accepted: bool,
    pub persisted: bool,
    pub title: String,
    pub viewer: ViewerContext,
    pub message: &'static str,
}

#[utoipa::path(
    get,
    path = "/api/v1/viewer",
    responses((status = 200, description = "Viewer context", body = ViewerContext))
)]
pub async fn viewer(headers: HeaderMap) -> Json<ViewerContext> {
    Json(viewer_from_headers(&headers))
}

#[utoipa::path(
    get,
    path = "/api/v1/notes",
    responses((status = 200, description = "Sample notes", body = NotesResponse))
)]
pub async fn list_notes(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Json<NotesResponse> {
    Json(NotesResponse {
        viewer: viewer_from_headers(&headers),
        notes: vec![
            NoteSummary {
                id: "smoke-001",
                title: "Gateway-authenticated note",
                summary: "This app assumes auth has already been handled upstream.",
            },
            NoteSummary {
                id: "smoke-002",
                title: "Scope-aware sample",
                summary: "Use this endpoint later to demo host/path/scope enforcement.",
            },
        ],
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/notes",
    request_body = CreateNote,
    responses((status = 201, description = "Accepted note creation", body = CreateNoteResponse))
)]
pub async fn create_note(
    State(_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateNote>,
) -> (axum::http::StatusCode, Json<CreateNoteResponse>) {
    (
        axum::http::StatusCode::CREATED,
        Json(CreateNoteResponse {
            accepted: true,
            persisted: false,
            title: payload.title,
            viewer: viewer_from_headers(&headers),
            message: "Write path reached business app; persistence stays disabled in smoke mode.",
        }),
    )
}

fn viewer_from_headers(headers: &HeaderMap) -> ViewerContext {
    ViewerContext {
        subject: header_value(headers, "x-auth-subject"),
        auth_type: header_value(headers, "x-auth-type"),
        scopes: header_value(headers, "x-auth-scopes")
            .map(|raw| {
                raw.split_whitespace()
                    .filter(|item| !item.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/viewer", get(viewer))
        .route("/api/v1/notes", get(list_notes).post(create_note))
        .with_state(state)
}
