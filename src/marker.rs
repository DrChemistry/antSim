use crate::ant::AntState;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Marker {
    pub intensity: f32,
    pub marker_type: MarkerType,
    pub grid_cell: (i32, i32), // Grid cell coordinates
}

#[derive(Component)]
pub struct MarkerLifetime {
    pub timer: Timer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkerType {
    Base,
    Food,
}

const INITIAL_INTENSITY: f32 = 100.0;
const BASE_MARKER_SIZE: f32 = 3.0;
pub const GRID_CELL_SIZE: f32 = 32.0;

// Grid cell data structure
#[derive(Default)]
pub struct GridCellData {
    pub base_marker: Option<Entity>,
    pub food_marker: Option<Entity>,
}

// Grid map resource to track markers per cell
#[derive(Resource, Default)]
pub struct GridMap {
    cells: HashMap<(i32, i32), GridCellData>,
}

impl GridMap {
    pub fn get_cell(&self, cell: (i32, i32)) -> Option<&GridCellData> {
        self.cells.get(&cell)
    }

    pub fn get_cell_mut(&mut self, cell: (i32, i32)) -> &mut GridCellData {
        self.cells.entry(cell).or_insert_with(GridCellData::default)
    }

    pub fn set_marker(&mut self, cell: (i32, i32), marker_type: MarkerType, entity: Entity) {
        let cell_data = self.get_cell_mut(cell);
        match marker_type {
            MarkerType::Base => cell_data.base_marker = Some(entity),
            MarkerType::Food => cell_data.food_marker = Some(entity),
        }
    }

