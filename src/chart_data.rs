use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub frame_time_ms: f32,
    pub avg_frame_time_ms: f32,
    pub total_ants: usize,
    pub searching_ants: usize,
    pub returning_ants: usize,
    pub total_markers: usize,
    pub food_markers: usize,
    pub base_markers: usize,
}

#[derive(Debug, Clone)]
pub struct SimulationData {
    pub filename: String,
    pub entries: Vec<LogEntry>,
}

impl SimulationData {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn parse_csv_file(path: &Path) -> Result<SimulationData, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut entries = Vec::new();

    for result in rdr.records() {
        let record = result?;

        if record.len() < 9 {
            continue; // Skip invalid rows
        }

        let entry = LogEntry {
            timestamp: record.get(0).unwrap_or("").to_string(),
            frame_time_ms: record.get(1).unwrap_or("0").parse().unwrap_or(0.0),
            avg_frame_time_ms: record.get(2).unwrap_or("0").parse().unwrap_or(0.0),
            total_ants: record.get(3).unwrap_or("0").parse().unwrap_or(0),
            searching_ants: record.get(4).unwrap_or("0").parse().unwrap_or(0),
            returning_ants: record.get(5).unwrap_or("0").parse().unwrap_or(0),
            total_markers: record.get(6).unwrap_or("0").parse().unwrap_or(0),
            food_markers: record.get(7).unwrap_or("0").parse().unwrap_or(0),
            base_markers: record.get(8).unwrap_or("0").parse().unwrap_or(0),
        };

        entries.push(entry);
    }

    Ok(SimulationData { filename, entries })
}

pub fn parse_multiple_csv_files(
    paths: Vec<PathBuf>,
) -> Result<Vec<SimulationData>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();

    for path in paths {
        match parse_csv_file(&path) {
            Ok(data) => results.push(data),
            Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
        }
    }

    Ok(results)
}

pub fn normalize_time_axis(entries: &[LogEntry]) -> Vec<f32> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Parse first timestamp as reference
    let first_timestamp = &entries[0].timestamp;
    let first_time = parse_timestamp(first_timestamp);

    entries
        .iter()
        .map(|entry| {
            let current_time = parse_timestamp(&entry.timestamp);
            (current_time - first_time) as f32
        })
        .collect()
}

fn parse_timestamp(timestamp: &str) -> i64 {
    // Try to parse timestamp format: "2025-12-28 16:03:38.890"
    // Convert to seconds since epoch for calculation
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S%.f") {
        dt.and_utc().timestamp()
    } else {
        // Fallback: use index if parsing fails
        0
    }
}

pub fn find_all_log_files(logs_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut log_files = Vec::new();

    if !logs_dir.exists() {
        return Ok(log_files);
    }

    let entries = std::fs::read_dir(logs_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("simulation_") && file_name.ends_with(".csv") {
                    log_files.push(path);
                }
            }
        }
    }

    // Sort by filename (which includes timestamp) for consistent ordering
    log_files.sort();

    Ok(log_files)
}
