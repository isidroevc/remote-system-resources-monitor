extern crate serde;

use std::fs;

use serde_derive::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub node_id: String,
    pub community_chain: String,
    pub monitor_server_url: String,
    pub refresh_time_millis: u64
}

pub fn load_config(filepath: &str) -> Configuration {
    let contents = fs::read_to_string(filepath)
        .expect("Something went wrong reading the config file");
    let result : Configuration = serde_json::from_str(&contents).unwrap();
    return result;
}