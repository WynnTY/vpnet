/*!
VPNet Web Management Interface

现代化的Web管理界面，包括：
- 透明玻璃UI设计
- 完整的深色/浅色主题支持
- 响应式设计
- 实时数据刷新
- 原生中文支持
- 移动端和桌面端优化
*/

use axum::{Router, routing::get, Extension};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use vpnet_web::api::ApiState;
use vpnet_web::config::WebConfig;
use vpnet_web::handler::{health_check, get_nodes, get_node, update_node, delete_node, get_stats};
use vpnet_web::handler::{get_devices, get_device, update_device, delete_device, restart_device};
use vpnet_web::handler::{get_routes, add_route, delete_route, update_route};
use vpnet_web::handler::{get_auth, login, logout, refresh_token};

mod api;
mod config;
mod handler;
mod utils;
mod middleware;

/// 启动Web服务器
pub async fn start_web_server(
    addr: SocketAddr,
    state: ApiState
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建CORS层
    let cors = CorsLayer::permissive();
    
    // 创建压缩层
    let compression = CompressionLayer::new();
    
    // 静态文件服务
    let static_files = ServeDir::new("static");
    
    // 创建路由
    let app = Router::new()
        // API路由
        .route("/api/health", get(health_check))
        .route("/api/nodes", get(get_nodes))
        .route("/api/nodes/:id", get(get_node))
        .route("/api/nodes/:id", put(update_node))
        .route("/api/nodes/:id", delete(delete_node))
        .route("/api/stats", get(get_stats))
        .route("/api/devices", get(get_devices))
        .route("/api/devices/:id", get(get_device))
        .route("/api/devices/:id", put(update_device))
        .route("/api/devices/:id", delete(delete_device))
        .route("/api/devices/:id/restart", post(restart_device))
        .route("/api/routes", get(get_routes))
        .route("/api/routes", post(add_route))
        .route("/api/routes/:id", delete(delete_route))
        .route("/api/routes/:id", put(update_route))
        .route("/api/auth", get(get_auth))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/refresh", post(refresh_token))
        // 静态文件服务
        .nest_service("/", static_files)
        // 应用中间件
        .layer(cors)
        .layer(compression)
        .layer(Extension(state))
        // 应用认证中间件
        .layer(middleware::auth::AuthMiddleware::new());
    
    // 启动服务器
    log::info!("Web server starting on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    // 加载配置
    let config = WebConfig::default();
    
    // 解析地址
    let addr: SocketAddr = format!("{}:{}", config.bind, config.port)
        .parse()?;
    
    // 创建API状态
    let state = ApiState::new(config.clone())?;
    
    // 启动Web服务器
    start_web_server(addr, state).await
}
