use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub source_bat: String,
    pub args: String,
}

pub mod apex;
pub mod autostart_connect;
pub mod engine_process;
pub mod hosts;
pub mod preset_loader;
pub mod probe;
pub mod strategy;
pub mod strategy_loader;
pub mod strategy_runner;
pub mod updater;
pub mod zapret2_runner;
pub mod zapret2_updater;
pub mod zapret_backend;
pub mod zapret_config;
