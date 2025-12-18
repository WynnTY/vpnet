/*!
VPNet网络模块

处理P2P通信、节点发现和网络连接管理，包括：
- UDP/TCP通信
- 节点发现和连接
- NAT穿透
- 连接管理
*/

use std::net::{SocketAddr, UdpSocket, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use std::collections::HashMap;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use crate::protocol::*;
use crate::crypto::*;

/// 网络管理器
pub struct NetworkManager {
    udp_socket: Arc<UdpSocket>,
    tcp_listener: Option<Arc<TcpListener>>,
    local_addr: SocketAddr,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    crypto: Arc<Mutex<CryptoContext>>,
    node_id: String,
    node_name: String,
    public_key: Vec<u8>,
}

/// 对等节点
pub struct Peer {
    pub node_id: String,
    pub node_name: String,
    pub address: SocketAddr,
    pub virtual_ip: String,
    pub public_key: Vec<u8>,
    pub status: NodeStatus,
    pub last_seen: u64,
    pub capabilities: u32,
}

/// NAT类型
pub enum NatType {
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    Unknown,
}

impl NetworkManager {
    /// 创建新的网络管理器
    pub fn new(
        local_addr: SocketAddr,
        node_id: String,
        node_name: String,
        public_key: Vec<u8>,
        crypto_key: &[u8]
    ) -> Result<Self, std::io::Error> {
        let udp_socket = UdpSocket::bind(local_addr)?;
        udp_socket.set_nonblocking(true)?;
        
        let crypto = CryptoContext::new(crypto_key, CryptoAlgorithm::AesGcm256);
        
        Ok(Self {
            udp_socket: Arc::new(udp_socket),
            tcp_listener: None,
            local_addr,
            peers: Arc::new(RwLock::new(HashMap::new())),
            crypto: Arc::new(Mutex::new(crypto)),
            node_id,
            node_name,
            public_key,
        })
    }
    
    /// 启动TCP监听器
    pub fn start_tcp_listener(&mut self, tcp_port: u16) -> Result<(), std::io::Error> {
        let tcp_addr = SocketAddr::new(self.local_addr.ip(), tcp_port);
        let listener = TcpListener::bind(tcp_addr)?;
        listener.set_nonblocking(true)?;
        self.tcp_listener = Some(Arc::new(listener));
        Ok(())
    }
    
    /// 启动网络服务
    pub async fn start(&self) {
        // 启动UDP接收任务
        let udp_socket = self.udp_socket.clone();
        let crypto = self.crypto.clone();
        let peers = self.peers.clone();
        let node_id = self.node_id.clone();
        
        tokio::spawn(async move {
            let mut buf = [0u8; MAX_PACKET_SIZE];
            loop {
                match udp_socket.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        let data = &buf[..len];
                        // 处理接收到的数据包
                        tokio::spawn(handle_udp_packet(
                            data.to_vec(), 
                            addr, 
                            crypto.clone(), 
                            peers.clone(),
                            node_id.clone()
                        ));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        log::error!("UDP receive error: {}", e);
                        break;
                    }
                }
            }
        });
        
        // 启动心跳任务
        let peers = self.peers.clone();
        let node_id = self.node_id.clone();
        let udp_socket = self.udp_socket.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(constants::HEARTBEAT_INTERVAL));
            loop {
                interval.tick().await;
                // 发送心跳包
                send_heartbeat(&udp_socket, &node_id, &peers).await;
                // 清理超时节点
                cleanup_timeout_peers(&peers).await;
            }
        });
    }
    
    /// 发送数据包到指定节点
    pub async fn send_packet(&self, peer_id: &str, packet: &Packet) -> Result<(), &'static str> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            let data = serde_json::to_vec(packet).map_err(|_| "Serialization failed")?;
            self.udp_socket.send_to(&data, peer.address)
                .map_err(|_| "Send failed")?;
            Ok(())
        } else {
            Err("Peer not found")
        }
    }
    
    /// 发现节点
    pub async fn discover_nodes(&self, discovery_addr: SocketAddr) -> Result<(), &'static str> {
        let discovery_msg = Packet {
            magic: constants::MAGIC,
            version: PROTOCOL_VERSION,
            msg_type: MessageType::NodeDiscovery,
            flags: 0,
            length: 0,
            checksum: 0,
            data: Vec::new(),
        };
        
        let data = serde_json::to_vec(&discovery_msg).map_err(|_| "Serialization failed")?;
        self.udp_socket.send_to(&data, discovery_addr)
            .map_err(|_| "Send failed")?;
        Ok(())
    }
    
    /// 获取所有对等节点
    pub async fn get_peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }
    
    /// 获取本地节点信息
    pub async fn get_local_info(&self) -> NodeInfo {
        NodeInfo {
            node_id: self.node_id.clone(),
            node_name: self.node_name.clone(),
            public_key: self.public_key.clone(),
            address: self.local_addr,
            virtual_ip: "10.0.0.1".to_string(), // 默认虚拟IP，实际应从配置获取
            subnet: "255.255.255.0".to_string(),
            online: true,
            last_seen: tokio::time::unix_epoch().elapsed().unwrap().as_secs(),
            capabilities: 0,
        }
    }
}

