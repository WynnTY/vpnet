/*!
VPNet Server 配置模块

定义和加载服务器配置，包括：
- 服务器基本配置
- 虚拟设备配置
- 节点配置
- API配置
- Web配置
- 认证配置
*/

use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;
use rand::Rng;
use base64::Engine;

/// 配置错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Toml parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    
    #[error("Missing required configuration: {0}")]
    Missing(String),
}

/// 服务器配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub server: Server,
    pub virtual_device: VirtualDevice,
    pub node: Node,
    pub api: Api,
    pub web: Web,
    pub auth: Auth,
}

/// 服务器基本配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub bind: String,
    pub port: u16,
    pub workers: u32,
    pub timeout: u64,
}

/// 虚拟设备配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VirtualDevice {
    pub name: String,
    pub ip: String,
    pub subnet: String,
    pub gateway: String,
    pub mtu: u32,
    pub enable_ipv6: bool,
    pub ipv6_address: Option<String>,
}

/// 节点配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub key_file: String,
    pub auto_discovery: bool,
    pub discovery_interval: u64,
}

/// API配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Api {
    pub bind: String,
    pub port: u16,
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
    pub rate_limit: u32,
}

/// Web配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Web {
    pub bind: String,
    pub port: u16,
    pub enable_tls: bool,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub enable_compression: bool,
}

/// 认证配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    pub enable: bool,
    pub secret_key: String,
    pub token_expiry: u64,
    pub allow_anonymous: bool,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

/// 生成默认配置
pub fn default_config() -> ServerConfig {
    let mut rng = rand::thread_rng();
    let secret_key = base64::engine::general_purpose::STANDARD.encode(
        rng.gen::<[u8; 32]>(),
    );
    
    ServerConfig {
        server: Server {
            bind: "0.0.0.0".to_string(),
            port: 51820,
            workers: 4,
            timeout: 30,
        },
        virtual_device: VirtualDevice {
            name: "vpnet0".to_string(),
            ip: "10.0.0.1".to_string(),
            subnet: "255.255.255.0".to_string(),
            gateway: "10.0.0.1".to_string(),
            mtu: 1420,
            enable_ipv6: false,
            ipv6_address: None,
        },
        node: Node {
            id: format!("node_{:x}", rng.gen::<u64>()),
            name: "VPNet Server".to_string(),
            key_file: "vpnet-key.json".to_string(),
            auto_discovery: true,
            discovery_interval: 60,
        },
        api: Api {
            bind: "0.0.0.0".to_string(),
            port: 51821,
            enable_cors: true,
            allowed_origins: vec!["*".to_string()],
            rate_limit: 100,
        },
        web: Web {
            bind: "0.0.0.0".to_string(),
            port: 51822,
            enable_tls: false,
            tls_cert: None,
            tls_key: None,
            enable_compression: true,
        },
        auth: Auth {
            enable: true,
            secret_key: secret_key,
            token_expiry: 86400,
            allow_anonymous: false,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        },
    }
}

/// 保存配置到文件
pub fn save_config(config: &ServerConfig, path: &str) -> Result<(), ConfigError> {
    let toml_str = toml::to_string_pretty(config)?;
    let mut file = File::create(path)?;
    file.write_all(toml_str.as_bytes())?;
    Ok(())
}

/// 加载或生成配置
pub fn load_or_generate_config(path: &str) -> Result<ServerConfig, ConfigError> {
    if Path::new(path).exists() {
        // 加载现有配置
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        // 生成新配置
        let config = default_config();
        save_config(&config, path)?;
        Ok(config)
    }
}

/// 加载或生成密钥对
pub fn load_or_generate_keys(config: &ServerConfig) -> Result<(Vec<u8>, Vec<u8>), ConfigError> {
    let key_path = Path::new(&config.node.key_file);
    
    if key_path.exists() {
        // 加载现有密钥
        let mut file = File::open(key_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let keys: serde_json::Value = serde_json::from_str(&content)?;
        let public_key = base64::engine::general_purpose::STANDARD.decode(
            keys["public_key"].as_str().ok_or(ConfigError::Missing("public_key".to_string()))?
        )?;
        let private_key = base64::engine::general_purpose::STANDARD.decode(
            keys["private_key"].as_str().ok_or(ConfigError::Missing("private_key".to_string()))?
        )?;
        
        Ok((public_key, private_key))
    } else {
        // 生成新密钥对
        let mut rng = rand::thread_rng();
        let public_key = rng.gen::<[u8; 32]>().to_vec();
        let private_key = rng.gen::<[u8; 32]>().to_vec();
        
        // 保存密钥到文件
        let keys = serde_json::json!({
            "public_key": base64::engine::general_purpose::STANDARD.encode(&public_key),
            "private_key": base64::engine::general_purpose::STANDARD.encode(&private_key),
            "generated_at": chrono::Utc::now().to_rfc3339()
        });
        
        let keys_str = serde_json::to_string_pretty(&keys)?;
        let mut file = File::create(key_path)?;
        file.write_all(keys_str.as_bytes())?;
        
        Ok((public_key, private_key))
    }
}

/// 验证配置
pub fn validate_config(config: &ServerConfig) -> Result<(), ConfigError> {
    // 验证服务器配置
    if config.server.bind.is_empty() {
        return Err(ConfigError::Missing("server.bind".to_string()));
    }
    
    if config.server.port == 0 {
        return Err(ConfigError::Invalid("server.port must be greater than 0".to_string()));
    }
    
    // 验证虚拟设备配置
    if config.virtual_device.name.is_empty() {
        return Err(ConfigError::Missing("virtual_device.name".to_string()));
    }
    
    if config.virtual_device.ip.is_empty() {
        return Err(ConfigError::Missing("virtual_device.ip".to_string()));
    }
    
    if config.virtual_device.subnet.is_empty() {
        return Err(ConfigError::Missing("virtual_device.subnet".to_string()));
    }
    
    if config.virtual_device.gateway.is_empty() {
        return Err(ConfigError::Missing("virtual_device.gateway".to_string()));
    }
    
    // 验证节点配置
    if config.node.id.is_empty() {
        return Err(ConfigError::Missing("node.id".to_string()));
    }
    
    if config.node.name.is_empty() {
        return Err(ConfigError::Missing("node.name".to_string()));
    }
    
    // 验证API配置
    if config.api.bind.is_empty() {
        return Err(ConfigError::Missing("api.bind".to_string()));
    }
    
    if config.api.port == 0 {
        return Err(ConfigError::Invalid("api.port must be greater than 0".to_string()));
    }
    
    // 验证Web配置
    if config.web.bind.is_empty() {
        return Err(ConfigError::Missing("web.bind".to_string()));
    }
    
    if config.web.port == 0 {
        return Err(ConfigError::Invalid("web.port must be greater than 0".to_string()));
    }
    
    // 验证认证配置
    if config.auth.secret_key.is_empty() {
        return Err(ConfigError::Missing("auth.secret_key".to_string()));
    }
    
    Ok(())
}
