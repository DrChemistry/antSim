use crate::ant::{Ant, AntState};
use bevy::prelude::*;

#[derive(Component)]
pub struct FoodSource;

pub fn check_food_collision(
    mut ants: Query<(&Transform, &mut Ant, &mut Sprite), (With<Ant>, Without<FoodSource>)>,
    food_query: Query<&Transform, (With<FoodSource>, Without<Ant>)>,
) {
    const COLLISION_THRESHOLD: f32 = 10.0;

    for (ant_transform, mut ant, mut sprite) in ants.iter_mut() {
        if ant.state == AntState::Searching && !ant.has_food {
            for food_transform in food_query.iter() {
                let distance = ant_transform
                    .translation
                    .truncate()
                    .distance(food_transform.translation.truncate());

                if distance < COLLISION_THRESHOLD {
                    // Pick up food
                    ant.has_food = true;
                    ant.state = AntState::Returning;
                    ant.state_timer = 0.0;
                    ant.marker_timer = 0.0; // Reset marker timer to start leaving food markers immediately
                                            // Make ant do a U-turn
                    ant.velocity = -ant.velocity;

                    // Update ant color to returning state (green when carrying food)
                    sprite.color = Color::rgb(0.2, 0.8, 0.2);
                    break;
                }
            }
        }
    }
}