/// 处理UDP数据包
async fn handle_udp_packet(
    data: Vec<u8>,
    addr: SocketAddr,
    crypto: Arc<Mutex<CryptoContext>>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    node_id: String
) {
    // 解析数据包
    if let Ok(packet) = serde_json::from_slice::<Packet>(&data) {
        // 验证魔术字和版本
        if packet.magic != constants::MAGIC || packet.version != PROTOCOL_VERSION {
            return;
        }
        
        // 验证校验和
        if !verify_checksum(&packet.data, packet.checksum) {
            log::warn!("Invalid checksum from {}", addr);
            return;
        }
        
        // 根据消息类型处理
        match packet.msg_type {
            MessageType::HandshakeRequest => {
                handle_handshake_request(packet, addr, crypto, peers, node_id).await;
            }
            MessageType::HandshakeResponse => {
                handle_handshake_response(packet, addr, crypto, peers).await;
            }
            MessageType::NodeDiscovery => {
                handle_node_discovery(packet, addr, crypto, peers, node_id).await;
            }
            MessageType::NodeInfo => {
                handle_node_info(packet, addr, peers).await;
            }
            MessageType::Heartbeat => {
                handle_heartbeat(packet, addr, peers).await;
            }
            MessageType::DataForward => {
                handle_data_forward(packet, crypto).await;
            }
            _ => {
                log::debug!("Received unhandled message type: {:?} from {}", packet.msg_type, addr);
            }
        }
    } else {
        log::warn!("Failed to parse packet from {}", addr);
    }
}

/// 处理握手请求
async fn handle_handshake_request(
    packet: Packet,
    addr: SocketAddr,
    crypto: Arc<Mutex<CryptoContext>>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    node_id: String
) {
    // 解析握手请求
    if let Ok(req) = serde_json::from_slice::<HandshakeRequest>(&packet.data) {
        // 生成会话密钥
        let mut crypto_guard = crypto.lock().await;
        let session_key = crypto_guard.generate_key(CryptoAlgorithm::AesGcm256);
        
        // 创建握手响应
        let resp = HandshakeResponse {
            version: PROTOCOL_VERSION,
            public_key: crypto_guard.generate_key(CryptoAlgorithm::AesGcm256),
            node_id: node_id.clone(),
            node_name: "VPNet Server".to_string(),
            status: 0,
            message: "Handshake successful".to_string(),
            session_key: session_key.clone(),
        };
        
        // 发送响应
        let resp_data = serde_json::to_vec(&resp).unwrap();
        let resp_packet = Packet {
            magic: constants::MAGIC,
            version: PROTOCOL_VERSION,
            msg_type: MessageType::HandshakeResponse,
            flags: 0,
            length: resp_data.len() as u16,
            checksum: calculate_checksum(&resp_data),
            data: resp_data,
        };
        
        let resp_packet_data = serde_json::to_vec(&resp_packet).unwrap();
        // 发送UDP数据包
        
        // 添加对等节点
        let mut peers_guard = peers.write().await;
        peers_guard.insert(req.node_id.clone(), Peer {
            node_id: req.node_id.clone(),
            node_name: req.node_name.clone(),
            address: addr,
            virtual_ip: "10.0.0.2".to_string(), // 默认虚拟IP，实际应从配置获取
            public_key: req.public_key.clone(),
            status: NodeStatus::Online,
            last_seen: tokio::time::unix_epoch().elapsed().unwrap().as_secs(),
            capabilities: req.capabilities,
        });
    }
}

/// 处理握手响应
async fn handle_handshake_response(
    packet: Packet,
    addr: SocketAddr,
    crypto: Arc<Mutex<CryptoContext>>,
    peers: Arc<RwLock<HashMap<String, Peer>>>
) {
    // 解析握手响应
    if let Ok(resp) = serde_json::from_slice::<HandshakeResponse>(&packet.data) {
        // 更新会话密钥
        // let mut crypto_guard = crypto.lock().await;
        // crypto_guard.update_session_key(&resp.session_key);
        
        // 更新对等节点
        let mut peers_guard = peers.write().await;
        peers_guard.insert(resp.node_id.clone(), Peer {
            node_id: resp.node_id.clone(),
            node_name: resp.node_name.clone(),
            address: addr,
            virtual_ip: "10.0.0.1".to_string(), // 默认虚拟IP，实际应从配置获取
            public_key: resp.public_key.clone(),
            status: NodeStatus::Online,
            last_seen: tokio::time::unix_epoch().elapsed().unwrap().as_secs(),
            capabilities: 0,
        });
    }
}

