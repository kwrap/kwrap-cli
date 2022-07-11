mod config;
mod http;
mod library;

use crate::{timestamp, Deserialize, Zeroize, ZeroizeOnDrop};
pub use config::*;
pub use http::HttpClient;
pub use library::LibraryClient;
use time_humanize::HumanTime;
use totp_rs::TOTP;

#[derive(Debug, Default, Deserialize, Clone, Zeroize, ZeroizeOnDrop)]
pub struct PasswordData {
    pub pin: Option<u32>,
    pub icon: Option<String>,
    pub name: Option<String>,
    pub user: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
    pub otp: Option<String>,
    pub links: Option<Vec<String>>,
    pub notes: Option<String>,
    pub custom: Option<Vec<CustomField>>,
    pub tags: Option<Vec<String>>,
    pub updated: Option<u32>,
    pub archive: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct CustomField {
    pub name: String,
    pub value: String,
    pub hidden: bool,
}

pub struct DisplayValue {
    pub key: String,
    pub value: String,
    pub copy_value: String,
}

impl DisplayValue {
    fn new<K: ToString, V: ToString, C: ToString>(key: K, value: V, copy_value: C) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            copy_value: copy_value.to_string(),
        }
    }
}

impl PasswordData {
    pub fn name(&self, show_pin: bool) -> String {
        let mut val = self
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| String::from("Untitled"));
        if self.pin.is_some() && show_pin {
            val.push_str(" \"");
        }
        val
    }

    pub fn user(&self) -> String {
        [&self.user, &self.email, &self.phone]
            .into_iter()
            .flatten()
            .next()
            .cloned()
            .unwrap_or_default()
    }

    pub fn to_display_value(&self) -> Vec<DisplayValue> {
        let mut values = vec![];
        if let Some(value) = &self.user {
            values.push(DisplayValue::new("User", value, value));
        }
        if let Some(value) = &self.email {
            values.push(DisplayValue::new("Email", value, value));
        }
        if let Some(value) = &self.phone {
            values.push(DisplayValue::new("Phone", value, value));
        }
        if let Some(value) = &self.password {
            values.push(DisplayValue::new("Password", "******", value));
        }
        if let Some(otp) = &self.otp {
            match TOTP::<Vec<u8>>::from_url(otp) {
                Ok(totp) => {
                    let t = timestamp();
                    let token = totp.generate(t);
                    values.push(DisplayValue::new(
                        format!("One-time password ({}s)", totp.step - t % totp.step),
                        &token,
                        &token,
                    ));
                }
                Err(_) => {
                    values.push(DisplayValue::new("One-time password", "------", ""));
                }
            }
        }
        if let Some(links) = &self.links {
            for value in links {
                values.push(DisplayValue::new("Link", value, value));
            }
        }
        if let Some(value) = &self.notes {
            values.push(DisplayValue::new("Notes", value, value));
        }
        if let Some(custom) = &self.custom {
            for CustomField {
                name,
                value,
                hidden,
            } in custom
            {
                if *hidden {
                    values.push(DisplayValue::new(name, "******", value));
                } else {
                    values.push(DisplayValue::new(name, value, value));
                }
            }
        }
        if let Some(tags) = &self.tags {
            let value = tags.join(", ");
            values.push(DisplayValue::new("Tags", &value, &value));
        }
        if let Some(value) = &self.updated {
            values.push(DisplayValue::new(
                "Update at",
                HumanTime::from_duration_since_timestamp(*value as u64),
                value,
            ));
        }
        values
    }
}
