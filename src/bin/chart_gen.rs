use ant_sim::chart_data::{find_all_log_files, parse_csv_file, parse_multiple_csv_files};
use ant_sim::chart_generator::{generate_markdown, XAxisType};
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "chart-gen")]
#[command(about = "Generate Mermaid charts from simulation log files")]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(["file", "compare", "all"])
))]
struct Args {
    /// Single CSV file to chart
    #[arg(long)]
    file: Option<PathBuf>,

    /// Multiple CSV files for comparison
    #[arg(long, num_args = 1..)]
    compare: Option<Vec<PathBuf>>,

    /// Use all CSV files in the logs/ directory
    #[arg(long)]
    all: bool,

    /// Output markdown file path
    #[arg(long, default_value = "")]
    output: String,

    /// Metrics to include: all, performance, ants, markers (comma-separated)
    #[arg(long, default_value = "all")]
    metrics: String,

    /// X-axis type: samples or time
    #[arg(long, default_value = "samples")]
    x_axis: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Determine which files to process
    let csv_files: Vec<PathBuf> = if args.all {
        let logs_dir = PathBuf::from("logs");
        find_all_log_files(&logs_dir)?
    } else if let Some(file) = args.file {
        vec![file]
    } else if let Some(files) = args.compare {
        files
    } else {
        eprintln!("Error: Must specify --file, --compare, or --all");
        std::process::exit(1);
    };

    if csv_files.is_empty() {
        eprintln!("Error: No CSV files found to process");
        std::process::exit(1);
    }

    println!("Processing {} file(s)...", csv_files.len());

    // Parse CSV files
    let simulations = if csv_files.len() == 1 {
        vec![parse_csv_file(&csv_files[0])?]
    } else {
        parse_multiple_csv_files(csv_files)?
    };

    if simulations.is_empty() {
        eprintln!("Error: No valid simulation data found");
        std::process::exit(1);
    }

    // Parse metrics
    let metrics: Vec<String> = args
        .metrics
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();

    // Parse x-axis type
    let x_axis_type = match args.x_axis.to_lowercase().as_str() {
        "time" => XAxisType::Time,
        "samples" | _ => XAxisType::Samples,
    };

    // Generate markdown
    let markdown = generate_markdown(&simulations, &metrics, x_axis_type);

    // Determine output path
    let output_path = if args.output.is_empty() {
        // Generate default output path
        let charts_dir = PathBuf::from("charts");
        if !charts_dir.exists() {
            std::fs::create_dir_all(&charts_dir)?;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        charts_dir.join(format!("chart_{}.md", timestamp))
    } else {
        PathBuf::from(args.output)
    };

    // Write output
    std::fs::write(&output_path, markdown)?;
    println!("Charts generated successfully: {}", output_path.display());

    Ok(())
}
