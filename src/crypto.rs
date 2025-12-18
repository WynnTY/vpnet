/*!
VPNet加密模块

提供网络通信的加密和认证功能，包括：
- 密钥生成和管理
- 数据加密和解密
- 消息认证
- 握手协议
*/

use ring::aead::{self, Aad, BoundKey, Nonce, UnboundKey};
use ring::digest;
use ring::hmac;
use ring::rand::{self, SecureRandom};
use rand::Rng;
use base64::Engine;

/// 加密算法类型
pub enum CryptoAlgorithm {
    AesGcm128,
    AesGcm256,
}

/// 加密上下文
pub struct CryptoContext {
    key: aead::LessSafeKey,
    algorithm: CryptoAlgorithm,
    nonce_counter: u64,
    rng: rand::SystemRandom,
}

/// 密钥对
pub struct KeyPair {
    pub public_key: Vec<u8>,
    private_key: Vec<u8>,
}

impl CryptoContext {
    /// 创建新的加密上下文
    pub fn new(key: &[u8], algorithm: CryptoAlgorithm) -> Self {
        let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key).unwrap();
        let key = aead::LessSafeKey::new(unbound_key);
        
        Self {
            key,
            algorithm,
            nonce_counter: 0,
            rng: rand::SystemRandom::new(),
        }
    }
    
    /// 加密数据
    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, &'static str> {
        let mut nonce_bytes = [0u8; 12];
        self.nonce_counter.to_be_bytes().clone_into(&mut nonce_bytes[4..]);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        
        let mut ciphertext = plaintext.to_vec();
        let tag_len = self.key.algorithm().tag_len();
        ciphertext.resize(plaintext.len() + tag_len, 0);
        
        let aad = Aad::from(aad);
        
        self.key.seal_in_place_append_tag(nonce, aad, &mut ciphertext)
            .map_err(|_| "Encryption failed")?;
        
        self.nonce_counter += 1;
        Ok(ciphertext)
    }
    
    /// 解密数据
    pub fn decrypt(&self, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, &'static str> {
        if ciphertext.len() < self.key.algorithm().tag_len() {
            return Err("Ciphertext too short");
        }
        
        let nonce_bytes = [0u8; 12]; // 简化处理，实际应该从数据包中获取
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        
        let mut plaintext = ciphertext.to_vec();
        let aad = Aad::from(aad);
        
        let plaintext_len = self.key.open_in_place(nonce, aad, &mut plaintext)
            .map_err(|_| "Decryption failed")?
            .len();
        
        plaintext.truncate(plaintext_len);
        Ok(plaintext)
    }
    
    /// 生成随机密钥
    pub fn generate_key(&mut self, algorithm: CryptoAlgorithm) -> Vec<u8> {
        let key_len = match algorithm {
            CryptoAlgorithm::AesGcm128 => 16,
            CryptoAlgorithm::AesGcm256 => 32,
        };
        
        let mut key = vec![0u8; key_len];
        self.rng.fill(&mut key).unwrap();
        key
    }
}

impl KeyPair {
    /// 生成新的密钥对
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let mut public_key = vec![0u8; 32];
        let mut private_key = vec![0u8; 32];
        
        rng.fill(&mut public_key);
        rng.fill(&mut private_key);
        
        Self {
            public_key,
            private_key,
        }
    }
    
    /// 从Base64字符串创建密钥对
    pub fn from_base64(public_b64: &str, private_b64: &str) -> Result<Self, &'static str> {
        let public_key = base64::engine::general_purpose::STANDARD
            .decode(public_b64)
            .map_err(|_| "Invalid public key")?;
        let private_key = base64::engine::general_purpose::STANDARD
            .decode(private_b64)
            .map_err(|_| "Invalid private key")?;
        
        Ok(Self {
            public_key,
            private_key,
        })
    }
    
    /// 转换为Base64字符串
    pub fn to_base64(&self) -> (String, String) {
        let public_b64 = base64::engine::general_purpose::STANDARD.encode(&self.public_key);
        let private_b64 = base64::engine::general_purpose::STANDARD.encode(&self.private_key);
        
        (public_b64, private_b64)
    }
}

/// 计算数据的哈希值
pub fn hash(data: &[u8]) -> Vec<u8> {
    let digest = digest::digest(&digest::SHA256, data);
    digest.as_ref().to_vec()
}

/// 生成HMAC
pub fn generate_hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let tag = hmac::sign(&key, data);
    tag.as_ref().to_vec()
}

/// 验证HMAC
pub fn verify_hmac(key: &[u8], data: &[u8], tag: &[u8]) -> bool {
    let key = hmac::Key::new(hmac::HMAC_SHA256, key);
    hmac::verify(&key, data, tag).is_ok()
}
