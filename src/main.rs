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

    // Window size is independent of map size (can be smaller than map)
    const WINDOW_WIDTH: f32 = 1024.0;
    const WINDOW_HEIGHT: f32 = 768.0;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ant Simulation".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
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
    use crate::marker::GRID_CELL_SIZE;

    // Map size in config is grid cells, convert to pixels
    let map_width_pixels = config.map_size.0 as f32 * GRID_CELL_SIZE;
    let map_height_pixels = config.map_size.1 as f32 * GRID_CELL_SIZE;

    // Set up 2D camera with zoom support
    // Start with a reasonable view size (e.g., 800x600 pixels visible area)
    const INITIAL_VIEW_HEIGHT: f32 = 600.0;
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode =
        bevy::render::camera::ScalingMode::FixedVertical(INITIAL_VIEW_HEIGHT);
    // Position camera at map center
    camera.transform = Transform::from_xyz(map_width_pixels / 2.0, map_height_pixels / 2.0, 0.0);

    commands.spawn(camera);
}
