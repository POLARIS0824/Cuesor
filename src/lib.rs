pub mod agent;
pub mod cli;
pub mod model;
pub mod platform;
pub mod session;
pub mod settings;

pub fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization cannot fail")
}
