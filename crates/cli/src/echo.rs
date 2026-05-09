//! printing utilities

use owo_colors::OwoColorize;

pub fn pair<D>(key: &str, value: D)
where
    D: std::fmt::Display,
{
    println!("{}: {value}", key.blue().bold());
}

pub fn opt<D>(key: &str, value: Option<D>)
where
    D: std::fmt::Display,
{
    if let Some(value) = value {
        pair(key, value);
    }
}

pub fn list<D>(key: &str, values: &[D])
where
    D: std::fmt::Display,
{
    if !values.is_empty() {
        println!("{}:", key.blue().bold());
        for value in values {
            println!("  - {value}");
        }
    }
}
