use crate::ant::{Ant, AntState};
use bevy::prelude::*;

#[derive(Component)]
pub struct FoodSource;

#[derive(Component)]
pub struct FoodQuantity {
    pub quantity: u32,
}

pub fn check_food_collision(
    mut commands: Commands,
    mut ants: Query<(&Transform, &mut Ant, &mut Sprite), (With<Ant>, Without<FoodSource>)>,
    mut food_query: Query<
        (Entity, &Transform, &mut FoodQuantity),
        (With<FoodSource>, Without<Ant>),
    >,
) {
    const COLLISION_THRESHOLD: f32 = 10.0;

    for (ant_transform, mut ant, mut sprite) in ants.iter_mut() {
        if ant.state == AntState::Searching && !ant.has_food {
            for (food_entity, food_transform, mut food_quantity) in food_query.iter_mut() {
                let distance = ant_transform
                    .translation
                    .truncate()
                    .distance(food_transform.translation.truncate());

                if distance < COLLISION_THRESHOLD && food_quantity.quantity > 0 {
                    // Pick up food
                    ant.has_food = true;
                    ant.state = AntState::Returning;
                    ant.state_timer = 0.0;
                    ant.marker_timer = 0.0; // Reset marker timer to start leaving food markers immediately
                                            // Make ant do a U-turn
                    ant.velocity = -ant.velocity;

                    // Update ant color to returning state (green when carrying food)
                    sprite.color = Color::rgb(0.2, 0.8, 0.2);

                    // Decrease food quantity
                    food_quantity.quantity -= 1;

                    // Despawn food source if quantity reaches 0
                    if food_quantity.quantity == 0 {
                        commands.entity(food_entity).despawn();
                    }

                    break;
                }
            }
        }
    }
}
