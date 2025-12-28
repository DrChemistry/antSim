use crate::ant::{Ant, AntState};
use bevy::prelude::*;

#[derive(Component)]
pub struct Base;

#[derive(Resource)]
pub struct SpawnTimer {
    pub timer: Timer,
}

pub fn spawn_ants(
    mut commands: Commands,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    base_query: Query<&Transform, (With<Base>, Without<Ant>)>,
    _config: Res<crate::config::Config>,
) {
    // Only spawn ants if spawn rate is greater than 0
    if _config.spawn_rate > 0.0 {
        spawn_timer.timer.tick(time.delta());

        if spawn_timer.timer.just_finished() {
            if let Ok(base_transform) = base_query.get_single() {
                commands.spawn((
                    Ant::new(),
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.8, 0.2, 0.2),
                            custom_size: Some(Vec2::new(6.0, 6.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(base_transform.translation),
                        ..default()
                    },
                ));
            }
        }
    }
}

pub fn check_base_collision(
    mut ants: Query<(&Transform, &mut Ant, &mut Sprite), (With<Ant>, Without<Base>)>,
    base_query: Query<&Transform, (With<Base>, Without<Ant>)>,
) {
    const COLLISION_THRESHOLD: f32 = 10.0;

    if let Ok(base_transform) = base_query.get_single() {
        for (transform, mut ant, mut sprite) in ants.iter_mut() {
            if ant.state == AntState::Returning && ant.has_food {
                let distance = transform
                    .translation
                    .truncate()
                    .distance(base_transform.translation.truncate());

                if distance < COLLISION_THRESHOLD {
                    // Drop food at base
                    ant.has_food = false;
                    ant.state = AntState::Searching;
                    ant.state_timer = 0.0;
                    ant.marker_timer = 0.0; // Reset marker timer to start leaving base markers immediately
                                            // Make ant do a U-turn
                    ant.velocity = -ant.velocity;
                    // Update ant color to searching state
                    sprite.color = Color::rgb(0.8, 0.2, 0.2);
                }
            }
        }
    }
}
