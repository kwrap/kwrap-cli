use crate::{absolute_path, trim_str, Deserialize, Serialize, ThrowError, Zeroize, ZeroizeOnDrop};
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use reqwest::Url;

#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "lowercase")]
pub enum Config {
    Http(HttpConfig),
    Library(LibraryConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HttpConfig {
    pub server: String,
    pub user: String,
    #[serde(skip)]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct LibraryConfig {
    pub path: String,
    #[serde(skip)]
    pub password: String,
}

impl Config {
    pub fn from_stdin() -> Config {
        let types = vec!["Use Kwrap Server", "Use Kwrap Library"];
        let selected = Select::with_theme(&ColorfulTheme::default())
            .items(&types)
            .default(0)
            .interact()
            .unwrap_exit("Read use type");

        let mut config = match selected {
            0 => Self::Http(HttpConfig {
                server: Self::read_server(),
                user: Self::read_username(),
                password: String::new(),
            }),
            1 => Self::Library(LibraryConfig {
                path: Self::read_path(),
                password: String::new(),
            }),
            _ => unimplemented!(),
        };
        config.read_password();
        config
    }

    fn tips(&self) -> String {
        match self {
            Self::Http(c) => trim_str(format!("{} -> {}", c.server, c.user)),
            Self::Library(c) => trim_str(&c.path),
        }
    }

    pub fn read_password(&mut self) {
        let tips = self.tips();
        let p = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Password ({})", tips))
            .interact()
            .unwrap_exit("Read password");
        match self {
            Self::Http(c) => c.password = p,
            Self::Library(c) => c.password = p,
        }
    }

    fn read_server() -> String {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Server")
            .validate_with(|input: &String| -> Result<(), String> {
                match Url::parse(input) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(format!("Invalid URL: {}", input)),
                }
            })
            .interact()
            .unwrap_exit("Read server")
    }

    fn read_username() -> String {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Username")
            .interact()
            .unwrap_exit("Read username")
    }

    fn read_path() -> String {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Kwrap Library Path")
            .interact()
            .map(absolute_path)
            .unwrap_exit("Read path")
    }
}
