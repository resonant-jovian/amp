//! AMP Server - Address-Parking Correlation Server
//! Interactive Ratatui TUI application for correlation, testing, and benchmarking

mod app;
mod classification;
mod tui;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Launch interactive Ratatui TUI
    let mut app = ui::App::new()?;
    app.run()?;
    Ok(())
}
