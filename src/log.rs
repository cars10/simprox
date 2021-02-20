use chrono::{offset::Local, SecondsFormat};

pub fn log<T>(text: T)
where
    T: Into<String> + std::fmt::Display,
{
    let now = Local::now();
    let formatted_date = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    println!("[{}] -- {}", formatted_date, text)
}
