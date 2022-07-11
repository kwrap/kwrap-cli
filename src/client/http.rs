use crate::*;
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::redirect::Policy;
use reqwest::{Result, StatusCode};
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct PreloginData {
    asalt: String,
    iterations: u32,
}

#[derive(Debug, Deserialize)]
struct ESalt {
    esalt: String,
}

#[derive(Debug, Deserialize, Zeroize, ZeroizeOnDrop)]
struct EncryptedPassword {
    // pid: String,
    data: String,
}

#[derive(Debug, Default, Zeroize, ZeroizeOnDrop)]
struct Auth {
    user: String,
    password: String,
}

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    config: HttpConfig,
    auth: Auth,
    key: [u8; 32],
}

impl Drop for HttpClient {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl HttpClient {
    pub fn new(mut config: HttpConfig) -> Self {
        if config.server.ends_with('/') {
            config.server.pop();
        }
        let client = ClientBuilder::new()
            .redirect(Policy::default())
            .brotli(true)
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            client,
            config,
            auth: Auth::default(),
            key: [0; 32],
        }
    }

    fn to_data<T: DeserializeOwned>(rst: Result<Response>) -> T {
        let res = rst
            .map_err(|err| err.to_string())
            .unwrap_exit("HTTP Request");
        if res.status() != StatusCode::OK {
            exit!("{}\nBody: {}", res.status(), res.text().unwrap_or_default());
        }
        res.json::<T>().unwrap_exit("Failed to parse response")
    }

    pub fn login(&mut self) {
        let user = sha256(&self.config.user);
        let rst = self
            .client
            .get(format!("{}/user/prelogin/{}", self.config.server, user))
            .send();
        let prelogin = Self::to_data::<PreloginData>(rst);
        let auth = Auth {
            user,
            password: base64::encode(pbkdf2(
                &self.config.password,
                decode_base64(prelogin.asalt),
                prelogin.iterations,
            )),
        };
        let rst = self
            .client
            .get(format!("{}/user/esalt", &self.config.server))
            .basic_auth(&auth.user, Some(&auth.password))
            .send();

        let esalt = Self::to_data::<ESalt>(rst).esalt;
        self.key = pbkdf2(
            &self.config.password,
            decode_base64(esalt),
            prelogin.iterations,
        );
        self.auth = auth;
    }

    // The login fn must be called first
    pub fn passwords(self) -> Vec<PasswordData> {
        let cipher = Aes256Gcm::new(self.key);
        let rst = self
            .client
            .get(format!("{}/passwords", self.config.server))
            .basic_auth(&self.auth.user, Some(&self.auth.password))
            .send();
        Self::to_data::<Vec<EncryptedPassword>>(rst)
            .into_iter()
            .map(|item| {
                let mut data = decode_base64(&item.data);
                cipher.decrypt_to(&mut data)
            })
            .collect()
    }
}
