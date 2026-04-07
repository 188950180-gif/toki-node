//! REST API 服务器

use std::net::SocketAddr;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::{handlers::ApiState, routes};

/// API 服务器配置
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// 监听地址
    pub listen_addr: String,
    /// 启用 CORS
    pub enable_cors: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            listen_addr: "0.0.0.0:8080".to_string(),
            enable_cors: true,
        }
    }
}

/// API 服务器
pub struct ApiServer {
    config: ApiConfig,
    router: Router,
}

impl ApiServer {
    /// 创建新的 API 服务器
    pub fn new(config: ApiConfig, state: ApiState) -> Self {
        let router = routes::create_routes(state);

        let router = if config.enable_cors {
            router.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
        } else {
            router
        };

        ApiServer { config, router }
    }

    /// 启动服务器
    pub async fn run(&self) {
        let addr: SocketAddr = self.config.listen_addr.parse().unwrap();
        info!("API 服务器启动: {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, self.router.clone()).await.unwrap();
    }
}
