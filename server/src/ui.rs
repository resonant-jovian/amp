use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Tabs},
};

use crate::classification;
use crate::tui::Tui;
use amp_core::api::api;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo, OverlappingChunksAlgo,
    RTreeSpatialAlgo, RaycastingAlgo,
};
use amp_core::structs::{AdressClean, CorrelationResult, MiljoeDataClean};

/// Type alias for correlation result tuples: (address, distance_meters, zone_info)
type CorrelationTuple = (String, f64, String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlgorithmChoice {
    DistanceBased,
    Raycasting,
    OverlappingChunks,
    RTree,
    KDTree,
    Grid,
}

impl AlgorithmChoice {
    pub const ALL: &'static [AlgorithmChoice] = &[
        AlgorithmChoice::KDTree,
        AlgorithmChoice::RTree,
        AlgorithmChoice::Grid,
        AlgorithmChoice::DistanceBased,
        AlgorithmChoice::Raycasting,
        AlgorithmChoice::OverlappingChunks,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AlgorithmChoice::DistanceBased => "Distance-Based",
            AlgorithmChoice::Raycasting => "Raycasting",
            AlgorithmChoice::OverlappingChunks => "Overlapping Chunks",
            AlgorithmChoice::RTree => "R-Tree",
            AlgorithmChoice::KDTree => "KD-Tree",
            AlgorithmChoice::Grid => "Grid",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Correlate,
    Test,
    Benchmark,
    Updates,
}

impl View {
    pub const ALL: &'static [View] = &[
        View::Dashboard,
        View::Correlate,
        View::Test,
        View::Benchmark,
        View::Updates,
    ];

    pub fn title(self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Correlate => "Correlate",
            View::Test => "Test (Browser)",
            View::Benchmark => "Benchmark",
            View::Updates => "Check Updates",
        }
    }
}

pub struct AppState {
    pub view: View,
    pub selected_tab: usize,

    pub selected_algorithm: AlgorithmChoice,
    pub cutoff: f64,

    pub is_running: bool,
    pub progress: f64,

    pub correlation_results: Vec<CorrelationResult>,

