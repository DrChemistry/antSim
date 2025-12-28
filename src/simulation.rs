use crate::ant::{follow_markers, keep_ants_in_bounds, move_ants};
use crate::base::{check_base_collision, spawn_ants, SpawnTimer};
use crate::config::Config;
use crate::food::check_food_collision;
use crate::marker::{spawn_markers, update_marker_visuals};
use bevy::prelude::*;

pub fn setup_simulation(mut commands: Commands, config: Res<Config>) {
    // Spawn base
    let (base_x, base_y) = (config.base_location.0 as f32, config.base_location.1 as f32);
    commands.spawn((
        crate::base::Base,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.3, 0.8),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(base_x, base_y, 0.0)),
            ..default()
        },
    ));

    // Spawn food sources
    for (food_x, food_y) in &config.food_locations {
        commands.spawn((
            crate::food::FoodSource,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.9, 0.7, 0.1),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    *food_x as f32,
                    *food_y as f32,
                    0.0,
                )),
                ..default()
            },
        ));
    }

    // Spawn initial ants at the base
    for _ in 0..config.initial_ant_count {
        commands.spawn((
            crate::ant::Ant::new(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(6.0, 6.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(base_x, base_y, 0.0)),
                ..default()
            },
        ));
    }

    // Initialize spawn timer
    commands.insert_resource(SpawnTimer {
        timer: Timer::from_seconds(config.spawn_rate, TimerMode::Repeating),
    });
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_simulation).add_systems(
            Update,
            (
                spawn_ants,
                follow_markers,
                move_ants,
                keep_ants_in_bounds,
                spawn_markers,
                update_marker_visuals,
                check_food_collision,
                check_base_collision,
            ),
        );
    }
}
