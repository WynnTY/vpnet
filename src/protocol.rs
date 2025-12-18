/*!
VPNet协议模块

定义VPNet的网络协议，包括：
- 消息类型和格式
- 数据包结构
- 协议常量
- 状态码
*/

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// VPNet协议版本
pub const PROTOCOL_VERSION: u8 = 1;

/// 协议消息类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// 握手请求
    HandshakeRequest = 1,
    /// 握手响应
    HandshakeResponse = 2,
    /// 节点发现
    NodeDiscovery = 3,
    /// 节点信息
    NodeInfo = 4,
    /// 数据转发
    DataForward = 5,
    /// 心跳包
    Heartbeat = 6,
    /// 路由更新
    RouteUpdate = 7,
    /// 连接关闭
    ConnectionClose = 8,
    /// 授权请求
    AuthRequest = 9,
    /// 授权响应
    AuthResponse = 10,
}

/// 握手请求消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub version: u8,
    pub public_key: Vec<u8>,
    pub node_id: String,
    pub node_name: String,
    pub supported_protocols: Vec<u8>,
    pub capabilities: u32,
}

/// 握手响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub version: u8,
    pub public_key: Vec<u8>,
    pub node_id: String,
    pub node_name: String,
    pub status: u8,
    pub message: String,
    pub session_key: Vec<u8>,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub node_name: String,
    pub public_key: Vec<u8>,
    pub address: SocketAddr,
    pub virtual_ip: String,
    pub subnet: String,
    pub online: bool,
    pub last_seen: u64,
    pub capabilities: u32,
}

/// 数据转发消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataForward {
    pub source_node: String,
    pub dest_node: String,
    pub data: Vec<u8>,
    pub protocol: u8, // 0x0800 for IPv4, 0x86DD for IPv6
}

/// 心跳包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub node_id: String,
    pub timestamp: u64,
    pub load: f32,
    pub uptime: u64,
}

/// 路由更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteUpdate {
    pub node_id: String,
    pub routes: Vec<RouteEntry>,
}

/// 路由条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub network: String,
    pub mask: String,
    pub gateway: String,
    pub metric: u32,
}

/// 授权请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub node_id: String,
    pub public_key: Vec<u8>,
    pub request_time: u64,
    pub signature: Vec<u8>,
}

/// 授权响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub node_id: String,
    pub status: u8,
    pub message: String,
    pub token: Option<String>,
    pub expires_at: Option<u64>,
}

/// VPNet数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub magic: u32,          // 魔术字: VNET (0x564E4554)
    pub version: u8,         // 协议版本
    pub msg_type: MessageType, // 消息类型
    pub flags: u8,           // 标志位
    pub length: u16,         // 数据包长度
    pub checksum: u16,       // 校验和
    pub data: Vec<u8>,       // 数据包内容
}

/// 节点状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    /// 离线
    Offline = 0,
    /// 在线
    Online = 1,
    /// 连接中
    Connecting = 2,
    /// 已授权
    Authorized = 3,
    /// 未授权
    Unauthorized = 4,
    /// 错误
    Error = 5,
}

/// 协议常量
pub mod constants {
    /// 魔术字
    pub const MAGIC: u32 = 0x564E4554; // "VNET"
    
    /// 最大消息长度
    pub const MAX_MESSAGE_LENGTH: u16 = 16384;
    
    /// 心跳间隔（秒）
    pub const HEARTBEAT_INTERVAL: u64 = 30;
    
    /// 超时时间（秒）
    pub const TIMEOUT: u64 = 120;
    
    /// 重试次数
    pub const MAX_RETRIES: u32 = 3;
    
    /// 默认MTU
    pub const DEFAULT_MTU: u32 = 1420;
}

/// 计算数据包校验和
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    let len = data.len();
    
    // 处理16位对齐的数据
    while i < len - 1 {
        sum += ((data[i] as u32) << 8) | data[i + 1] as u32;
        i += 2;
    }
    
    // 处理剩余的字节
    if i < len {
        sum += (data[i] as u32) << 8;
    }
    
    // 折叠进位
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    // 取反
    !sum as u16
}

/// 验证数据包校验和
pub fn verify_checksum(data: &[u8], checksum: u16) -> bool {
    calculate_checksum(data) == checksum
}
