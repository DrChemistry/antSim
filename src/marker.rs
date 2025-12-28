use crate::ant::AntState;
use bevy::prelude::*;

#[derive(Component)]
pub struct Marker {
    pub intensity: f32,
    pub marker_type: MarkerType,
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

// Spawn markers for ants
// Depending on the state of the ant, the marker type is different
pub fn spawn_markers(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut crate::ant::Ant)>,
    time: Res<Time>,
    config: Res<crate::config::Config>,
) {
    let dt = time.delta_seconds();

    for (transform, mut ant) in ants.iter_mut() {
        // Only spawn food markers for returning ants (carrying food)
        // Update marker timer
        ant.marker_timer += dt;
        ant.state_timer += dt;

        // Spawn marker at intervals
        if ant.marker_timer >= config.marker_spawn_interval {
            // Calculate initial intensity based on state timer
            let initial_intensity = INITIAL_INTENSITY - (ant.state_timer / config.marker_lifetime);

            commands.spawn((
                Marker {
                    intensity: initial_intensity,
                    marker_type: if ant.state == AntState::Returning {
                        MarkerType::Food
                    } else {
                        MarkerType::Base
                    },
                },
                MarkerLifetime {
                    timer: Timer::from_seconds(config.marker_lifetime, TimerMode::Once),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: if ant.state == AntState::Returning {
                            Color::rgba(0.2, 0.8, 0.2, 1.0) // Green color
                        } else {
                            Color::rgba(0.2, 0.6, 1.0, 1.0) // Blue color
                        },
                        custom_size: Some(Vec2::new(BASE_MARKER_SIZE, BASE_MARKER_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        transform.translation.truncate().extend(-0.1),
                    ), // Lower z-value to render behind ants
                    ..default()
                },
            ));

            ant.marker_timer = 0.0;
        }
    }
}

pub fn update_marker_visuals(
    mut commands: Commands,
    mut markers: Query<(&Marker, &mut Sprite, &mut MarkerLifetime, Entity)>,
    time: Res<Time>,
) {
    for (marker, mut sprite, mut lifetime, entity) in markers.iter_mut() {
        // Intensity stays constant, so opacity and size are based on initial intensity
        lifetime.timer.tick(time.delta());

        // Remove marker when timer finishes (reaches 0)
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn();
            break;
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
