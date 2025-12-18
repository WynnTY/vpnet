/*!
VPNet - A cross-platform, high-performance, decentralized virtual LAN platform

This library provides the core functionality for VPNet, including:
- Network protocol implementation
- Encryption and security
- Peer-to-peer communication
- Virtual network interface management
*/

pub mod crypto;
pub mod network;
pub mod protocol;
pub mod utils;
pub mod virtual_device;

pub use protocol::*;
pub use network::*;
pub use crypto::*;
pub use virtual_device::*;

/// VPNet version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default VPNet port
pub const DEFAULT_PORT: u16 = 51820;

/// Default MTU for virtual device
pub const DEFAULT_MTU: u32 = 1420;

/// Maximum packet size
pub const MAX_PACKET_SIZE: usize = 1500;
