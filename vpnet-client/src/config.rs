/*!
VPNet Client 配置模块

定义和加载客户端配置，包括：
- 客户端基本配置
- 服务器配置
- 虚拟设备配置
- 认证配置
- 监控配置
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

/// 客户端配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClientConfig {
    pub client: Client,
    pub server: Server,
    pub virtual_device: VirtualDevice,
    pub auth: Auth,
    pub monitor: Monitor,
}

/// 客户端基本配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub key_file: String,
    pub enable_auto_connect: bool,
    pub reconnect_interval: u64,
    pub max_reconnect_attempts: u32,
}

/// 服务器配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub address: String,
    pub timeout: u64,
    pub enable_encryption: bool,
    pub enable_compression: bool,
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
    pub auto_config: bool,
}

/// 认证配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub token_file: String,
    pub enable_auto_login: bool,
    pub auth_timeout: u64,
}

/// 监控配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Monitor {
    pub enable: bool,
    pub interval: u64,
    pub log_level: String,
    pub enable_stats: bool,
    pub stats_file: Option<String>,
    pub stats_interval: u64,
}

/// 生成默认配置
pub fn default_config() -> ClientConfig {
    let mut rng = rand::thread_rng();
    
    ClientConfig {
        client: Client {
            id: format!("client_{:x}", rng.gen::<u64>()),
            name: format!("Client-{:x}", rng.gen::<u32>()),
            port: 51820,
            key_file: "vpnet-client-key.json".to_string(),
            enable_auto_connect: true,
            reconnect_interval: 5,
            max_reconnect_attempts: 10,
        },
        server: Server {
            address: "127.0.0.1:51820".to_string(),
            timeout: 30,
            enable_encryption: true,
            enable_compression: true,
        },
        virtual_device: VirtualDevice {
            name: "vpnet0".to_string(),
            ip: "10.0.0.2".to_string(),
            subnet: "255.255.255.0".to_string(),
            gateway: "10.0.0.1".to_string(),
            mtu: 1420,
            enable_ipv6: false,
            ipv6_address: None,
            auto_config: true,
        },
        auth: Auth {
            username: None,
            password: None,
            token: None,
            token_file: "vpnet-token.json".to_string(),
            enable_auto_login: true,
            auth_timeout: 60,
        },
        monitor: Monitor {
            enable: true,
            interval: 30,
            log_level: "info".to_string(),
            enable_stats: true,
            stats_file: Some("vpnet-stats.json".to_string()),
            stats_interval: 60,
        },
    }
}

/// 保存配置到文件
pub fn save_config(config: &ClientConfig, path: &str) -> Result<(), ConfigError> {
    let toml_str = toml::to_string_pretty(config)?;
    let mut file = File::create(path)?;
    file.write_all(toml_str.as_bytes())?;
    Ok(())
}

/// 加载或生成配置
pub fn load_or_generate_config(path: &str) -> Result<ClientConfig, ConfigError> {
    if Path::new(path).exists() {
        // 加载现有配置
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: ClientConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        // 生成新配置
        let config = default_config();
        save_config(&config, path)?;
        Ok(config)
    }
}

/// 加载或生成密钥对
pub fn load_or_generate_keys(config: &ClientConfig) -> Result<(Vec<u8>, Vec<u8>), ConfigError> {
    let key_path = Path::new(&config.client.key_file);
    
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
        let keys = serde_json::json!({"public_key": base64::engine::general_purpose::STANDARD.encode(&public_key),"private_key": base64::engine::general_purpose::STANDARD.encode(&private_key),"generated_at": chrono::Utc::now().to_rfc3339()});
        
        let keys_str = serde_json::to_string_pretty(&keys)?;
        let mut file = File::create(key_path)?;
        file.write_all(keys_str.as_bytes())?;
        
        Ok((public_key, private_key))
    }
}

/// 验证配置
pub fn validate_config(config: &ClientConfig) -> Result<(), ConfigError> {
    // 验证客户端配置
    if config.client.id.is_empty() {
        return Err(ConfigError::Missing("client.id".to_string()));
    }
    
    if config.client.name.is_empty() {
        return Err(ConfigError::Missing("client.name".to_string()));
    }
    
    if config.client.port == 0 {
        return Err(ConfigError::Invalid("client.port must be greater than 0".to_string()));
    }
    
    // 验证服务器配置
    if config.server.address.is_empty() {
        return Err(ConfigError::Missing("server.address".to_string()));
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
    
    // 验证认证配置
    if config.auth.token_file.is_empty() {
        return Err(ConfigError::Missing("auth.token_file".to_string()));
    }
    
    // 验证监控配置
    if config.monitor.log_level.is_empty() {
        return Err(ConfigError::Missing("monitor.log_level".to_string()));
    }
    
    Ok(())
}

/// 保存认证令牌
pub fn save_auth_token(config: &ClientConfig, token: &str) -> Result<(), ConfigError> {
    let token_path = Path::new(&config.auth.token_file);
    let token_data = serde_json::json!({"token": token,"expires_at": chrono::Utc::now().timestamp() + config.auth.auth_timeout as i64});
    
    let token_str = serde_json::to_string_pretty(&token_data)?;
    let mut file = File::create(token_path)?;
    file.write_all(token_str.as_bytes())?;
    Ok(())
}

/// 加载认证令牌
pub fn load_auth_token(config: &ClientConfig) -> Result<Option<String>, ConfigError> {
    let token_path = Path::new(&config.auth.token_file);
    
    if token_path.exists() {
        let mut file = File::open(token_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let token_data: serde_json::Value = serde_json::from_str(&content)?;
        let token = token_data["token"].as_str().map(|s| s.to_string());
        let expires_at = token_data["expires_at"].as_i64();
        
        // 检查令牌是否过期
        if let (Some(token), Some(expires_at)) = (token.clone(), expires_at) {
            if chrono::Utc::now().timestamp() < expires_at {
                Ok(Some(token))
            } else {
                // 令牌已过期，删除令牌文件
                std::fs::remove_file(token_path)?;
                Ok(None)
            }
        } else {
            Ok(token)
        }
    } else {
        Ok(None)
    }
}
