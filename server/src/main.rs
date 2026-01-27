//! AMP Server - Address-Parking Correlation Server
//! Supports interactive Ratatui TUI, CLI commands, web testing, and benchmarking

mod app;
mod classification;
mod cli;
mod tui;
mod ui;

use crate::cli::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Correlate { algorithm, cutoff } => {
            cli::run_correlation(algorithm, cutoff)?;
        }
        Commands::Tui => {
            // Launch interactive Ratatui TUI
            let mut app = ui::App::new()?;
            app.run()?;
        }
        Commands::Test {
            algorithm,
            cutoff,
            windows,
        } => {
            cli::run_test_mode(algorithm, cutoff, windows)?;
        }
        Commands::Benchmark {
            sample_size,
            cutoff,
        } => {
            cli::run_benchmark(sample_size, cutoff)?;
        }
        Commands::CheckUpdates { checksum_file } => {
            cli::check_updates(&checksum_file).await?
        }
    }

    Ok(())
}
