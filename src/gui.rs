use crate::ant::{Ant, AntState};
use crate::marker::{Marker, MarkerType};
use bevy::prelude::*;

const FRAME_HISTORY_SIZE: usize = 60;

#[derive(Resource)]
pub struct FrameTiming {
    current_frame_time: f32,
    frame_history: Vec<f32>,
    history_index: usize,
}

impl Default for FrameTiming {
    fn default() -> Self {
        Self {
            current_frame_time: 0.0,
            frame_history: vec![0.0; FRAME_HISTORY_SIZE],
            history_index: 0,
        }
    }
}

impl FrameTiming {
    pub fn update(&mut self, delta: f32) {
        self.current_frame_time = delta * 1000.0; // Convert to milliseconds
        self.frame_history[self.history_index] = self.current_frame_time;
        self.history_index = (self.history_index + 1) % FRAME_HISTORY_SIZE;
    }

    pub fn current_ms(&self) -> f32 {
        self.current_frame_time
    }

    pub fn average_ms(&self) -> f32 {
        let sum: f32 = self.frame_history.iter().sum();
        sum / self.frame_history.len() as f32
    }
}

#[derive(Component)]
pub struct DebugUI;

pub fn update_frame_timing(mut frame_timing: ResMut<FrameTiming>, time: Res<Time<Real>>) {
    frame_timing.update(time.delta_seconds());
}

pub fn update_debug_ui(
    mut query: Query<&mut Text, With<DebugUI>>,
    frame_timing: Res<FrameTiming>,
    ants: Query<&Ant>,
    markers: Query<&Marker>,
) {
    // Count ants by state
    let mut searching_count = 0;
    let mut returning_count = 0;
    for ant in ants.iter() {
        match ant.state {
            AntState::Searching => searching_count += 1,
            AntState::Returning => returning_count += 1,
        }
    }
    let total_ants = searching_count + returning_count;

    // Count markers by type
    let mut base_marker_count = 0;
    let mut food_marker_count = 0;
    for marker in markers.iter() {
        match marker.marker_type {
            MarkerType::Base => base_marker_count += 1,
            MarkerType::Food => food_marker_count += 1,
        }
    }
    let total_markers = base_marker_count + food_marker_count;

    // Update the text
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!(
            "Frame Time: {:.2} ms\n\
             Avg Frame Time: {:.2} ms\n\
             \n\
             Ants: {}\n\
             - Searching: {}\n\
             - Returning: {}\n\
             \n\
             Markers: {}\n\
             - Base: {}\n\
             - Food: {}",
            frame_timing.current_ms(),
            frame_timing.average_ms(),
            total_ants,
            searching_count,
            returning_count,
            total_markers,
            base_marker_count,
            food_marker_count
        );
    }
}

pub fn setup_debug_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                bottom: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                DebugUI,
            ));
        });
}

pub struct DebugGUIPlugin;

impl Plugin for DebugGUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameTiming>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(Update, (update_frame_timing, update_debug_ui));
    }
}
