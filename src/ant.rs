use crate::marker::{GridMap, Marker, MarkerType};
use bevy::prelude::*;
use rand::Rng;

#[derive(Component, Debug)]
pub struct Ant {
    pub state: AntState,
    pub has_food: bool,
    pub velocity: Vec2,
    pub direction_change_timer: f32,
    pub marker_timer: f32,
    pub state_timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AntState {
    Searching,
    Returning,
}

impl Ant {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        Self {
            state: AntState::Searching,
            has_food: false,
            velocity: Vec2::new(angle.cos(), angle.sin()),
            direction_change_timer: 0.0,
            marker_timer: 0.0,
            state_timer: 0.0,
        }
    }
}

pub fn move_ants(
    mut ants: Query<(&mut Transform, &mut Ant)>,
    time: Res<Time>,
    base_pos: Query<&Transform, (With<crate::base::Base>, Without<Ant>)>,
    food_query: Query<&Transform, (With<crate::food::FoodSource>, Without<Ant>)>,
) {
    const ANT_SPEED: f32 = 50.0;
    const DIRECTION_CHANGE_INTERVAL: f32 = 1.5;
    const COLLISION_THRESHOLD: f32 = 5.0;
    const FOOD_DETECTION_RADIUS: f32 = 50.0; // Radius to detect nearby food

    let dt = time.delta_seconds();

    for (mut transform, mut ant) in ants.iter_mut() {
        match ant.state {
            AntState::Searching => {
                let ant_pos = transform.translation.truncate();
                let mut closest_food: Option<Vec2> = None;
                let mut closest_distance = FOOD_DETECTION_RADIUS;

                // Check for nearby food sources
                for food_transform in food_query.iter() {
                    let food_pos = food_transform.translation.truncate();
                    let distance = ant_pos.distance(food_pos);

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_food = Some(food_pos);
                    }
                }

                // If food is nearby, move directly toward it
                if let Some(food_pos) = closest_food {
                    let direction_to_food = (food_pos - ant_pos).normalize();
                    ant.velocity = direction_to_food;
                } else {
                    // No food nearby, continue with normal searching behavior
                    // Update direction change timer
                    ant.direction_change_timer += dt;

                    // Change direction periodically
                    // But only a few degrees at a time
                    if ant.direction_change_timer >= DIRECTION_CHANGE_INTERVAL {
                        let mut rng = rand::thread_rng();
                        // Get current angle of velocity vector
                        let current_angle = ant.velocity.y.atan2(ant.velocity.x);
                        // Add a small random change (in radians, ~±6 degrees)
                        let angle_change = rng.gen_range(-0.1..0.1);
                        let new_angle = current_angle + angle_change;
                        // Create new velocity vector with slightly changed direction
                        ant.velocity = Vec2::new(new_angle.cos(), new_angle.sin()).normalize();
                        ant.direction_change_timer = 0.0;
                    }
                }
            }
            AntState::Returning => {
                // Move toward base, but marker following may have already influenced direction
                // If no markers were found, move directly toward base
                if let Ok(base_transform) = base_pos.get_single() {
                    let base_direction = (base_transform.translation.truncate()
                        - transform.translation.truncate())
                    .normalize();

                    // Blend base direction with current velocity (which may have been influenced by markers)
                    // This allows markers to guide the path while still generally heading toward base
                    let blended = (ant.velocity * 0.7 + base_direction * 0.3).normalize();
                    ant.velocity = blended;

                    // Check if reached base
                    let distance = transform
                        .translation
                        .truncate()
                        .distance(base_transform.translation.truncate());
                    if distance < COLLISION_THRESHOLD {
                        // Will be handled by base collision system
                    }
                }
            }
        }

        // Move ant
        transform.translation += (ant.velocity * ANT_SPEED * dt).extend(0.0);
    }
}

pub fn follow_markers(
    mut ants: Query<(&Transform, &mut Ant)>,
    markers: Query<(&Marker, &Transform), (With<Marker>, Without<Ant>)>,
    grid_map: Res<GridMap>,
) {
    const MARKER_DETECTION_RADIUS: f32 = 10.0;
    const MAX_INTENSITY: f32 = 100.0;
    const INFLUENCE_STRENGTH: f32 = 0.3; // How much markers influence direction (0.0 to 1.0)

    for (ant_transform, mut ant) in ants.iter_mut() {
        // Determine which marker type to follow based on ant state
        let target_marker_type = match ant.state {
            AntState::Searching => MarkerType::Food,
            AntState::Returning => MarkerType::Base,
        };

        let ant_pos = ant_transform.translation.truncate();
        let mut strongest_marker: Option<(Vec2, f32)> = None; // (position, intensity)

        // Get nearby grid cells
        let nearby_cells = grid_map.get_nearby_cells(ant_pos, MARKER_DETECTION_RADIUS);

        // Check markers in nearby cells
        for cell in nearby_cells {
            if let Some(cell_data) = grid_map.get_cell(cell) {
                // Get the marker entity of the target type
                let marker_entity = match target_marker_type {
                    MarkerType::Base => cell_data.base_marker,
                    MarkerType::Food => cell_data.food_marker,
                };

                if let Some(entity) = marker_entity {
                    // Query the marker to get its data
                    if let Ok((marker, marker_transform)) = markers.get(entity) {
                        if marker.marker_type != target_marker_type {
                            continue;
                        }

                        let marker_pos = marker_transform.translation.truncate();
                        let distance = ant_pos.distance(marker_pos);

                        if distance <= MARKER_DETECTION_RADIUS {
                            // Use intensity as the strength
                            let strength = marker.intensity;

                            if let Some((_, current_strength)) = strongest_marker {
                                if strength > current_strength {
                                    strongest_marker = Some((marker_pos, strength));
                                }
                            } else {
                                strongest_marker = Some((marker_pos, strength));
                            }
                        }
                    }
                }
            }
        }

        // If a marker was found, blend its direction with current velocity
        if let Some((marker_pos, intensity)) = strongest_marker {
            // Calculate direction toward the marker
            let direction_to_marker = (marker_pos - ant_pos).normalize();

            // Calculate influence factor based on marker intensity
            let influence = (intensity / MAX_INTENSITY) * INFLUENCE_STRENGTH;

            // Blend current velocity with marker direction
            let blended_velocity =
                ant.velocity * (1.0 - influence) + direction_to_marker * influence;
            ant.velocity = blended_velocity.normalize();
        }
    }
}

pub fn keep_ants_in_bounds(
    mut ants: Query<&mut Transform, With<Ant>>,
    config: Res<crate::config::Config>,
) {
    use crate::marker::GRID_CELL_SIZE;
    // Map size in config is grid cells, convert to pixels
    let map_width_pixels = config.map_size.0 as f32 * GRID_CELL_SIZE;
    let map_height_pixels = config.map_size.1 as f32 * GRID_CELL_SIZE;

    for mut transform in ants.iter_mut() {
        // Wrap around horizontally: left to right, right to left
        if transform.translation.x < 0.0 {
            transform.translation.x = map_width_pixels;
        } else if transform.translation.x > map_width_pixels {
            transform.translation.x = 0.0;
        }

        // Wrap around vertically: up to down, down to up
        if transform.translation.y < 0.0 {
            transform.translation.y = map_height_pixels;
        } else if transform.translation.y > map_height_pixels {
            transform.translation.y = 0.0;
        }
    }
}
