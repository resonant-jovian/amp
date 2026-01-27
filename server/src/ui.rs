//! AMP TUI - Professional UI Module
//! Rebuilt with modern Ratatui patterns (v0.30) and professional architecture
//! Inspired by: Slumber, Yozefu
//! Pattern: Elm architecture with component-based design
//! Features: Light/Dark mode, Vertical layout, Ctrl+C exit

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
};

use crate::classification;
use crate::tui::Tui;
use amp_core::api::api;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo,
    OverlappingChunksAlgo, RaycastingAlgo, RTreeSpatialAlgo,
};
use amp_core::structs::{AdressClean, CorrelationResult, MiljoeDataClean};

const AMP_LOGO: &str = r#"
  _   _   ___
 / \ / \ / _ \
|  _   | / | | |
| (_) || \ |_| /
 \___/  \___/
"#;

/// Color theme for light/dark modes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTheme {
    Light,
    Dark,
}

impl ColorTheme {
    /// Auto-detect theme based on terminal background
    pub fn auto() -> Self {
        // Default to dark for now, can be enhanced with terminal detection
        ColorTheme::Dark
    }

    pub fn text_color(&self) -> Color {
        match self {
            ColorTheme::Light => Color::Black,
            ColorTheme::Dark => Color::White,
        }
    }

    pub fn bg_color(&self) -> Color {
        match self {
            ColorTheme::Light => Color::White,
            ColorTheme::Dark => Color::Black,
        }
    }

    pub fn alt_text_color(&self) -> Color {
        match self {
            ColorTheme::Light => Color::DarkGray,
            ColorTheme::Dark => Color::Gray,
        }
    }

    pub fn header_style(&self) -> Style {
        match self {
            ColorTheme::Light => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            ColorTheme::Dark => Style::default().fg(Color::Cyan),
        }
    }
}

/// Algorithm enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Algorithm {
    KDTree,
    RTree,
    Grid,
    DistanceBased,
    Raycasting,
    OverlappingChunks,
}

impl Algorithm {
    pub const ALL: &'static [Algorithm] = &[
        Algorithm::KDTree,
        Algorithm::RTree,
        Algorithm::Grid,
        Algorithm::DistanceBased,
        Algorithm::Raycasting,
        Algorithm::OverlappingChunks,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::KDTree => "KD-Tree",
            Algorithm::RTree => "R-Tree",
            Algorithm::Grid => "Grid",
            Algorithm::DistanceBased => "Distance-Based",
            Algorithm::Raycasting => "Raycasting",
            Algorithm::OverlappingChunks => "Overlapping Chunks",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Algorithm::KDTree => "Fast k-dimensional tree partitioning",
            Algorithm::RTree => "Efficient rectangle-based indexing",
            Algorithm::Grid => "Regular grid approximation",
            Algorithm::DistanceBased => "Brute force distance check",
            Algorithm::Raycasting => "Polygon containment testing",
            Algorithm::OverlappingChunks => "Advanced chunk partitioning",
        }
    }
}

/// View enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Correlate,
    Results,
    Benchmark,
    Updates,
}

impl View {
    pub const ALL: &'static [View] = &[
        View::Dashboard,
        View::Correlate,
        View::Results,
        View::Benchmark,
        View::Updates,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Correlate => "Correlate",
            View::Results => "Results",
            View::Benchmark => "Benchmark",
            View::Updates => "Updates",
        }
    }
}

type CorrelationTuple = (String, f64, String);

/// Per-view state
pub struct DashboardState {
    #[allow(dead_code)]
    scroll_offset: u16,
}

pub struct CorrelateState {
    running: bool,
    progress: f64,
    #[allow(dead_code)]
    status_msg: String,
    details: Vec<String>,
}

pub struct ResultsState {
    results: Vec<CorrelationResult>,
    #[allow(dead_code)]
    scroll_offset: usize,
    #[allow(dead_code)]
    selected_idx: Option<usize>,
}

pub struct BenchmarkState {
    running: bool,
    results: Vec<(String, Duration, Duration)>,
    output: Vec<String>,
}

pub struct UpdatesState {
    last_check: Option<Instant>,
    status: String,
}

/// Global application state
pub struct AppState {
    pub current_view: View,
    pub current_algorithm: Algorithm,
    pub cutoff_distance: f64,
    pub should_quit: bool,
    pub theme: ColorTheme,