    pub fn remove_marker(&mut self, cell: (i32, i32), marker_type: MarkerType) {
        if let Some(cell_data) = self.cells.get_mut(&cell) {
            match marker_type {
                MarkerType::Base => cell_data.base_marker = None,
                MarkerType::Food => cell_data.food_marker = None,
            }
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn get_nearby_cells(&self, pos: Vec2, radius: f32) -> Vec<(i32, i32)> {
        let center_cell = world_to_grid(pos);
        let radius_cells = (radius / GRID_CELL_SIZE).ceil() as i32;
        let mut cells = Vec::new();

        for dx in -radius_cells..=radius_cells {
            for dy in -radius_cells..=radius_cells {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                let cell_world = grid_to_world(cell);
                if pos.distance(cell_world) <= radius {
                    cells.push(cell);
                }
            }
        }
        cells
    }
}

// Get the 3x3 grid cells in front of the ant based on their velocity direction
pub fn get_front_cells(pos: Vec2, velocity: Vec2) -> Vec<(i32, i32)> {
    let current_cell = world_to_grid(pos);

    // Normalize velocity to get direction
    let direction = if velocity.length() > 0.01 {
        velocity.normalize()
    } else {
        // Default to moving right if velocity is too small
        Vec2::new(1.0, 0.0)
    };

    // Calculate which cell is directly in front
    // We look 1-2 grid cells ahead in the direction of movement
    // Use the dominant direction component to determine the front cell
    let front_offset_x = if direction.x.abs() > direction.y.abs() {
        // Moving more horizontally
        direction.x.signum() as i32
    } else if direction.x.abs() < direction.y.abs() {
        // Moving more vertically
        0
    } else {
        // Diagonal movement - use both components
        direction.x.signum() as i32
    };

    let front_offset_y = if direction.y.abs() > direction.x.abs() {
        // Moving more vertically
        direction.y.signum() as i32
    } else if direction.y.abs() < direction.x.abs() {
        // Moving more horizontally
        0
    } else {
        // Diagonal movement - use both components
        direction.y.signum() as i32
    };

    // Center cell is the one directly in front (1 cell ahead)
    let front_center_cell = (
        current_cell.0 + front_offset_x,
        current_cell.1 + front_offset_y,
    );

    // Get 3x3 grid centered on the front cell
    let mut cells = Vec::new();
    for dx in -1..=1 {
        for dy in -1..=1 {
            cells.push((front_center_cell.0 + dx, front_center_cell.1 + dy));
        }
    }
    cells
}

// Convert world position to grid cell coordinates
pub fn world_to_grid(pos: Vec2) -> (i32, i32) {
    (
        (pos.x / GRID_CELL_SIZE).floor() as i32,
        (pos.y / GRID_CELL_SIZE).floor() as i32,
    )
}

// Convert grid cell coordinates to world position (center of cell)
pub fn grid_to_world(cell: (i32, i32)) -> Vec2 {
    Vec2::new(
        (cell.0 as f32 * GRID_CELL_SIZE) + (GRID_CELL_SIZE / 2.0),
        (cell.1 as f32 * GRID_CELL_SIZE) + (GRID_CELL_SIZE / 2.0),
    )
}

// Spawn markers for ants
// Depending on the state of the ant, the marker type is different
pub fn spawn_markers(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut crate::ant::Ant)>,
    mut grid_map: ResMut<GridMap>,
    time: Res<Time>,
    config: Res<crate::config::Config>,
) {
    let dt = time.delta_seconds();

    for (transform, mut ant) in ants.iter_mut() {
        // Update marker timer
        ant.marker_timer += dt;
        ant.state_timer += dt;

        // Spawn marker at intervals
        if ant.marker_timer >= config.marker_spawn_interval {
            // Find nearest grid cell to ant's position
            let ant_pos = transform.translation.truncate();
            let grid_cell = world_to_grid(ant_pos);
            let marker_type = if ant.state == AntState::Returning {
                MarkerType::Food
            } else {
                MarkerType::Base
            };

            // Check if cell already has a marker of this type
            let cell_data = grid_map.get_cell(grid_cell);
            if let Some(cell_data) = cell_data {
                // If marker exists, despawn it (replace behavior)
                if let Some(old_entity) = match marker_type {
                    MarkerType::Base => cell_data.base_marker,
                    MarkerType::Food => cell_data.food_marker,
                } {
                    commands.entity(old_entity).despawn();
                }
            }

            // Calculate initial intensity based on state timer
            let initial_intensity = INITIAL_INTENSITY - (ant.state_timer / config.marker_lifetime);

            // Position marker at center of grid cell
            let marker_world_pos = grid_to_world(grid_cell);

            // Spawn new marker
            let marker_entity = commands
                .spawn((
                    Marker {
                        intensity: initial_intensity,
                        marker_type,
                        grid_cell,
                    },
                    MarkerLifetime {
                        timer: Timer::from_seconds(config.marker_lifetime, TimerMode::Once),
                    },
                    SpriteBundle {
                        sprite: Sprite {
                            color: if marker_type == MarkerType::Food {
                                Color::rgba(0.2, 0.8, 0.2, 1.0) // Green color
                            } else {
                                Color::rgba(0.2, 0.6, 1.0, 1.0) // Blue color
                            },
                            custom_size: Some(Vec2::new(BASE_MARKER_SIZE, BASE_MARKER_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_translation(marker_world_pos.extend(-0.1)), // Lower z-value to render behind ants
                        ..default()
                    },
                ))
                .id();

            // Register marker in grid map
            grid_map.set_marker(grid_cell, marker_type, marker_entity);

            ant.marker_timer = 0.0;
        }
    }
}

pub fn update_marker_visuals(
    mut commands: Commands,
    mut markers: Query<(&Marker, &mut Sprite, &mut MarkerLifetime, Entity)>,
    mut grid_map: ResMut<GridMap>,
    time: Res<Time>,
) {
    for (marker, mut sprite, mut lifetime, entity) in markers.iter_mut() {
        // Intensity stays constant, so opacity and size are based on initial intensity
        lifetime.timer.tick(time.delta());

        // Remove marker when timer finishes (reaches 0)
        if lifetime.timer.just_finished() {
            // Remove from grid map
            grid_map.remove_marker(marker.grid_cell, marker.marker_type);
            commands.entity(entity).despawn();
            continue;
        }
        let opacity = (marker.intensity / INITIAL_INTENSITY).clamp(0.0, 1.0);

        // Use different colors based on marker type
        let color = match marker.marker_type {
            MarkerType::Base => Color::rgba(0.2, 0.6, 1.0, opacity), // Blue
            MarkerType::Food => Color::rgba(0.2, 0.8, 0.2, opacity), // Green
        };
        sprite.color = color;

        // Size based on intensity (which stays constant)
        let size_scale = (marker.intensity / INITIAL_INTENSITY).clamp(0.0, 1.0);
        let size = BASE_MARKER_SIZE * size_scale;
        sprite.custom_size = Some(Vec2::new(size, size));
    }
}
