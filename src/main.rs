use bevy::prelude::*;

mod ant;
mod base;
mod chart_data;
mod chart_generator;
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
    let (map_width, map_height) = (config.map_size.0 as f32, config.map_size.1 as f32);

    // Calculate window size with padding around map (100-150 pixels on each side)
    const WINDOW_PADDING: f32 = 120.0;
    let window_width = map_width + (WINDOW_PADDING * 2.0);
    let window_height = map_height + (WINDOW_PADDING * 2.0);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ant Simulation".into(),
                resolution: (window_width, window_height).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(config)
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3))) // Darker grey for out-of-bounds
        .add_plugins(SimulationPlugin)
        .add_plugins(DebugGUIPlugin)
        .add_plugins(LoggingPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands, config: Res<Config>) {
    let (map_width, map_height) = (config.map_size.0 as f32, config.map_size.1 as f32);

    // Set up 2D camera with fixed projection matching map size
    // This ensures the map doesn't scale when window is resized
    let mut camera = Camera2dBundle::default();
    // Set fixed area to match map dimensions
    // The area defines what the camera sees in world units, centered at origin
    camera.projection.area = Rect {
        min: Vec2::new(-map_width / 2.0, -map_height / 2.0),
        max: Vec2::new(map_width / 2.0, map_height / 2.0),
    };
    // Position camera at map center
    camera.transform = Transform::from_xyz(map_width / 2.0, map_height / 2.0, 0.0);

    commands.spawn(camera);
}
