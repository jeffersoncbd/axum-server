use axum::{
    extract::{rejection::JsonRejection, Request as AxumRequest},
    http::StatusCode,
    middleware::{self, Next},
    response::Response as AxumResponse,
    Json, Router,
};
use logger;
use serde_json::{json, Value};
use std::env;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors;

pub use axum::routing::{get, MethodRouter};

pub type Request<T> = Result<Json<T>, JsonRejection>;
pub type Response = (StatusCode, Json<Value>);

pub struct MakeResponse;
impl MakeResponse {
    pub fn ok(body: Value) -> Response {
        (StatusCode::OK, Json(body))
    }
    pub fn bad_request(body: Value) -> Response {
        (StatusCode::BAD_REQUEST, Json(body))
    }
    pub fn unauthorized(body: Value) -> Response {
        (StatusCode::UNAUTHORIZED, Json(body))
    }
    pub fn internal_server_error() -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "feedback": "Internal server error" })),
        )
    }
}

const FILE_NAME: &str = "🌐 AxumServer/main";

fn get_server_port() -> String {
    logger::log(
        FILE_NAME,
        "Recuperando valor da variável \"SERVER_PORT\"...",
    );
    if let Err(error) = dotenvy::dotenv() {
        let message = format!(
            "{}: \"{}\"",
            "Atenção! erro ao tentar carregar arquivo .env!",
            error.to_string()
        );
        logger::log("🟡 AxumServer/main", &message);
    }
    env::var("SERVER_PORT")
        .expect("\n\t❌ A variável de ambiente \"SERVER_PORT\" não foi definida!\n\n")
}

async fn logger_middleware(request: AxumRequest, next: Next) -> AxumResponse {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let response = next.run(request).await;
    let status = response.status().to_string();

    let header = format!("🌐 {} {}", method, uri);
    logger::log(&header, &status);

    response
}

pub async fn start(routes: Vec<(&'static str, MethodRouter)>) {
    let server_port = get_server_port();

    let mut router = Router::new();

    for route in routes.iter() {
        router = router.route(route.0, route.1.clone())
    }
    drop(routes);
    router = router.layer(
        ServiceBuilder::new()
            .layer(
                cors::CorsLayer::new()
                    .allow_headers(cors::Any)
                    .allow_methods(cors::Any)
                    .allow_origin(cors::Any),
            )
            .layer(middleware::from_fn(logger_middleware)),
    );

    let address = format!("0.0.0.0:{}", &server_port);
    let listener = TcpListener::bind(&address)
        .await
        .expect("\n\t❌ Falha o tentar criar listener...\n\n");

    logger::log(
        FILE_NAME,
        &format!("Servidor inciado na porta \"{}\"", server_port),
    );

    axum::serve(listener, router)
        .await
        .expect("\n\t❌ Falha o tentar iniciar o servidor\n\n");
}
