use std::io::Error;

use axum::{
    extract::rejection::JsonRejection, http::StatusCode, routing::MethodRouter, Json, Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

type AxumRoute = (&'static str, MethodRouter);

pub use axum::{extract, http, routing};
pub use serde_json::{json, Value};

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

pub struct AxumServer {
    router: Router,
    listener: TcpListener,
}

impl AxumServer {
    pub async fn new(
        server_port: String,
        routes: Vec<AxumRoute>,
        cors: Option<CorsLayer>,
    ) -> Result<Self, Error> {
        let address = format!("0.0.0.0:{}", server_port);

        let mut router = Router::new();
        for route in routes {
            router = router.route(route.0, route.1);
        }

        if let Some(cors) = cors {
            router = router.layer(ServiceBuilder::new().layer(cors));
        }

        let listener = TcpListener::bind(address).await?;

        Ok(AxumServer { router, listener })
    }

    pub async fn run(self) -> Result<(), Error> {
        axum::serve(self.listener, self.router).await
    }
}
