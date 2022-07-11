mod client;
mod crypto;
mod kwrap;
mod ui;
mod utils;

use ace::App;
pub use client::*;
pub use crypto::*;
use home_config::HomeConfig;
pub use kwrap::KwrapFile;
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use ui::start;
pub use utils::*;
pub use zeroize::{Zeroize, ZeroizeOnDrop};

fn main() {
    let hc = HomeConfig::new(env!("CARGO_PKG_NAME"), "config.json");

    {
        let app = App::new()
            .config(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .cmd("logout", "Clear login info")
            .cmd("info", "Print login info")
            .cmd("help", "Print help information")
            .cmd("version", "Print version information");

        if let Some(cmd) = app.command() {
            match cmd.as_str() {
                "logout" => {
                    hc.delete().unwrap_exit("Delete config file");
                }
                "info" => {
                    if hc.path().is_file() {
                        println!("Config: {}", hc.path().display());
                        println!("{}", hc.read_to_string().unwrap_or_default());
                    } else {
                        exit!("Config file does not exist");
                    }
                }
                "help" => {
                    app.print_help();
                }
                "version" => {
                    app.print_version();
                }
                _ => {
                    app.print_error_try("help");
                    std::process::exit(1);
                }
            }
            return;
        }
    }

    let config = hc
        .json::<Config>()
        .map(|mut config| {
            config.read_password();
            config
        })
        .unwrap_or_else(|_| Config::from_stdin());

    let list = match &config {
        Config::Http(c) => {
            let mut client = HttpClient::new(c.clone());
            client.login();
            client.passwords()
        }
        Config::Library(c) => {
            let client = LibraryClient::new(c);
            client.passwords
        }
    };

    let _ = hc.save_json(&config);

    drop(config);

    ui::start(list).unwrap_exit("UI Error")
}
