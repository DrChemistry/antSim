use crate::ant::{Ant, AntState};
use crate::marker::{Marker, MarkerType};
use bevy::prelude::*;

const FRAME_HISTORY_SIZE: usize = 60;
const HOVER_ZONE_SIZE: f32 = 100.0;

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

#[derive(Resource, Default)]
pub struct GuiSettings {
    pub hide_markers: bool,
    pub hide_ants: bool,
    pub hide_gui: bool,
    pub gui_hovered: bool,
}

#[derive(Component)]
pub struct DebugUI;

#[derive(Component)]
pub struct CheckboxHideMarkers;

#[derive(Component)]
pub struct CheckboxHideAnts;

#[derive(Component)]
pub struct CheckboxHideGUI;

#[derive(Component)]
pub struct MainStatsPanel;

#[derive(Component)]
pub struct HideGUIPanel;

#[derive(Component)]
pub struct HoverZone;

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
    // Main stats panel in bottom-left
    let main_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
                ..default()
            },
            MainStatsPanel,
        ))
        .id();

    commands.entity(main_panel).with_children(|parent| {
        // Hide Markers checkbox
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.3, 0.3, 0.3, 0.8).into(),
                    ..default()
                },
                CheckboxHideMarkers,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "☐ Hide Markers",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });

        // Hide Ants checkbox
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(4.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.3, 0.3, 0.3, 0.8).into(),
                    ..default()
                },
                CheckboxHideAnts,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "☐ Hide Ants",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });

        // Stats text
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

    // Hide GUI panel in top-left
    let hide_gui_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
                ..default()
            },
            HideGUIPanel,
        ))
        .id();

    commands.entity(hide_gui_panel).with_children(|parent| {
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.3, 0.3, 0.3, 0.8).into(),
                    ..default()
                },
                CheckboxHideGUI,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "☐ Hide GUI",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            });
    });

    // Hover zone in top-left corner (invisible but interactive)
    commands.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(HOVER_ZONE_SIZE),
                height: Val::Px(HOVER_ZONE_SIZE),
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        },
        HoverZone,
    ));
}

// Separate handlers for each checkbox
pub fn handle_hide_markers_checkbox(
    mut interaction_query: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<CheckboxHideMarkers>),
    >,
    mut settings: ResMut<GuiSettings>,
    mut text_query: Query<&mut Text>,
    children: Query<&Children>,
) {
    for (entity, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings.hide_markers = !settings.hide_markers;
            // Update checkbox text
            if let Ok(children) = children.get(entity) {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = if settings.hide_markers {
                            "☑ Hide Markers".to_string()
                        } else {
                            "☐ Hide Markers".to_string()
                        };
                    }
                }
            }
        }
    }
}

pub fn handle_hide_ants_checkbox(
    mut interaction_query: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<CheckboxHideAnts>),
    >,
    mut settings: ResMut<GuiSettings>,
    mut text_query: Query<&mut Text>,
    children: Query<&Children>,
) {
    for (entity, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings.hide_ants = !settings.hide_ants;
            // Update checkbox text
            if let Ok(children) = children.get(entity) {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = if settings.hide_ants {
                            "☑ Hide Ants".to_string()
                        } else {
                            "☐ Hide Ants".to_string()
                        };
                    }
                }
            }
        }
    }
}

pub fn handle_hide_gui_checkbox(
    mut interaction_query: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<CheckboxHideGUI>),
    >,
    mut settings: ResMut<GuiSettings>,
    mut text_query: Query<&mut Text>,
    children: Query<&Children>,
) {
    for (entity, interaction) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            settings.hide_gui = !settings.hide_gui;
            // Update checkbox text
            if let Ok(children) = children.get(entity) {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = if settings.hide_gui {
                            "☑ Hide GUI".to_string()
                        } else {
                            "☐ Hide GUI".to_string()
                        };
                    }
                }
            }
        }
    }
}

pub fn toggle_markers_visibility(
    mut commands: Commands,
    markers: Query<Entity, (With<Marker>, Without<Ant>, Without<Visibility>)>,
    mut markers_with_visibility: Query<&mut Visibility, (With<Marker>, Without<Ant>)>,
    settings: Res<GuiSettings>,
) {
    let target_visibility = if settings.hide_markers {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    // Insert Visibility component for entities that don't have it
    for entity in markers.iter() {
        commands.entity(entity).insert(target_visibility);
    }

    // Update existing Visibility components
    for mut visibility in markers_with_visibility.iter_mut() {
        if *visibility != target_visibility {
            *visibility = target_visibility;
        }
    }
}

pub fn toggle_ants_visibility(
    mut commands: Commands,
    ants: Query<Entity, (With<Ant>, Without<Visibility>)>,
    mut ants_with_visibility: Query<&mut Visibility, With<Ant>>,
    settings: Res<GuiSettings>,
) {
    let target_visibility = if settings.hide_ants {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    // Insert Visibility component for entities that don't have it
    for entity in ants.iter() {
        commands.entity(entity).insert(target_visibility);
    }

    // Update existing Visibility components
    for mut visibility in ants_with_visibility.iter_mut() {
        if *visibility != target_visibility {
            *visibility = target_visibility;
        }
    }
}

pub fn handle_gui_hover(
    mut hover_zone_query: Query<&Interaction, (With<HoverZone>, Changed<Interaction>)>,
    mut settings: ResMut<GuiSettings>,
) {
    // Reset hover state if GUI is not hidden
    if !settings.hide_gui {
        settings.gui_hovered = false;
        return;
    }

    for interaction in hover_zone_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                settings.gui_hovered = true;
            }
            Interaction::None => {
                settings.gui_hovered = false;
            }
            _ => {}
        }
    }
}

pub fn update_gui_visibility(
    mut queries: ParamSet<(
        Query<&mut Visibility, With<MainStatsPanel>>,
        Query<&mut Visibility, With<HideGUIPanel>>,
    )>,
    settings: Res<GuiSettings>,
) {
    let should_show = !settings.hide_gui || settings.gui_hovered;
    let target_visibility = if should_show {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Update main panel visibility
    for mut visibility in queries.p0().iter_mut() {
        if *visibility != target_visibility {
            *visibility = target_visibility;
        }
    }

    // Update hide GUI panel visibility
    for mut visibility in queries.p1().iter_mut() {
        if *visibility != target_visibility {
            *visibility = target_visibility;
        }
    }
}

pub struct DebugGUIPlugin;

impl Plugin for DebugGUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameTiming>()
            .init_resource::<GuiSettings>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(
                Update,
                (
                    update_frame_timing,
                    update_debug_ui,
                    handle_hide_markers_checkbox,
                    handle_hide_ants_checkbox,
                    handle_hide_gui_checkbox,
                    toggle_markers_visibility,
                    toggle_ants_visibility,
                    handle_gui_hover,
                    update_gui_visibility,
                ),
            );
    }
}
