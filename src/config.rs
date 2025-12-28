use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Config {
    pub map_size: (u32, u32),
    pub base_location: (u32, u32),
    pub food_locations: Vec<(u32, u32)>,
    pub spawn_rate: f32,
    pub marker_spawn_interval: f32,
    pub marker_lifetime: f32,
    pub initial_ant_count: u32,
    pub food_quantity: u32,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string("config.json")?;
        let config: Config = serde_json::from_str(&config_str)?;
        Ok(config)
    }
}