    pub last_action: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: View::Dashboard,
            selected_tab: 0,
            selected_algorithm: AlgorithmChoice::KDTree,
            cutoff: 20.0,
            is_running: false,
            progress: 0.0,
            correlation_results: Vec::new(),
            last_action: "Ready. [1-5] to navigate".to_string(),
        }
    }
}

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
            tui.terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)?
                && let Event::Key(key) = crossterm::event::read()?
                    && self.on_key(key)? {
                        break;
                    }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick()?;
                last_tick = Instant::now();
            }
        }

        tui.exit()?;
        Ok(())
    }

    fn on_key(&mut self, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Left => {
                if self.state.selected_tab > 0 {
                    self.state.selected_tab -= 1;
                    self.state.view = View::ALL[self.state.selected_tab];
                }
            }
            KeyCode::Right => {
                if self.state.selected_tab < View::ALL.len() - 1 {
                    self.state.selected_tab += 1;
                    self.state.view = View::ALL[self.state.selected_tab];
                }
            }
            KeyCode::Char('1') => {
                self.state.selected_tab = 0;
                self.state.view = View::Dashboard;
            }
            KeyCode::Char('2') => {
                self.state.selected_tab = 1;
                self.state.view = View::Correlate;
            }
            KeyCode::Char('3') => {
                self.state.selected_tab = 2;
                self.state.view = View::Test;
            }
            KeyCode::Char('4') => {
                self.state.selected_tab = 3;
                self.state.view = View::Benchmark;
            }
            KeyCode::Char('5') => {
                self.state.selected_tab = 4;
                self.state.view = View::Updates;
            }
            KeyCode::Char('a') => {
                let idx = AlgorithmChoice::ALL
                    .iter()
                    .position(|a| *a == self.state.selected_algorithm)
                    .unwrap_or(0);
                let next = (idx + 1) % AlgorithmChoice::ALL.len();
                self.state.selected_algorithm = AlgorithmChoice::ALL[next];
                self.state.last_action = format!(
                    "Algorithm: {}",
                    self.state.selected_algorithm.label()
                );
            }
            KeyCode::Char('+') => {
                self.state.cutoff += 5.0;
                self.state.last_action = format!("Cutoff: {:.1}m", self.state.cutoff);
            }
            KeyCode::Char('-') => {
                if self.state.cutoff > 5.0 {
                    self.state.cutoff -= 5.0;
                    self.state.last_action = format!("Cutoff: {:.1}m", self.state.cutoff);
                }
            }
            KeyCode::Enter => match self.state.view {
                View::Correlate => {
                    self.run_correlation()?;
                }
                View::Test => {
                    self.run_test_mode()?;
                }
                View::Benchmark => {
                    self.run_benchmark()?;
                }
                View::Updates => {
                    self.run_update_check()?;
                }
                View::Dashboard => {}
            },
            _ => {}
        }

        Ok(false)
    }

    fn on_tick(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(size);

        self.draw_header(frame, main_layout[0]);
        self.draw_body(frame, main_layout[1]);
        self.draw_footer(frame, main_layout[2]);
    }

    fn draw_header(&self, frame: &mut ratatui::Frame, area: Rect) {
        let tabs: Vec<&str> = View::ALL.iter().map(|v| v.title()).collect();

        let tabs_widget = Tabs::new(tabs)
            .block(Block::default().borders(Borders::BOTTOM))
            .highlight_style(Style::default().fg(Color::Yellow).bold())
            .select(self.state.selected_tab);

        frame.render_widget(tabs_widget, area);
    }

    fn draw_body(&self, frame: &mut ratatui::Frame, area: Rect) {
        match self.state.view {
            View::Dashboard => self.draw_dashboard(frame, area),
            View::Correlate => self.draw_correlate(frame, area),
            View::Test => self.draw_test(frame, area),
            View::Benchmark => self.draw_benchmark(frame, area),
            View::Updates => self.draw_updates(frame, area),
        }
    }

    fn draw_footer(&self, frame: &mut ratatui::Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let help = Paragraph::new("[←/→] Tab  [1-5] Jump  [a] Algo  [+/-] Cut  [↵] Run  [q] Quit")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(help, layout[0]);

        let status = Paragraph::new(self.state.last_action.clone())
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::LEFT));
        frame.render_widget(status, layout[1]);
    }

    fn draw_dashboard(&self, frame: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(3)])
            .split(area);

        let art = Paragraph::new(
            "   ___    __  __  ____\n  / _ |  / / / / / __/\n / __ | / /_/ / _\\ \\\
/_/ |_| \\____/ /___/\n\nAddress → Miljözone → Parking"
        )
        .block(Block::default().borders(Borders::ALL).title(" amp-server "))
        .alignment(Alignment::Center);
        frame.render_widget(art, chunks[0]);

        let info = Paragraph::new(
            "Select a tab to correlate addresses, run browser tests, benchmark algorithms, or check for Malmö data updates."
        )
        .block(Block::default().borders(Borders::ALL).title(" Getting Started "))
        .wrap(ratatui::text::Wrap { trim: true });
        frame.render_widget(info, chunks[1]);
    }

    fn draw_correlate(&self, frame: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(2), Constraint::Min(5)])
            .split(area);

        let config = Paragraph::new(format!(
            "Algorithm: {} | Cutoff: {:.1}m | Press [Enter] to start",
            self.state.selected_algorithm.label(),
            self.state.cutoff,
        ))
        .block(Block::default().borders(Borders::ALL).title(" Configuration "));
        frame.render_widget(config, chunks[0]);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent((self.state.progress * 100.0) as u16)
            .label(format!("{:.0}%", self.state.progress * 100.0));
        frame.render_widget(gauge, chunks[1]);

        let results_text = if self.state.correlation_results.is_empty() {
            "No results yet".to_string()
        } else {
            format!("Found {} correlations", self.state.correlation_results.len())
        };

        let results = Paragraph::new(results_text)
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(results, chunks[2]);
    }

    fn draw_test(&self, frame: &mut ratatui::Frame, area: Rect) {
        let p = Paragraph::new(format!(
            "Algorithm: {}\nCutoff: {:.1}m\n\nPress [Enter] to launch browser-based testing (opens browser windows).",
            self.state.selected_algorithm.label(),
            self.state.cutoff,
        ))
        .block(Block::default().borders(Borders::ALL).title(" Browser Test Mode "))
        .wrap(ratatui::text::Wrap { trim: true });
        frame.render_widget(p, area);
    }

    fn draw_benchmark(&self, frame: &mut ratatui::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        let config = Paragraph::new(format!(
            "Cutoff: {:.1}m | Press [Enter] to benchmark all 6 algorithms",
            self.state.cutoff,
        ))
        .block(Block::default().borders(Borders::ALL).title(" Benchmark "));
        frame.render_widget(config, chunks[0]);

        let block = Paragraph::new("Results will appear in console output")
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(block, chunks[1]);
    }

    fn draw_updates(&self, frame: &mut ratatui::Frame, area: Rect) {
        let p = Paragraph::new(
            "Press [Enter] to check Malmö open data portal for updates to addresses, environmental zones, and parking regulations."
        )
        .block(Block::default().borders(Borders::ALL).title(" Data Updates "))
        .wrap(ratatui::text::Wrap { trim: true });
        frame.render_widget(p, area);
    }

    fn run_correlation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.is_running = true;
        self.state.progress = 0.0;
        self.state.last_action = "Running correlation...".to_string();

        let (addresses, miljodata, parkering): (
            Vec<AdressClean>,
            Vec<MiljoeDataClean>,
            Vec<MiljoeDataClean>,
        ) = api()?;

        let cutoff = self.state.cutoff;
        let algorithm = self.state.selected_algorithm;

        let total = addresses.len();
        let mut counter = 0usize;

        let miljo_results = self.correlate_dataset(
            algorithm,
            &addresses,
            &miljodata,
            cutoff,
            &mut counter,
            total,
        )?;
        let parkering_results = self.correlate_dataset(
            algorithm,
            &addresses,
            &parkering,
            cutoff,
            &mut counter,
            total,
        )?;

        self.state.correlation_results =
            self.merge_results(&addresses, &miljo_results, &parkering_results);
        self.state.is_running = false;
        self.state.progress = 1.0;
        self.state.last_action = format!("Correlation complete: {} matches", self.state.correlation_results.len());

        Ok(())
    }

    fn correlate_dataset(
        &mut self,
        algorithm: AlgorithmChoice,
        addresses: &[AdressClean],
        zones: &[MiljoeDataClean],
        cutoff: f64,
        counter: &mut usize,
        total: usize,
    ) -> Result<Vec<CorrelationTuple>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        match algorithm {
            AlgorithmChoice::DistanceBased => {
                let algo = DistanceBasedAlgo;
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff {
                            let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                            results.push((addr.adress.clone(), dist, info));
                        }
                    *counter += 1;
                    self.state.progress = *counter as f64 / total as f64;
                }
            }
            AlgorithmChoice::Raycasting => {
                let algo = RaycastingAlgo;
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff {
                            let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                            results.push((addr.adress.clone(), dist, info));
                        }
                    *counter += 1;
                    self.state.progress = *counter as f64 / total as f64;
                }
            }
            AlgorithmChoice::OverlappingChunks => {
                let algo = OverlappingChunksAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff {
                            let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                            results.push((addr.adress.clone(), dist, info));
                        }
                    *counter += 1;
                    self.state.progress = *counter as f64 / total as f64;
                }
            }
            AlgorithmChoice::RTree => {
                let algo = RTreeSpatialAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff {
                            let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                            results.push((addr.adress.clone(), dist, info));
                        }
                    *counter += 1;
                    self.state.progress = *counter as f64 / total as f64;
                }
            }
            AlgorithmChoice::KDTree => {
                let algo = KDTreeSpatialAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff {
                            let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                            results.push((addr.adress.clone(), dist, info));
                        }
                    *counter += 1;
                    self.state.progress = *counter as f64 / total as f64;
                }
            }
            AlgorithmChoice::Grid => {
                let algo = GridNearestAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff {
                            let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                            results.push((addr.adress.clone(), dist, info));
                        }
                    *counter += 1;
                    self.state.progress = *counter as f64 / total as f64;
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
        use std::collections::HashMap;

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

    fn run_test_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.last_action =
            "Launching browser-based test mode (see external windows)...".into();
        classification::run_test_mode_legacy(self.state.selected_algorithm, self.state.cutoff)?;
        Ok(())
    }

    fn run_benchmark(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.last_action = "Running benchmark in external logging (see stdout)...".into();
        classification::run_benchmark_legacy(self.state.cutoff)?;
        Ok(())
    }

    fn run_update_check(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.last_action = "Checking remote data for updates...".into();
        classification::run_check_updates_legacy()?;
        Ok(())
    }
}
