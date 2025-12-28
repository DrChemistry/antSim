use bevy::prelude::*;

mod ant;
mod base;
mod config;
mod food;
mod gui;
mod logging;
mod marker;
mod simulation;

use config::Config;
use gui::DebugGUIPlugin;
use logging::LoggingPlugin;
use simulation::SimulationPlugin;

fn main() {
    // Load configuration
    let config = Config::load().expect("Failed to load config.json");
    let (map_width, map_height) = (config.map_size.0, config.map_size.1);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ant Simulation".into(),
                resolution: (map_width as f32, map_height as f32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(config)
        .add_plugins(SimulationPlugin)
        .add_plugins(DebugGUIPlugin)
        .add_plugins(LoggingPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands, config: Res<Config>) {
    let (map_width, map_height) = (config.map_size.0 as f32, config.map_size.1 as f32);

    // Set up 2D camera with origin at bottom-left
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(map_width / 2.0, map_height / 2.0, 0.0),
        ..default()
    });
}
