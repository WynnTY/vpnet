/*!
VPNet虚拟设备模块

管理虚拟网络接口，包括：
- 虚拟网卡的创建和配置
- 数据包的发送和接收
- 设备状态管理
- 跨平台支持
*/

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::net::Ipv4Addr;
use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket};
use pnet::packet::ipv4::{Ipv4Packet, MutableIpv4Packet};
use pnet::packet::tcp::{TcpPacket, MutableTcpPacket};
use pnet::packet::udp::{UdpPacket, MutableUdpPacket};
use pnet::packet::{MutablePacket, Packet};
use std::collections::HashMap;
use std::time::Duration;

/// 虚拟设备配置
pub struct VirtualDeviceConfig {
    pub name: String,
    pub ip: Ipv4Addr,
    pub subnet: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub mtu: u32,
    pub mac: Option<[u8; 6]>,
}

/// 虚拟设备
pub struct VirtualDevice {
    config: VirtualDeviceConfig,
    interface: Option<NetworkInterface>,
    send_channel: Option<Arc<Mutex<dyn datalink::DataLinkSender>>>,
    recv_channel: Option<Arc<Mutex<dyn datalink::DataLinkReceiver>>>,
    packet_tx: mpsc::Sender<Vec<u8>>,
    packet_rx: mpsc::Receiver<Vec<u8>>,
    device_id: String,
    is_running: bool,
}

/// 设备状态
pub enum DeviceStatus {
    Up,
    Down,
    Error(String),
}

impl VirtualDevice {
    /// 创建新的虚拟设备
    pub fn new(config: VirtualDeviceConfig, device_id: String) -> Result<Self, &'static str> {
        let (packet_tx, packet_rx) = mpsc::channel(1024);
        
