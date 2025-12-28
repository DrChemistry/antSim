use crate::chart_data::{LogEntry, SimulationData};

#[derive(Clone)]
pub enum XAxisType {
    Samples,
    Time,
}

pub fn generate_performance_charts(
    simulations: &[SimulationData],
    x_axis_type: XAxisType,
) -> Vec<String> {
    let mut charts = Vec::new();

    // Frame Time chart
    charts.push(generate_chart(
        "Frame Time",
        "Frame Time (ms)",
        simulations,
        x_axis_type.clone(),
        |entry| entry.frame_time_ms,
    ));

    // Average Frame Time chart
    charts.push(generate_chart(
        "Average Frame Time",
        "Average Frame Time (ms)",
        simulations,
        x_axis_type.clone(),
        |entry| entry.avg_frame_time_ms,
    ));

    charts
}

pub fn generate_ant_charts(simulations: &[SimulationData], x_axis_type: XAxisType) -> Vec<String> {
    let mut charts = Vec::new();

    // Total Ants chart
    charts.push(generate_chart(
        "Total Ants",
        "Total Ants",
        simulations,
        x_axis_type.clone(),
        |entry| entry.total_ants as f32,
    ));

    // Searching Ants chart
    charts.push(generate_chart(
        "Searching Ants",
        "Searching Ants",
        simulations,
        x_axis_type.clone(),
        |entry| entry.searching_ants as f32,
    ));

    // Returning Ants chart
    charts.push(generate_chart(
        "Returning Ants",
        "Returning Ants",
        simulations,
        x_axis_type,
        |entry| entry.returning_ants as f32,
    ));

    charts
}

pub fn generate_marker_charts(
    simulations: &[SimulationData],
    x_axis_type: XAxisType,
) -> Vec<String> {
    let mut charts = Vec::new();

    // Total Markers chart
    charts.push(generate_chart(
        "Total Markers",
        "Total Markers",
        simulations,
        x_axis_type.clone(),
        |entry| entry.total_markers as f32,
    ));

    // Food Markers chart
    charts.push(generate_chart(
        "Food Markers",
        "Food Markers",
        simulations,
        x_axis_type.clone(),
        |entry| entry.food_markers as f32,
    ));

    // Base Markers chart
    charts.push(generate_chart(
        "Base Markers",
        "Base Markers",
        simulations,
        x_axis_type,
        |entry| entry.base_markers as f32,
    ));

    charts
}

fn generate_chart<F>(
    title: &str,
    y_label: &str,
    simulations: &[SimulationData],
    x_axis_type: XAxisType,
    value_extractor: F,
) -> String
where
    F: Fn(&LogEntry) -> f32,
{
    if simulations.is_empty() {
        return format!("<!-- No data for {} -->", title);
    }

    // Find minimum length for alignment
    let min_len = simulations.iter().map(|s| s.len()).min().unwrap_or(0);
    if min_len == 0 {
        return format!("<!-- No data for {} -->", title);
    }

    // Extract data from all simulations
    let mut all_values: Vec<Vec<f32>> = Vec::new();

    for sim in simulations {
        let values: Vec<f32> = sim
            .entries
            .iter()
            .take(min_len)
            .map(&value_extractor)
            .collect();

        if !values.is_empty() {
            all_values.push(values);
        }
    }

    if all_values.is_empty() {
        return format!("<!-- No data for {} -->", title);
    }

    // Generate x-axis
    let x_axis_values: Vec<String> = match x_axis_type {
        XAxisType::Samples => (0..min_len).map(|i| i.to_string()).collect(),
        XAxisType::Time => {
            if let Some(first_sim) = simulations.first() {
                let times = crate::chart_data::normalize_time_axis(&first_sim.entries[..min_len]);
                times.iter().map(|t| format!("{:.1}", t)).collect()
            } else {
                (0..min_len).map(|i| i.to_string()).collect()
            }
        }
    };

    // Calculate y-axis range
    let all_flat: Vec<f32> = all_values.iter().flatten().copied().collect();
    let min_val = all_flat.iter().copied().fold(f32::INFINITY, f32::min);
    let max_val = all_flat.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let y_min = if min_val.is_finite() {
        (min_val * 0.9).max(0.0).floor()
    } else {
        0.0
    };
    let y_max = if max_val.is_finite() {
        (max_val * 1.1).ceil()
    } else {
        100.0
    };

    // Build Mermaid chart
    let mut chart = format!(
        "xychart-beta\n    title \"{}\"\n    x-axis [{}]\n    y-axis \"{}\" {} --> {}\n",
        title,
        x_axis_values.join(", "),
        y_label,
        y_min as i32,
        y_max as i32
    );

    // Add lines for each simulation
    for (idx, values) in all_values.iter().enumerate() {
        let label = if simulations.len() > 1 {
            // Use filename without extension as label
            let sim_name = &simulations[idx].filename;
            let label = sim_name
                .strip_suffix(".csv")
                .unwrap_or(sim_name)
                .to_string();
            format!("\"{}\"", label)
        } else {
            String::new()
        };

        let values_str: Vec<String> = values.iter().map(|v| format!("{:.2}", v)).collect();
        chart.push_str(&format!("    line {} [{}]\n", label, values_str.join(", ")));
    }

    chart
}

