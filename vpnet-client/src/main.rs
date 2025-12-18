/*!
VPNet Client - 轻量级、快捷的虚拟局域网客户端

客户端主要功能：
- 连接到VPNet网络
- 与服务端进行认证和授权
- 管理本地虚拟设备
- 处理网络数据转发
- 跨平台支持
- 轻便快捷的运行
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
use vpnet::{NetworkManager, DeviceManager, VirtualDeviceConfig};
use vpnet_client::config::ClientConfig;
use vpnet_client::auth::AuthClient;
use vpnet_client::device::setup_virtual_device;
use vpnet_client::network::connect_to_server;
use vpnet_client::monitor::start_monitor;

mod config;
mod auth;
mod device;
mod network;
mod monitor;
mod utils;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "vpnet-client.toml")]
    config: String,
    
    /// 启用调试日志
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,
    
    /// 连接到指定服务器
    #[arg(short, long)]
    server: Option<String>,
    
    /// 虚拟IP地址
    #[arg(long)]
    virtual_ip: Option<String>,
    
    /// 以守护进程模式运行
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    daemon: bool,
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
    
    log::info!("VPNet Client starting...");
    
    // 加载配置
    let mut config_file = File::open(&args.config)?;
    let mut config_content = String::new();
    config_file.read_to_string(&mut config_content)?;
    let mut config: ClientConfig = toml::from_str(&config_content)?;
    
    // 从命令行参数覆盖配置
    if let Some(server) = args.server {
        config.server.address = server;
    }
    if let Some(virtual_ip) = args.virtual_ip {
        config.virtual_device.ip = virtual_ip;
    }
    
    log::debug!("Config loaded: {:?}", config);
    
    // 解析服务器地址
    let server_addr: SocketAddr = config.server.address.parse()?;
    
    // 初始化认证客户端
    let auth_client = Arc::new(Mutex::new(AuthClient::new(
        config.auth.clone(),
        server_addr
    )?));
    
    // 连接到服务器并进行认证
    let auth_token = auth_client.lock().await.authenticate().await?;
    log::info!("Authenticated with server successfully");
    
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
    
    // 初始化网络管理器
    let local_addr: SocketAddr = format!("0.0.0.0:{}", config.client.port)
        .parse()?;
    
    let network_manager = Arc::new(Mutex::new(NetworkManager::new(
        local_addr,
        config.client.id.clone(),
        config.client.name.clone(),
        auth_client.lock().await.get_public_key().await,
        auth_client.lock().await.get_private_key().await.as_ref()
    )?));
    
    // 启动网络服务
    network_manager.lock().await.start().await;
    log::info!("Network service started on {}", local_addr);
    
    // 连接到服务器
    let connection = connect_to_server(
        network_manager.clone(),
        server_addr,
        auth_token.clone(),
        config.server.clone()
    ).await?;
    log::info!("Connected to server {}", server_addr);
    
    // 启动监控任务
    let monitor_handle = start_monitor(
        network_manager.clone(),
        device.clone(),
        config.monitor.interval
    );
    
    // 设置路由
    if let Err(e) = device::setup_routes(&config.virtual_device).await {
        log::warn!("Failed to setup routes: {}", e);
    }
    
    log::info!("VPNet Client started successfully");
    log::info!("Virtual IP: {}", config.virtual_device.ip);
    log::info!("Subnet: {}", config.virtual_device.subnet);
    log::info!("Connected to server: {}", config.server.address);
    
    // 主循环 - 处理信号和优雅关闭
    let signal = tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    
    log::info!("Received shutdown signal, stopping services...");
    
    // 清理路由
    if let Err(e) = device::cleanup_routes(&config.virtual_device).await {
        log::warn!("Failed to cleanup routes: {}", e);
    }
    
    // 关闭虚拟设备
    device.lock().await.stop().await?;
    
    // 等待监控任务结束
    if let Some(handle) = monitor_handle {
        handle.abort();
    }
    
    log::info!("VPNet Client stopped successfully");
    
    Ok(())
}