    // Per-view states
    #[allow(dead_code)]
    dashboard: DashboardState,
    correlate: CorrelateState,
    results: ResultsState,
    benchmark: BenchmarkState,
    updates: UpdatesState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_view: View::Dashboard,
            current_algorithm: Algorithm::KDTree,
            cutoff_distance: 20.0,
            should_quit: false,
            theme: ColorTheme::auto(),
            dashboard: DashboardState { scroll_offset: 0 },
            correlate: CorrelateState {
                running: false,
                progress: 0.0,
                status_msg: "Ready. Press [Enter] to start.".to_string(),
                details: Vec::new(),
            },
            results: ResultsState {
                results: Vec::new(),
                scroll_offset: 0,
                selected_idx: None,
            },
            benchmark: BenchmarkState {
                running: false,
                results: Vec::new(),
                output: vec!["Benchmarks available: KD-Tree, R-Tree, Grid, Distance, Raycasting, Chunks".to_string()],
            },
            updates: UpdatesState {
                last_check: None,
                status: "Ready. Press [Enter] to check.".to_string(),
            },
        }
    }
}

/// Main application wrapper
pub struct App {
    tui: Tui,
    state: AppState,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tui = Tui::new()?;
        Ok(Self {
            tui,
            state: AppState::default(),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tui = std::mem::replace(&mut self.tui, Tui::new()?);
        tui.enter()?;

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        loop {
            tui.terminal.draw(|f| self.render(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    self.handle_key(key)?;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            if self.state.should_quit {
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    fn render(&self, f: &mut Frame) {
        let area = f.area();

        // Optimize for small/vertical screens: reduce header to 1 line if needed
        let header_height = if area.height < 20 { 1 } else { 2 };
        let footer_height = 1;
        let content_height = area.height.saturating_sub(header_height + footer_height) as u16;

        // Main layout: header | content | footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(content_height),
                Constraint::Length(footer_height),
            ])
            .split(area);

        self.render_header(f, chunks[0]);
        self.render_content(f, chunks[1]);
        self.render_footer(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        // Tab bar
        let titles: Vec<&str> = View::ALL.iter().map(|v| v.title()).collect();
        let current_idx = View::ALL
            .iter()
            .position(|v| *v == self.state.current_view)
            .unwrap_or(0);

        let tabs = Tabs::new(titles)
            .select(current_idx)
            .style(Style::default().fg(self.state.theme.alt_text_color()))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
            .divider("│");

        f.render_widget(tabs, area);
    }

    fn render_content(&self, f: &mut Frame, area: Rect) {
        match self.state.current_view {
            View::Dashboard => self.render_dashboard(f, area),
            View::Correlate => self.render_correlate(f, area),
            View::Results => self.render_results(f, area),
            View::Benchmark => self.render_benchmark(f, area),
            View::Updates => self.render_updates(f, area),
        }
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(" 📊 AMP Dashboard ")
            .title_alignment(Alignment::Center)
            .style(theme.header_style());

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create content lines - using proper spacing instead of offsets
        let mut lines = vec![
            Line::from(AMP_LOGO),
        ];

        // Add spacing
        lines.push(Line::from(""));

        // Title
        lines.push(Line::from(vec![
            Span::styled("Address Parking Mapper", Style::default().fg(theme.text_color()).add_modifier(Modifier::BOLD)),
        ]));

        // Description
        lines.push(Line::from(vec![
            Span::raw("Correlate addresses with parking zones using "),
            Span::styled("spatial algorithms", Style::default().fg(Color::Yellow)),
        ]));

        // Spacing
        lines.push(Line::from(""));

        // Stats section
        lines.push(Line::from(vec![
            Span::styled("📋 Quick Stats:", Style::default().fg(theme.text_color()).add_modifier(Modifier::BOLD)),
        ]));

        // Algorithm line
        lines.push(Line::from(vec![
            Span::raw(format!("  • Algorithm: {}", self.state.current_algorithm.name())).fg(theme.text_color()),
        ]));

        // Cutoff line
        lines.push(Line::from(vec![
            Span::raw(format!("  • Cutoff: {:.1}m", self.state.cutoff_distance)).fg(theme.text_color()),
        ]));

        // Spacing
        lines.push(Line::from(""));

        // Navigation section
        lines.push(Line::from(vec![
            Span::styled("⌨️ Navigation:", Style::default().fg(theme.text_color()).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  [1-5] Jump | [←→] Tab | [a] Algorithm | [+/-] Distance").fg(theme.text_color()),
        ]));

        // Spacing
        lines.push(Line::from(""));

        // Exit section
        lines.push(Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Run | "),
            Span::styled("[Ctrl+C]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Exit"),
        ]));

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme.text_color()));

        f.render_widget(paragraph, inner);
    }

    fn render_correlate(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;

        // Layout: config | progress | details
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),  // config section
                Constraint::Percentage(20),  // progress
                Constraint::Percentage(40),  // details
            ])
            .split(area);

