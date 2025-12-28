use crate::ant::{follow_markers, keep_ants_in_bounds, move_ants};
use crate::base::{check_base_collision, spawn_ants, SpawnTimer};
use crate::config::Config;
use crate::food::check_food_collision;
use crate::marker::{spawn_markers, update_marker_visuals, GridMap, GRID_CELL_SIZE};
use bevy::prelude::*;

pub fn setup_simulation(mut commands: Commands, config: Res<Config>) {
    // Map size in config is grid cells, convert to pixels
    let map_width_pixels = config.map_size.0 as f32 * GRID_CELL_SIZE;
    let map_height_pixels = config.map_size.1 as f32 * GRID_CELL_SIZE;

    // Spawn map background (lighter grey area representing the simulation playground)
    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.9, 0.9, 0.9), // Lighter grey for map area
            custom_size: Some(Vec2::new(map_width_pixels, map_height_pixels)),
            ..default()
        },
        transform: Transform::from_xyz(map_width_pixels / 2.0, map_height_pixels / 2.0, -1.0), // Behind all entities
        ..default()
    },));

    // Spawn base (2x2 grid cells = 64x64 pixels)
    // base_location in config is the grid cell coordinate of the bottom-left corner
    let base_size = 2.0 * GRID_CELL_SIZE; // 64x64 pixels
                                          // base_location is now grid cell coordinates
    let base_cell = (config.base_location.0 as i32, config.base_location.1 as i32);
    // Calculate bottom-left corner of the cell in world coordinates
    // Convert grid coordinates to world coordinates by multiplying by GRID_CELL_SIZE
    let base_bottom_left_world = Vec2::new(
        base_cell.0 as f32 * GRID_CELL_SIZE,
        base_cell.1 as f32 * GRID_CELL_SIZE,
    );
    // Center of 2x2 grid is at bottom-left + 1 cell in both directions
    let base_center = base_bottom_left_world + Vec2::new(GRID_CELL_SIZE, GRID_CELL_SIZE);

    commands.spawn((
        crate::base::Base,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.3, 0.8),
                custom_size: Some(Vec2::new(base_size, base_size)),
                ..default()
            },
            transform: Transform::from_translation(base_center.extend(0.0)),
            ..default()
        },
    ));

    // Spawn ants at base center
    let base_spawn_pos = base_center;

    // Spawn food sources
    // food_locations in config are grid cell coordinates
    use crate::marker::grid_to_world;
    for (food_cell_x, food_cell_y) in &config.food_locations {
        let food_cell = (*food_cell_x as i32, *food_cell_y as i32);
        let food_world_pos = grid_to_world(food_cell);
        commands.spawn((
            crate::food::FoodSource,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.9, 0.7, 0.1),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_translation(food_world_pos.extend(0.0)),
                ..default()
            },
        ));
    }

    // Spawn initial ants at the base center
    for _ in 0..config.initial_ant_count {
        commands.spawn((
            crate::ant::Ant::new(),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(6.0, 6.0)),
                    ..default()
                },
                transform: Transform::from_translation(base_spawn_pos.extend(0.0)),
                ..default()
            },
        ));
    }

    // Initialize spawn timer
    commands.insert_resource(SpawnTimer {
        timer: Timer::from_seconds(config.spawn_rate, TimerMode::Repeating),
    });

    // Initialize grid map
    commands.insert_resource(GridMap::default());
}

pub fn render_grid(
    mut commands: Commands,
    config: Res<Config>,
    existing_grid: Query<Entity, With<GridLine>>,
) {
    // Clear existing grid lines
    for entity in existing_grid.iter() {
        commands.entity(entity).despawn();
    }

    // Map size in config is grid cells, convert to pixels
    let map_width_pixels = config.map_size.0 as f32 * GRID_CELL_SIZE;
    let map_height_pixels = config.map_size.1 as f32 * GRID_CELL_SIZE;
    let grid_color = Color::rgba(0.7, 0.7, 0.7, 0.3); // Light grey, semi-transparent
    const LINE_WIDTH: f32 = 1.0;

    // Draw vertical lines
    let num_vertical = config.map_size.0 as i32;
    for i in 0..=num_vertical {
        let x = i as f32 * GRID_CELL_SIZE;
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: grid_color,
                    custom_size: Some(Vec2::new(LINE_WIDTH, map_height_pixels)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x, map_height_pixels / 2.0, -0.5)),
                ..default()
            },
            GridLine,
        ));
    }

    // Draw horizontal lines
    let num_horizontal = config.map_size.1 as i32;
    for i in 0..=num_horizontal {
        let y = i as f32 * GRID_CELL_SIZE;
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: grid_color,
                    custom_size: Some(Vec2::new(map_width_pixels, LINE_WIDTH)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(map_width_pixels / 2.0, y, -0.5)),
                ..default()
            },
            GridLine,
        ));
    }
}

#[derive(Component)]
pub struct GridLine;

pub fn camera_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<GridLine>)>,
    time: Res<Time>,
) {
    const CAMERA_SPEED: f32 = 250.0; // pixels per second

    if let Ok(mut transform) = camera_query.get_single_mut() {
        let mut movement = Vec2::ZERO;

        // zqsd keys (French keyboard layout: z=up, q=left, s=down, d=right)
        // Also support wasd keys (English keyboard layout)
        // In Bevy 0.12, KeyCode variants use format Key<Letter>
        if keyboard_input.pressed(KeyCode::Z) || keyboard_input.pressed(KeyCode::W) {
            movement.y += 2.0;
        }
        if keyboard_input.pressed(KeyCode::Q) || keyboard_input.pressed(KeyCode::A) {
            movement.x -= 2.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            movement.y -= 2.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            movement.x += 2.0;
        }

        // Normalize diagonal movement
        if movement.length() > 0.0 {
            movement = movement.normalize();
        }

        // Apply movement
        let delta = movement * CAMERA_SPEED * time.delta_seconds();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;

        // Optional: Clamp camera to map bounds (or allow free movement)
        // For now, allow free movement
    }
}

pub fn camera_zoom(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    const ZOOM_SPEED: f32 = 0.1;
    const MIN_SCALE: f32 = 0.5;
    const MAX_SCALE: f32 = 3.0;

    let mut total_scroll = 0.0;
    for event in mouse_wheel_events.read() {
        // MouseWheel can be in pixels or lines, handle both
        total_scroll += match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y / 10.0, // Convert pixels to approximate lines
        };
    }

    if total_scroll != 0.0 {
        if let Ok(mut projection) = camera_query.get_single_mut() {
            // Adjust the scale based on scroll
            // Negative scroll (scroll down) = zoom out (increase scale)
            // Positive scroll (scroll up) = zoom in (decrease scale)
            let scale_change = -total_scroll * ZOOM_SPEED;
            let current_scale = projection.scale;
            let new_scale = (current_scale + scale_change).clamp(MIN_SCALE, MAX_SCALE);
            projection.scale = new_scale;
        }
    }
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_simulation, render_grid))
            .add_systems(
                Update,
                (
                    camera_movement,
                    camera_zoom,
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