/// 处理节点发现
async fn handle_node_discovery(
    packet: Packet,
    addr: SocketAddr,
    crypto: Arc<Mutex<CryptoContext>>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    node_id: String
) {
    // 发送节点信息响应
    let node_info = NodeInfo {
        node_id: node_id.clone(),
        node_name: "VPNet Server".to_string(),
        public_key: crypto.lock().await.generate_key(CryptoAlgorithm::AesGcm256),
        address: addr,
        virtual_ip: "10.0.0.1".to_string(),
        subnet: "255.255.255.0".to_string(),
        online: true,
        last_seen: tokio::time::unix_epoch().elapsed().unwrap().as_secs(),
        capabilities: 0,
    };
    
    let node_info_data = serde_json::to_vec(&node_info).unwrap();
    let resp_packet = Packet {
        magic: constants::MAGIC,
        version: PROTOCOL_VERSION,
        msg_type: MessageType::NodeInfo,
        flags: 0,
        length: node_info_data.len() as u16,
        checksum: calculate_checksum(&node_info_data),
        data: node_info_data,
    };
    
    let resp_packet_data = serde_json::to_vec(&resp_packet).unwrap();
    // 发送UDP数据包
}

/// 处理节点信息
async fn handle_node_info(
    packet: Packet,
    addr: SocketAddr,
    peers: Arc<RwLock<HashMap<String, Peer>>>
) {
    // 解析节点信息
    if let Ok(node_info) = serde_json::from_slice::<NodeInfo>(&packet.data) {
        let mut peers_guard = peers.write().await;
        peers_guard.insert(node_info.node_id.clone(), Peer {
            node_id: node_info.node_id.clone(),
            node_name: node_info.node_name.clone(),
            address: addr,
            virtual_ip: node_info.virtual_ip.clone(),
            public_key: node_info.public_key.clone(),
            status: NodeStatus::Online,
            last_seen: tokio::time::unix_epoch().elapsed().unwrap().as_secs(),
            capabilities: node_info.capabilities,
        });
    }
}

/// 处理心跳包
async fn handle_heartbeat(
    packet: Packet,
    addr: SocketAddr,
    peers: Arc<RwLock<HashMap<String, Peer>>>
) {
    // 解析心跳包
    if let Ok(heartbeat) = serde_json::from_slice::<Heartbeat>(&packet.data) {
        let mut peers_guard = peers.write().await;
        if let Some(peer) = peers_guard.get_mut(&heartbeat.node_id) {
            peer.last_seen = tokio::time::unix_epoch().elapsed().unwrap().as_secs();
            peer.status = NodeStatus::Online;
        }
    }
}

/// 处理数据转发
async fn handle_data_forward(
    packet: Packet,
    crypto: Arc<Mutex<CryptoContext>>
) {
    // 解析数据转发消息
    if let Ok(forward) = serde_json::from_slice::<DataForward>(&packet.data) {
        // 解密数据
        let mut crypto_guard = crypto.lock().await;
        if let Ok(plaintext) = crypto_guard.decrypt(&forward.data, &[]) {
            // 将数据转发到虚拟设备
            log::debug!("Forwarding data from {} to {} ({} bytes)", 
                        forward.source_node, forward.dest_node, plaintext.len());
            // 实际实现中，这里应该将数据发送到虚拟网卡
        }
    }
}

/// 发送心跳包
async fn send_heartbeat(
    udp_socket: &Arc<UdpSocket>,
    node_id: &str,
    peers: &Arc<RwLock<HashMap<String, Peer>>>
) {
    let heartbeat = Heartbeat {
        node_id: node_id.to_string(),
        timestamp: tokio::time::unix_epoch().elapsed().unwrap().as_secs(),
        load: 0.0, // 实际应获取系统负载
        uptime: 0, // 实际应获取系统运行时间
    };
    
    let heartbeat_data = serde_json::to_vec(&heartbeat).unwrap();
    let packet = Packet {
        magic: constants::MAGIC,
        version: PROTOCOL_VERSION,
        msg_type: MessageType::Heartbeat,
        flags: 0,
        length: heartbeat_data.len() as u16,
        checksum: calculate_checksum(&heartbeat_data),
        data: heartbeat_data,
    };
    
    let packet_data = serde_json::to_vec(&packet).unwrap();
    
    let peers_guard = peers.read().await;
    for peer in peers_guard.values() {
        if let Err(e) = udp_socket.send_to(&packet_data, peer.address) {
            log::warn!("Failed to send heartbeat to {}: {}", peer.node_id, e);
        }
    }
}

/// 清理超时节点
async fn cleanup_timeout_peers(peers: &Arc<RwLock<HashMap<String, Peer>>>) {
    let mut peers_guard = peers.write().await;
    let now = tokio::time::unix_epoch().elapsed().unwrap().as_secs();
    
    peers_guard.retain(|_, peer| {
        if now - peer.last_seen > constants::TIMEOUT {
            log::info!("Removing timeout peer: {}", peer.node_id);
            false
        } else {
            true
        }
    });
}
