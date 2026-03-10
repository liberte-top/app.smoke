use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod handler;

use handler::notes::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(handler::health::health, handler::notes::viewer, handler::notes::list_notes),
    components(schemas(
        handler::health::Health,
        handler::notes::ViewerContext,
        handler::notes::NoteSummary,
        handler::notes::NotesResponse,
    ))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState);
    let port = std::env::var("PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(4310);

    let app = Router::new()
        .merge(handler::health::routes())
        .merge(handler::notes::routes(state))
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.expect("bind failed");
    eprintln!("app.smoke.api listening on {addr}");
    axum::serve(listener, app).await.expect("serve error");
}