        Ok(Self {
            config,
            interface: None,
            send_channel: None,
            recv_channel: None,
            packet_tx,
            packet_rx,
            device_id,
            is_running: false,
        })
    }
    
    /// 启动虚拟设备
    pub async fn start(&mut self) -> Result<(), &'static str> {
        // 在实际实现中，这里应该创建虚拟网卡
        // 例如，在Linux上使用tun/tap设备，在Windows上使用Wintun或OpenVPN虚拟网卡
        
        // 目前是模拟实现，实际需要根据不同平台调用相应的API
        self.is_running = true;
        
        // 查找或创建虚拟网卡
        let interfaces = datalink::interfaces();
        let interface = interfaces.into_iter()
            .find(|iface| iface.name == self.config.name)
            .or_else(|| {
                // 实际实现中，这里应该创建新的虚拟网卡
                log::warn!("Virtual interface {} not found, creating a new one...", self.config.name);
                None
            });
        
        if let Some(iface) = interface {
            self.interface = Some(iface);
            
            // 配置虚拟网卡
            self.configure_interface().await?;
            
            // 启动数据传输任务
            self.start_data_transfer().await;
        }
        
        Ok(())
    }
    
    /// 配置虚拟设备
    async fn configure_interface(&mut self) -> Result<(), &'static str> {
        // 实际实现中，这里应该配置虚拟网卡的IP、子网掩码、网关等
        log::info!("Configuring interface {} with IP: {}/{}", 
                  self.config.name, self.config.ip, self.config.subnet);
        
        Ok(())
    }
    
    /// 启动数据传输任务
    async fn start_data_transfer(&mut self) {
        // 启动接收任务
        let recv_channel = self.recv_channel.clone();
        let packet_tx = self.packet_tx.clone();
        
        tokio::spawn(async move {
            if let Some(recv) = recv_channel {
                let mut buf = [0u8; 1500];
                loop {
                    match recv.lock().await.next() {
                        Ok(packet) => {
                            if let Err(e) = packet_tx.send(packet.to_vec()).await {
                                log::error!("Failed to send packet: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to receive packet: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }
    
    /// 从虚拟设备接收数据包
    pub async fn recv(&mut self) -> Result<Vec<u8>, &'static str> {
        self.packet_rx.recv().await
            .map_err(|_| "Failed to receive packet")
    }
    
    /// 发送数据包到虚拟设备
    pub async fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.is_running {
            return Err("Device is not running");
        }
        
        // 实际实现中，这里应该将数据发送到虚拟网卡
        log::debug!("Sending packet to virtual device {} ({} bytes)", 
                  self.config.name, data.len());
        
        if let Some(send) = &self.send_channel {
            send.lock().await.send_to(data, None)
                .map_err(|_| "Failed to send packet")?;
        }
        
        Ok(())
    }
    
    /// 停止虚拟设备
    pub async fn stop(&mut self) -> Result<(), &'static str> {
        self.is_running = false;
        // 实际实现中，这里应该关闭虚拟网卡
        log::info!("Stopping virtual device {}", self.config.name);
        Ok(())
    }
    
    /// 获取设备状态
    pub async fn get_status(&self) -> DeviceStatus {
        if self.is_running {
            DeviceStatus::Up
        } else {
            DeviceStatus::Down
        }
    }
    
    /// 获取设备配置
    pub async fn get_config(&self) -> &VirtualDeviceConfig {
        &self.config
    }
    
    /// 获取设备ID
    pub async fn get_device_id(&self) -> &str {
        &self.device_id
    }
    
    /// 重置设备
    pub async fn reset(&mut self) -> Result<(), &'static str> {
        self.stop().await?;
        self.start().await
    }
    
    /// 更新设备配置
    pub async fn update_config(&mut self, new_config: VirtualDeviceConfig) -> Result<(), &'static str> {
        self.stop().await?;
        self.config = new_config;
        self.start().await
    }
}

/// 设备管理器
pub struct DeviceManager {
    devices: Arc<Mutex<HashMap<String, Arc<Mutex<VirtualDevice>>>>},
    device_counter: u32,
}

impl DeviceManager {
    /// 创建新的设备管理器
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            device_counter: 0,
        }
    }
    
    /// 创建虚拟设备
    pub async fn create_device(
        &mut self, 
        config: VirtualDeviceConfig
    ) -> Result<String, &'static str> {
        let device_id = format!("device_{}", self.device_counter);
        self.device_counter += 1;
        
        let device = VirtualDevice::new(config, device_id.clone())?;
        let device = Arc::new(Mutex::new(device));
        
        let mut devices = self.devices.lock().await;
        devices.insert(device_id.clone(), device);
        
        Ok(device_id)
    }
    
    /// 获取虚拟设备
    pub async fn get_device(
        &self, 
        device_id: &str
    ) -> Result<Arc<Mutex<VirtualDevice>>, &'static str> {
        let devices = self.devices.lock().await;
        devices.get(device_id)
            .cloned()
            .ok_or("Device not found")
    }
    
    /// 启动虚拟设备
    pub async fn start_device(
        &self, 
        device_id: &str
    ) -> Result<(), &'static str> {
        let device = self.get_device(device_id).await?;
        let mut device = device.lock().await;
        device.start().await
    }
    
    /// 停止虚拟设备
    pub async fn stop_device(
        &self, 
        device_id: &str
    ) -> Result<(), &'static str> {
        let device = self.get_device(device_id).await?;
        let mut device = device.lock().await;
        device.stop().await
    }
    
    /// 删除虚拟设备
    pub async fn delete_device(
        &self, 
        device_id: &str
    ) -> Result<(), &'static str> {
        let mut devices = self.devices.lock().await;
        devices.remove(device_id);
        Ok(())
    }
    
    /// 获取所有设备
    pub async fn get_all_devices(
        &self
    ) -> HashMap<String, Arc<Mutex<VirtualDevice>>> {
        let devices = self.devices.lock().await;
        devices.clone()
    }
    
    /// 获取设备状态
    pub async fn get_device_status(
        &self, 
        device_id: &str
    ) -> Result<DeviceStatus, &'static str> {
        let device = self.get_device(device_id).await?;
        let device = device.lock().await;
        Ok(device.get_status().await)
    }
}

/// 默认虚拟设备配置
pub fn default_config(name: String, ip: Ipv4Addr) -> VirtualDeviceConfig {
    VirtualDeviceConfig {
        name,
        ip,
        subnet: Ipv4Addr::new(255, 255, 255, 0),
        gateway: Ipv4Addr::new(10, 0, 0, 1),
        mtu: 1420,
        mac: None,
    }
}

/// 生成随机MAC地址
pub fn generate_random_mac() -> [u8; 6] {
    let mut mac = [0u8; 6];
    mac[0] = 0x02; // 本地管理地址
    for i in 1..6 {
        mac[i] = rand::random();
    }
    mac
}

/// 解析以太网数据包
pub fn parse_ethernet_packet(data: &[u8]) -> Option<EthernetPacket> {
    EthernetPacket::new(data)
}

/// 解析IPv4数据包
pub fn parse_ipv4_packet(data: &[u8]) -> Option<Ipv4Packet> {
    Ipv4Packet::new(data)
}

/// 解析TCP数据包
pub fn parse_tcp_packet(data: &[u8]) -> Option<TcpPacket> {
    TcpPacket::new(data)
}

/// 解析UDP数据包
pub fn parse_udp_packet(data: &[u8]) -> Option<UdpPacket> {
    UdpPacket::new(data)
}
