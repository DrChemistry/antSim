use crate::ant::{Ant, AntState};
use crate::gui::FrameTiming;
use crate::marker::{Marker, MarkerType};
use bevy::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Resource)]
pub struct SimulationLogger {
    log_timer: Timer,
    file_path: PathBuf,
    header_written: bool,
}

impl SimulationLogger {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create logs directory if it doesn't exist
        let logs_dir = PathBuf::from("logs");
        if !logs_dir.exists() {
            std::fs::create_dir_all(&logs_dir)?;
        }

        // Generate timestamped filename
        let now = chrono::Local::now();
        let filename = format!("simulation_{}.csv", now.format("%Y-%m-%d_%H-%M-%S"));
        let file_path = logs_dir.join(filename);

        Ok(Self {
            log_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            file_path,
            header_written: false,
        })
    }

    fn write_header(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        writeln!(
            file,
            "timestamp,frame_time_ms,avg_frame_time_ms,total_ants,searching_ants,returning_ants,total_markers,food_markers,base_markers"
        )?;

        self.header_written = true;
        Ok(())
    }

    fn write_log_entry(
        &mut self,
        frame_time_ms: f32,
        avg_frame_time_ms: f32,
        total_ants: usize,
        searching_ants: usize,
        returning_ants: usize,
        total_markers: usize,
        food_markers: usize,
        base_markers: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Write header if not written yet
        if !self.header_written {
            self.write_header()?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(
            file,
            "{},{:.2},{:.2},{},{},{},{},{},{}",
            timestamp,
            frame_time_ms,
            avg_frame_time_ms,
            total_ants,
            searching_ants,
            returning_ants,
            total_markers,
            food_markers,
            base_markers
        )?;

        Ok(())
    }

    pub fn should_log(&mut self, time: &Time, frame_time_ms: f32) -> bool {
        // If frame time > 1 second, log every update
        if frame_time_ms > 1000.0 {
            return true;
        }

        // Otherwise, log every 1 second
        self.log_timer.tick(time.delta());
        self.log_timer.just_finished()
    }
}

pub fn log_simulation_stats(
    mut logger: ResMut<SimulationLogger>,
    time: Res<Time>,
    frame_timing: Res<FrameTiming>,
    ants: Query<&Ant>,
    markers: Query<&Marker>,
) {
    let frame_time_ms = frame_timing.current_ms();

    // Check if we should log
    if !logger.should_log(&time, frame_time_ms) {
        return;
    }

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

    // Write log entry
    if let Err(e) = logger.write_log_entry(
        frame_time_ms,
        frame_timing.average_ms(),
        total_ants,
        searching_count,
        returning_count,
        total_markers,
        food_marker_count,
        base_marker_count,
    ) {
        eprintln!("Error writing log entry: {}", e);
    }
}

pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize logger resource
        match SimulationLogger::new() {
            Ok(logger) => {
                app.insert_resource(logger);
                app.add_systems(
                    Update,
                    log_simulation_stats.after(crate::gui::update_frame_timing),
                );
            }
            Err(e) => {
                eprintln!("Failed to initialize simulation logger: {}", e);
            }
        }
    }
}
