use crate::*;
use std::fs::File;

pub struct LibraryClient {
    pub passwords: Vec<PasswordData>,
}

impl LibraryClient {
    pub fn new(config: &LibraryConfig) -> Self {
        let f = File::open(&config.path).unwrap_exit(format!("Open file failed {}", config.path));
        let mut kwrap = KwrapFile::parse(f).unwrap_exit("Parse Kwrap file failed");
        let key = pbkdf2(&config.password, kwrap.salt, kwrap.iterations);
        Self {
            passwords: Aes256Gcm::new(key).decrypt_to(&mut kwrap.data),
        }
    }
}