        // Config box
        self.render_algorithm_selector(f, chunks[0]);

        // Progress bar
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" Progress "),
            )
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent((self.state.correlate.progress * 100.0) as u16)
            .label(format!("{:.0}%", self.state.correlate.progress * 100.0));

        f.render_widget(gauge, chunks[1]);

        // Details list
        let items: Vec<ListItem> = self
            .state
            .correlate
            .details
            .iter()
            .map(|line| ListItem::new(line.as_str()).style(Style::default().fg(theme.text_color())))
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(" Details ")
                .title_alignment(Alignment::Left),
        );

        f.render_widget(list, chunks[2]);
    }

    fn render_algorithm_selector(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(" ⚙️  Configuration ")
            .style(theme.header_style());

        let inner = block.inner(area);
        f.render_widget(block, area);

        // Create algorithm grid
        let rows: Vec<Row> = Algorithm::ALL
            .iter()
            .map(|algo| {
                let is_selected = *algo == self.state.current_algorithm;
                let style = if is_selected {
                    Style::default()
                        .fg(theme.bg_color())
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_color())
                };

                let check = if is_selected { "✓" } else { " " };
                Row::new(vec![
                    format!("{} {}", check, algo.name()),
                    algo.description().to_string(),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(35),
                Constraint::Percentage(65),
            ],
        )
        .style(Style::default().fg(theme.text_color()));

        f.render_widget(table, inner);
    }

    fn render_results(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;
        let result_count = self.state.results.results.len();

        if result_count == 0 {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(" 📊 Results (0 found) ")
                .style(Style::default().fg(theme.alt_text_color()));

            let para = Paragraph::new("No results. Run correlation first (Tab 2)")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.text_color()))
                .block(block);

            f.render_widget(para, area);
            return;
        }

        // Results table
        let rows: Vec<Row> = self
            .state
            .results
            .results
            .iter()
            .take(100) // Allow more rows on vertical screens
            .map(|result| {
                Row::new(vec![
                    result.address.clone(),
                    format!(
                        "{:.1}m",
                        result.miljo_match.as_ref().map(|(d, _)| d).copied().unwrap_or(999.0)
                    ),
                    format!(
                        "{:.1}m",
                        result.parkering_match.as_ref().map(|(d, _)| d).copied().unwrap_or(999.0)
                    ),
                ])
                .style(Style::default().fg(theme.text_color()))
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["Address", "Miljö (m)", "Parkering (m)"])
                .style(theme.header_style())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(format!(" 📊 Results ({} found) ", result_count)),
        );

        f.render_widget(table, area);
    }

    fn render_benchmark(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Controls
                Constraint::Percentage(80), // Results table
            ])
            .split(area);

        // Controls
        let controls = Paragraph::new("Press [Enter] to benchmark all 6 algorithms")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" 🎯 Controls "),
            )
            .style(Style::default().fg(theme.text_color()))
            .alignment(Alignment::Center);

        f.render_widget(controls, chunks[0]);

        // Results
        if self.state.benchmark.results.is_empty() {
            let msg = Paragraph::new("No benchmark results yet")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .title(" ⚡ Performance "),
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.alt_text_color()));

            f.render_widget(msg, chunks[1]);
        } else {
            let rows: Vec<Row> = self
                .state
                .benchmark
                .results
                .iter()
                .map(|(name, total, avg)| {
                    Row::new(vec![
                        name.clone(),
                        format!("{}ms", total.as_millis()),
                        format!("{}μs", avg.as_micros()),
                    ])
                    .style(Style::default().fg(theme.text_color()))
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ],
            )
            .header(
                Row::new(vec!["Algorithm", "Total Time", "Per Address"])
                    .style(theme.header_style())
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" ⚡ Performance Results "),
            );

            f.render_widget(table, chunks[1]);
        }
    }

    fn render_updates(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Controls
                Constraint::Percentage(80), // Status
            ])
            .split(area);

        // Controls
        let controls = Paragraph::new("Press [Enter] to check Malmö data portal")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" 🔍 Data Portal "),
            )
            .style(Style::default().fg(theme.text_color()))
            .alignment(Alignment::Center);

        f.render_widget(controls, chunks[0]);

        // Status
        let mut status_lines = Vec::new();

        if let Some(last) = self.state.updates.last_check {
            status_lines.push(Line::from(format!(
                "Last check: {:.1}s ago",
                last.elapsed().as_secs_f64()
            )));
            status_lines.push(Line::from(""));
        }

        status_lines.push(Line::from(self.state.updates.status.clone()));

        let status = Paragraph::new(status_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title(" ✓ Status "),
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme.text_color()));

        f.render_widget(status, chunks[1]);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let theme = self.state.theme;
        let status_text = format!(
            " {} | Cutoff: {:.1}m | Ctrl+C to Exit ",
            self.state.current_algorithm.name(),
            self.state.cutoff_distance
        );

        let footer = Paragraph::new(status_text)
            .style(Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray))
            .alignment(Alignment::Left);

        f.render_widget(footer, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            // Ctrl+C always exits
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.should_quit = true;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => self.state.should_quit = true,
            KeyCode::Char('1') => self.state.current_view = View::Dashboard,
            KeyCode::Char('2') => self.state.current_view = View::Correlate,
            KeyCode::Char('3') => self.state.current_view = View::Results,
            KeyCode::Char('4') => self.state.current_view = View::Benchmark,
            KeyCode::Char('5') => self.state.current_view = View::Updates,
            KeyCode::Left => self.navigate_tabs(-1),
            KeyCode::Right => self.navigate_tabs(1),
            KeyCode::Char('a') | KeyCode::Char('A') => self.cycle_algorithm(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.adjust_cutoff(5.0),
            KeyCode::Char('-') | KeyCode::Char('_') => self.adjust_cutoff(-5.0),
            KeyCode::Enter => self.execute_action()?,
            _ => {}
        }

        Ok(())
    }

    fn navigate_tabs(&mut self, direction: i32) {
        let current_idx = View::ALL
            .iter()
            .position(|v| *v == self.state.current_view)
            .unwrap_or(0) as i32;

        let new_idx = (current_idx + direction).clamp(0, View::ALL.len() as i32 - 1) as usize;
        self.state.current_view = View::ALL[new_idx];
    }

    fn cycle_algorithm(&mut self) {
        let current_idx = Algorithm::ALL
            .iter()
            .position(|a| *a == self.state.current_algorithm)
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % Algorithm::ALL.len();
        self.state.current_algorithm = Algorithm::ALL[next_idx];
    }

    fn adjust_cutoff(&mut self, delta: f64) {
        self.state.cutoff_distance = (self.state.cutoff_distance + delta).max(5.0);
    }

    fn execute_action(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.state.current_view {
            View::Dashboard => {}
            View::Correlate => self.run_correlation()?,
            View::Results => {}
            View::Benchmark => self.run_benchmark()?,
            View::Updates => self.run_updates()?,
        }
        Ok(())
    }

    fn run_correlation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.correlate.running = true;
        self.state.correlate.progress = 0.0;
        self.state.correlate.details.clear();
        self.state.correlate.details.push("Loading data...".to_string());

        let (addresses, miljodata, parkering): (
            Vec<AdressClean>,
            Vec<MiljoeDataClean>,
            Vec<MiljoeDataClean>,
        ) = api()?;

        self.state.correlate.details.push(format!(
            "Loaded {} addresses, {} miljö zones, {} parkering zones",
            addresses.len(),
            miljodata.len(),
            parkering.len()
        ));

        let total = addresses.len();
        let mut counter = 0usize;

        // Convert to old-style algorithm for compatibility
        let algo_choice = match self.state.current_algorithm {
            Algorithm::KDTree => crate::ui::AlgorithmChoice::KDTree,
            Algorithm::RTree => crate::ui::AlgorithmChoice::RTree,
            Algorithm::Grid => crate::ui::AlgorithmChoice::Grid,
            Algorithm::DistanceBased => crate::ui::AlgorithmChoice::DistanceBased,
            Algorithm::Raycasting => crate::ui::AlgorithmChoice::Raycasting,
            Algorithm::OverlappingChunks => crate::ui::AlgorithmChoice::OverlappingChunks,
        };

        let miljo_results = self.correlate_dataset(
            algo_choice,
            &addresses,
            &miljodata,
            &mut counter,
            total,
        )?;

        self.state
            .correlate
            .details
            .push(format!("Miljödata matches: {}", miljo_results.len()));

        let parkering_results = self.correlate_dataset(
            algo_choice,
            &addresses,
            &parkering,
            &mut counter,
            total,
        )?;

        self.state
            .correlate
            .details
            .push(format!("Parkering matches: {}", parkering_results.len()));

        self.state.results.results =
            self.merge_results(&addresses, &miljo_results, &parkering_results);

        self.state.correlate.progress = 1.0;
        self.state.correlate.details.push(format!(
            "Correlation complete! {} total matches",
            self.state.results.results.len()
        ));
        self.state.correlate.running = false;

        // Switch to results view
        self.state.current_view = View::Results;

        Ok(())
    }

    fn correlate_dataset(
        &mut self,
        algorithm: AlgorithmChoice,
        addresses: &[AdressClean],
        zones: &[MiljoeDataClean],
        counter: &mut usize,
        total: usize,
    ) -> Result<Vec<CorrelationTuple>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        match algorithm {
            AlgorithmChoice::DistanceBased => {
                let algo = DistanceBasedAlgo;
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= self.state.cutoff_distance
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::Raycasting => {
                let algo = RaycastingAlgo;
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= self.state.cutoff_distance
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::OverlappingChunks => {
                let algo = OverlappingChunksAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= self.state.cutoff_distance
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::RTree => {
                let algo = RTreeSpatialAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= self.state.cutoff_distance
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::KDTree => {
                let algo = KDTreeSpatialAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= self.state.cutoff_distance
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::Grid => {
                let algo = GridNearestAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= self.state.cutoff_distance
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
        }

        Ok(results)
    }

    fn merge_results(
        &self,
        addresses: &[AdressClean],
        miljo_results: &[CorrelationTuple],
        parkering_results: &[CorrelationTuple],
    ) -> Vec<CorrelationResult> {
        let miljo_map: HashMap<_, _> = miljo_results
            .iter()
            .map(|(addr, dist, info)| (addr.clone(), (*dist, info.clone())))
            .collect();

        let parkering_map: HashMap<_, _> = parkering_results
            .iter()
            .map(|(addr, dist, info)| (addr.clone(), (*dist, info.clone())))
            .collect();

        addresses
            .iter()
            .map(|addr| {
                let miljo_match = miljo_map.get(&addr.adress).map(|(d, i)| (*d, i.clone()));
                let parkering_match = parkering_map
                    .get(&addr.adress)
                    .map(|(d, i)| (*d, i.clone()));

                CorrelationResult {
                    address: addr.adress.clone(),
                    postnummer: addr.postnummer.clone(),
                    miljo_match,
                    parkering_match,
                }
            })
            .collect()
    }

    fn run_benchmark(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.benchmark.running = true;
        self.state.benchmark.results.clear();
        self.state.benchmark.output.clear();
        self.state
            .benchmark
            .output
            .push("Benchmarking all algorithms...".to_string());

        classification::run_benchmark_legacy(self.state.cutoff_distance)?;

        self.state
            .benchmark
            .output
            .push("Benchmark complete!".to_string());
        self.state.benchmark.running = false;

        Ok(())
    }

    fn run_updates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.updates.last_check = Some(Instant::now());
        self.state.updates.status = "Checking Malmö data portal...".to_string();

        classification::run_check_updates_legacy()?;

        self.state.updates.status = "Update check complete!".to_string();

        Ok(())
    }
}

// Keep old-style enum for compatibility with existing code
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlgorithmChoice {
    DistanceBased,
    Raycasting,
    OverlappingChunks,
    RTree,
    KDTree,
    Grid,
}
