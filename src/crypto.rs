use crate::*;
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, UnboundKey, AES_256_GCM, NONCE_LEN,
};
use ring::{
    digest::{digest, SHA256, SHA256_OUTPUT_LEN},
    error::Unspecified,
    pbkdf2::{derive, PBKDF2_HMAC_SHA256},
};
use std::num::NonZeroU32;

pub fn sha256(content: &str) -> String {
    let user = digest(&SHA256, content.as_bytes());
    hex::encode(user.as_ref())
}

pub fn pbkdf2<P: AsRef<[u8]>, S: AsRef<[u8]>>(
    password: P,
    salt: S,
    iterations: u32,
) -> [u8; SHA256_OUTPUT_LEN] {
    assert_ne!(iterations, 0);
    let mut out = [0; 32];
    derive(
        PBKDF2_HMAC_SHA256,
        NonZeroU32::new(iterations).unwrap(),
        salt.as_ref(),
        password.as_ref(),
        &mut out,
    );
    out
}

struct StaticNonce([u8; NONCE_LEN]);

impl StaticNonce {
    fn new(bytes: &[u8]) -> Result<Self, Unspecified> {
        match bytes.try_into() {
            Ok(d) => Ok(Self(d)),
            Err(_) => Err(Unspecified),
        }
    }
}

impl NonceSequence for StaticNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Ok(Nonce::assume_unique_for_key(self.0))
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Aes256Gcm {
    key: [u8; 32],
}

impl Aes256Gcm {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn decrypt<'a>(&self, encrypted: &'a mut [u8]) -> &'a mut [u8] {
        let key = UnboundKey::new(&AES_256_GCM, &self.key).unwrap_exit("AES-GCM Key");
        let nonce = StaticNonce::new(&encrypted[..NONCE_LEN]).unwrap_exit("AES-GCM Nonce");
        let mut opening_key = OpeningKey::new(key, nonce);
        let data = opening_key
            .open_in_place(Aad::empty(), &mut encrypted[NONCE_LEN..])
            .unwrap_exit("Password error");
        data
    }

    pub fn decrypt_to<T: DeserializeOwned>(&self, data: &mut [u8]) -> T {
        let data = self.decrypt(data);
        let json = serde_json::from_slice::<T>(data).unwrap_exit("Failed to parse JSON");
        data.zeroize();
        json
    }
}
