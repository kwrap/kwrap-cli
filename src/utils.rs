use arboard::Clipboard;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;

#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {
        {
            eprint!("{}", "Error: ");
            eprintln!($($arg)*);
            std::process::exit(1)
        }
    };
}

pub fn copy_text<S: ToString>(text: S) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|err| err.to_string())?;
    clipboard
        .set_text(text.to_string())
        .map_err(|err| err.to_string())
}

pub fn time_now() -> String {
    let now = OffsetDateTime::now_local();
    now.format("%H:%M:%S")
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_exit("Get system time")
        .as_secs()
}

pub fn decode_base64<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
    base64::decode(input.as_ref()).unwrap_exit("base64 decoding failed")
}

pub fn trim_str<S: AsRef<str>>(s: S) -> String {
    let s = s.as_ref();
    if s.len() > 32 {
        format!("...{}", &s[(s.len() - 32)..])
    } else {
        s.to_string()
    }
}

// Convert path to absolute path
pub fn absolute_path(path: String) -> String {
    let p: &Path = path.as_ref();
    if p.is_absolute() {
        return path;
    }
    let cur = std::env::current_dir().unwrap_exit("current_dir");
    cur.join(p).display().to_string()
}

pub trait ThrowError<T, E, M> {
    fn unwrap_exit(self, msg: M) -> T;
}

impl<T, E, M> ThrowError<T, E, M> for Result<T, E>
where
    E: Debug,
    M: Display,
{
    fn unwrap_exit(self, msg: M) -> T {
        self.unwrap_or_else(|err| exit!("{}\n{:#?}", msg, err))
    }
}