pub fn generate_markdown(
    simulations: &[SimulationData],
    metrics: &[String],
    x_axis_type: XAxisType,
) -> String {
    let mut markdown = String::new();

    // Header
    markdown.push_str("# Simulation Charts\n\n");

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    markdown.push_str(&format!("Generated: {}\n\n", now));

    if simulations.len() == 1 {
        markdown.push_str(&format!("Source: {}\n\n", simulations[0].filename));
    } else {
        markdown.push_str("Sources:\n");
        for sim in simulations {
            markdown.push_str(&format!("- {}\n", sim.filename));
        }
        markdown.push_str("\n");
    }

    // Performance Metrics
    if metrics.contains(&"all".to_string()) || metrics.contains(&"performance".to_string()) {
        markdown.push_str("## Performance Metrics\n\n");
        let charts = generate_performance_charts(simulations, x_axis_type.clone());
        for (idx, chart) in charts.iter().enumerate() {
            let chart_titles = ["Frame Time", "Average Frame Time"];
            if idx < chart_titles.len() {
                markdown.push_str(&format!("### {}\n\n", chart_titles[idx]));
            }
            markdown.push_str("```mermaid\n");
            markdown.push_str(chart);
            markdown.push_str("```\n\n");
        }
    }

    // Ant Charts
    if metrics.contains(&"all".to_string()) || metrics.contains(&"ants".to_string()) {
        markdown.push_str("## Ant Metrics\n\n");
        let charts = generate_ant_charts(simulations, x_axis_type.clone());
        let chart_titles = ["Total Ants", "Searching Ants", "Returning Ants"];
        for (idx, chart) in charts.iter().enumerate() {
            if idx < chart_titles.len() {
                markdown.push_str(&format!("### {}\n\n", chart_titles[idx]));
            }
            markdown.push_str("```mermaid\n");
            markdown.push_str(chart);
            markdown.push_str("```\n\n");
        }
    }

    // Marker Charts
    if metrics.contains(&"all".to_string()) || metrics.contains(&"markers".to_string()) {
        markdown.push_str("## Marker Metrics\n\n");
        let charts = generate_marker_charts(simulations, x_axis_type);
        let chart_titles = ["Total Markers", "Food Markers", "Base Markers"];
        for (idx, chart) in charts.iter().enumerate() {
            if idx < chart_titles.len() {
                markdown.push_str(&format!("### {}\n\n", chart_titles[idx]));
            }
            markdown.push_str("```mermaid\n");
            markdown.push_str(chart);
            markdown.push_str("```\n\n");
        }
    }

    markdown
}
