/*!
VPNet Server - 高性能去中心化虚拟局域网服务端

服务端主要功能：
- 管理客户端连接
- 节点授权和认证
- 虚拟网络管理
- Web管理界面
- 跨平台支持
*/

use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;
use vpnet::{NetworkManager, DeviceManager, VirtualDeviceConfig, default_config};
use vpnet_server::config::ServerConfig;
use vpnet_server::auth::AuthManager;
use vpnet_server::api::start_api_server;
use vpnet_server::node::{NodeManager, Node};
use vpnet_server::web::start_web_server;

mod config;
mod auth;
mod api;
mod node;
mod web;
mod utils;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "vpnet-server.toml")]
    config: String,
    
    /// 启用调试日志
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,
    
    /// 绑定地址
    #[arg(short, long)]
    bind: Option<String>,
    
    /// 端口
    #[arg(short, long)]
    port: Option<u16>,
    
    /// 虚拟IP地址
    #[arg(long)]
    virtual_ip: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    let args = Args::parse();
    
    // 初始化日志
    let mut logger = Builder::new();
    logger.filter(None, if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });
    logger.init();
    
    log::info!("VPNet Server starting...");
    
    // 加载配置
    let mut config_file = File::open(&args.config)?;
    let mut config_content = String::new();
    config_file.read_to_string(&mut config_content)?;
    let mut config: ServerConfig = toml::from_str(&config_content)?;
    
    // 从命令行参数覆盖配置
    if let Some(bind) = args.bind {
        config.server.bind = bind;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(virtual_ip) = args.virtual_ip {
        config.virtual_device.ip = virtual_ip;
    }
    
    log::debug!("Config loaded: {:?}", config);
    
    // 生成或加载密钥对
    let (public_key, private_key) = config::load_or_generate_keys(&config)?;
    
    // 初始化认证管理器
    let auth_manager = Arc::new(Mutex::new(AuthManager::new(config.auth.clone())?));
    
    // 初始化节点管理器
    let node_manager = Arc::new(Mutex::new(NodeManager::new(config.node.clone())?));
    
    // 初始化网络管理器
    let local_addr: SocketAddr = format!("{}:{}", config.server.bind, config.server.port)
        .parse()?;
    
    let network_manager = Arc::new(Mutex::new(NetworkManager::new(
        local_addr,
        config.node.id.clone(),
        config.node.name.clone(),
        public_key,
        &private_key
    )?));
    
    // 初始化设备管理器
    let mut device_manager = DeviceManager::new();
    
    // 创建虚拟设备
    let virtual_ip = config.virtual_device.ip.parse()?;
    let device_config = VirtualDeviceConfig {
        name: config.virtual_device.name.clone(),
        ip: virtual_ip,
        subnet: config.virtual_device.subnet.parse()?,
        gateway: config.virtual_device.gateway.parse()?,
        mtu: config.virtual_device.mtu,
        mac: None,
    };
    
    let device_id = device_manager.create_device(device_config).await?;
    let device = device_manager.get_device(&device_id).await?;
    
    // 启动虚拟设备
    device.lock().await.start().await?;
    log::info!("Virtual device {} started successfully", config.virtual_device.name);
    
    // 启动网络服务
    network_manager.lock().await.start().await;
    log::info!("Network service started on {}", local_addr);
    
    // 启动API服务器
    let api_addr: SocketAddr = format!("{}:{}", config.api.bind, config.api.port)
        .parse()?;
    let api_handle = tokio::spawn(start_api_server(
        api_addr,
        auth_manager.clone(),
        node_manager.clone(),
        network_manager.clone(),
        Arc::new(Mutex::new(device_manager.clone())),
        config.api.clone()
    ));
    
    // 启动Web管理界面
    let web_addr: SocketAddr = format!("{}:{}", config.web.bind, config.web.port)
        .parse()?;
    let web_handle = tokio::spawn(start_web_server(
        web_addr,
        auth_manager.clone(),
        node_manager.clone(),
        network_manager.clone(),
        config.web.clone()
    ));
    
    log::info!("VPNet Server started successfully");
    log::info!("Web management interface available at http://{}", web_addr);
    log::info!("API server available at http://{}", api_addr);
    
    // 主循环 - 处理信号和优雅关闭
    let signal = tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    
    log::info!("Received shutdown signal, stopping services...");
    
    // 关闭虚拟设备
    device.lock().await.stop().await?;
    
    // 等待API和Web服务器关闭
    api_handle.await??;
    web_handle.await??;
    
    log::info!("VPNet Server stopped successfully");
    
    Ok(())
}
