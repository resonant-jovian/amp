use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs},
};

use crate::classification;
use crate::tui::Tui;
use amp_core::api::api;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, DistanceBasedAlgo, GridNearestAlgo, KDTreeSpatialAlgo, OverlappingChunksAlgo,
    RTreeSpatialAlgo, RaycastingAlgo,
};
use amp_core::structs::{AdressClean, CorrelationResult, MiljoeDataClean};

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

/// State per view - keep data when switching tabs
pub struct CorrelateState {
    pub progress: f64,
    pub results: Vec<CorrelationResult>,
    pub output_lines: Vec<String>,
}

pub struct TestState {
    pub output_lines: Vec<String>,
}

pub struct BenchmarkState {
    pub output_lines: Vec<String>,
}

pub struct UpdatesState {
    pub output_lines: Vec<String>,
}

pub struct AppState {
    pub view: View,
    pub selected_tab: usize,
    pub selected_algorithm: AlgorithmChoice,
    pub cutoff: f64,
    pub status_line: String,

    // Per-view state - persists when switching
    pub correlate_state: CorrelateState,
    pub test_state: TestState,
    pub benchmark_state: BenchmarkState,
    pub updates_state: UpdatesState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: View::Dashboard,
            selected_tab: 0,
            selected_algorithm: AlgorithmChoice::KDTree,
            cutoff: 20.0,
            status_line: "Ready".to_string(),
            correlate_state: CorrelateState {
                progress: 0.0,
                results: Vec::new(),
                output_lines: vec!["Ready. Press [Enter] to correlate.".to_string()],
            },
            test_state: TestState {
                output_lines: vec!["Ready. Press [Enter] to run browser test.".to_string()],
            },
            benchmark_state: BenchmarkState {
                output_lines: vec!["Ready. Press [Enter] to benchmark.".to_string()],
            },
            updates_state: UpdatesState {
                output_lines: vec!["Ready. Press [Enter] to check updates.".to_string()],
            },
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
            {
                // Always exit on Ctrl+C
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                    && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
                {
                    break;
                }

                if self.on_key(key)? {
                    break;
                }
            }

            if last_tick.elapsed() >= tick_rate {
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
            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
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
            KeyCode::Char('a') | KeyCode::Char('A') => {
                let idx = AlgorithmChoice::ALL
                    .iter()
                    .position(|a| *a == self.state.selected_algorithm)
                    .unwrap_or(0);
                let next = (idx + 1) % AlgorithmChoice::ALL.len();
                self.state.selected_algorithm = AlgorithmChoice::ALL[next];
                self.state.status_line =
                    format!("Algorithm: {}", self.state.selected_algorithm.label());
            }
            KeyCode::Char('+') => {
                self.state.cutoff += 5.0;
                self.state.status_line = format!("Cutoff: {:.1}m", self.state.cutoff);
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                if self.state.cutoff > 5.0 {
                    self.state.cutoff -= 5.0;
                }
                self.state.status_line = format!("Cutoff: {:.1}m", self.state.cutoff);
            }
            KeyCode::Enter => match self.state.view {
                View::Correlate => {
                    if let Err(e) = self.run_correlation() {
                        self.state.status_line = format!("Error: {}", e);
                    }
                }
                View::Test => {
                    if let Err(e) = self.run_test_mode() {
                        self.state.status_line = format!("Error: {}", e);
                    }
                }
                View::Benchmark => {
                    if let Err(e) = self.run_benchmark() {
                        self.state.status_line = format!("Error: {}", e);
                    }
                }
                View::Updates => {
                    if let Err(e) = self.run_update_check() {
                        self.state.status_line = format!("Error: {}", e);
                    }
                }
                View::Dashboard => {}
            },
            _ => {}
        }

        Ok(false)
    }

    fn draw(&self, frame: &mut Frame) {
        let size = frame.area();

        // Responsive layout - adapts to terminal height
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // tabs
                Constraint::Min(5),    // content (flexible)
                Constraint::Length(1), // status
            ])
            .split(size);

        self.draw_tabs(frame, layout[0]);
        self.draw_body(frame, layout[1]);
        self.draw_status(frame, layout[2]);
    }

    fn draw_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles: Vec<&str> = View::ALL.iter().map(|v| v.title()).collect();
        let tabs = Tabs::new(titles)
            .select(self.state.selected_tab)
            .highlight_style(Style::default().fg(Color::Yellow).bold());
        frame.render_widget(tabs, area);
    }

    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let status =
            Paragraph::new(self.state.status_line.clone()).style(Style::default().fg(Color::Cyan));
        frame.render_widget(status, area);
    }

    fn draw_body(&self, frame: &mut Frame, area: Rect) {
        match self.state.view {
            View::Dashboard => self.draw_dashboard(frame, area),
            View::Correlate => self.draw_correlate(frame, area),
            View::Test => self.draw_test(frame, area),
            View::Benchmark => self.draw_benchmark(frame, area),
            View::Updates => self.draw_updates(frame, area),
        }
    }

    fn draw_dashboard(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" AMP ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let text = "   ___    __  __  ____\n  / _ |  / / / / / __/\n / __ | / /_/ / _\\ \\\
/_/ |_| \\____/ /___/\n\nAddress → Miljözone → Parking\n\nNavigate: [1-5] or [← →]\nControls: [a] Algorithm  [+/-] Cutoff  [Enter] Run\nExit: [q] or [Ctrl+C]";

        let p = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(p, inner);
    }

    fn draw_correlate(&self, frame: &mut Frame, area: Rect) {
        // Responsive constraints based on available height
        let constraints = if area.height > 15 {
            vec![
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Min(5),
            ]
        } else if area.height > 10 {
            vec![
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(3),
            ]
        } else {
            vec![
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(2),
            ]
        };

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Config box
        let config_text = format!(
            "Algorithm: {} | Cutoff: {:.1}m\nPress [Enter]",
            self.state.selected_algorithm.label(),
            self.state.cutoff,
        );
        let config = Paragraph::new(config_text)
            .block(Block::default().borders(Borders::ALL).title(" Config "));
        frame.render_widget(config, sections[0]);

        // Progress bar
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent((self.state.correlate_state.progress * 100.0) as u16);
        frame.render_widget(gauge, sections[1]);

        // Output - persists when switching tabs
        let output_items: Vec<ListItem> = self
            .state
            .correlate_state
            .output_lines
            .iter()
            .take(100) // limit to prevent performance issues
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let results_text = if self.state.correlate_state.results.is_empty() {
            "(no results)".to_string()
        } else {
            format!("({} found)", self.state.correlate_state.results.len())
        };

        let output_list = List::new(output_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Results {} ", results_text)),
        );
        frame.render_widget(output_list, sections[2]);
    }

    fn draw_test(&self, frame: &mut Frame, area: Rect) {
        let constraints = if area.height > 10 {
            vec![Constraint::Length(3), Constraint::Min(4)]
        } else {
            vec![Constraint::Length(2), Constraint::Min(2)]
        };

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Config
        let config_text = format!(
            "Algorithm: {} | Cutoff: {:.1}m | [Enter] to run",
            self.state.selected_algorithm.label(),
            self.state.cutoff,
        );
        let config = Paragraph::new(config_text)
            .block(Block::default().borders(Borders::ALL).title(" Config "));
        frame.render_widget(config, sections[0]);

        // Output (persists)
        let output_items: Vec<ListItem> = self
            .state
            .test_state
            .output_lines
            .iter()
            .take(100)
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let output_list =
            List::new(output_items).block(Block::default().borders(Borders::ALL).title(" Output "));
        frame.render_widget(output_list, sections[1]);
    }

    fn draw_benchmark(&self, frame: &mut Frame, area: Rect) {
        let constraints = if area.height > 10 {
            vec![Constraint::Length(3), Constraint::Min(4)]
        } else {
            vec![Constraint::Length(2), Constraint::Min(2)]
        };

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Config
        let config_text = format!(
            "Cutoff: {:.1}m | [Enter] to benchmark all 6 algorithms",
            self.state.cutoff,
        );
        let config = Paragraph::new(config_text)
            .block(Block::default().borders(Borders::ALL).title(" Config "));
        frame.render_widget(config, sections[0]);

        // Output (persists)
        let output_items: Vec<ListItem> = self
            .state
            .benchmark_state
            .output_lines
            .iter()
            .take(100)
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let output_list =
            List::new(output_items).block(Block::default().borders(Borders::ALL).title(" Output "));
        frame.render_widget(output_list, sections[1]);
    }

    fn draw_updates(&self, frame: &mut Frame, area: Rect) {
        let constraints = if area.height > 10 {
            vec![Constraint::Length(2), Constraint::Min(4)]
        } else {
            vec![Constraint::Length(1), Constraint::Min(2)]
        };

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Config
        let config_text = "[Enter] to check Malmö data portal";
        let config = Paragraph::new(config_text)
            .block(Block::default().borders(Borders::ALL).title(" Config "));
        frame.render_widget(config, sections[0]);

        // Output (persists)
        let output_items: Vec<ListItem> = self
            .state
            .updates_state
            .output_lines
            .iter()
            .take(100)
            .map(|line| ListItem::new(line.as_str()))
            .collect();

        let output_list =
            List::new(output_items).block(Block::default().borders(Borders::ALL).title(" Output "));
        frame.render_widget(output_list, sections[1]);
    }

    fn run_correlation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.correlate_state.progress = 0.0;
        self.state.correlate_state.output_lines.clear();
        self.state
            .correlate_state
            .output_lines
            .push("Starting correlation...".to_string());
        self.state.status_line = "Correlating...".to_string();

        let (addresses, miljodata, parkering): (
            Vec<AdressClean>,
            Vec<MiljoeDataClean>,
            Vec<MiljoeDataClean>,
        ) = api()?;

        let total = addresses.len();
        let mut counter = 0usize;

        let miljo_results = self.correlate_dataset(
            self.state.selected_algorithm,
            &addresses,
            &miljodata,
            self.state.cutoff,
            &mut counter,
            total,
        )?;

        self.state
            .correlate_state
            .output_lines
            .push(format!("Miljodata: {}", miljo_results.len()));

        let parkering_results = self.correlate_dataset(
            self.state.selected_algorithm,
            &addresses,
            &parkering,
            self.state.cutoff,
            &mut counter,
            total,
        )?;

        self.state
            .correlate_state
            .output_lines
            .push(format!("Parkering: {}", parkering_results.len()));

        self.state.correlate_state.results =
            self.merge_results(&addresses, &miljo_results, &parkering_results);
        self.state.correlate_state.progress = 1.0;
        self.state.correlate_state.output_lines.push(format!(
            "Total: {}",
            self.state.correlate_state.results.len()
        ));
        self.state.status_line =
            format!("Done: {} results", self.state.correlate_state.results.len());

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
                        && dist <= cutoff
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate_state.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::Raycasting => {
                let algo = RaycastingAlgo;
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate_state.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::OverlappingChunks => {
                let algo = OverlappingChunksAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate_state.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::RTree => {
                let algo = RTreeSpatialAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate_state.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::KDTree => {
                let algo = KDTreeSpatialAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate_state.progress = *counter as f64 / (total as f64 * 2.0);
                }
            }
            AlgorithmChoice::Grid => {
                let algo = GridNearestAlgo::new(zones);
                for addr in addresses {
                    if let Some((idx, dist)) = algo.correlate(addr, zones)
                        && dist <= cutoff
                    {
                        let info = zones.get(idx).map(|z| z.info.clone()).unwrap_or_default();
                        results.push((addr.adress.clone(), dist, info));
                    }
                    *counter += 1;
                    self.state.correlate_state.progress = *counter as f64 / (total as f64 * 2.0);
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
        self.state.test_state.output_lines.clear();
        self.state
            .test_state
            .output_lines
            .push("Launching browser...".to_string());
        self.state.status_line = "Opening browser...".to_string();

        classification::run_test_mode_legacy(self.state.selected_algorithm, self.state.cutoff)?;

        self.state
            .test_state
            .output_lines
            .push("Complete".to_string());
        self.state.status_line = "Test complete".to_string();
        Ok(())
    }

    fn run_benchmark(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.benchmark_state.output_lines.clear();
        self.state
            .benchmark_state
            .output_lines
            .push("Starting benchmark...".to_string());
        self.state.status_line = "Benchmarking...".to_string();

        classification::run_benchmark_legacy(self.state.cutoff)?;

        self.state
            .benchmark_state
            .output_lines
            .push("Complete".to_string());
        self.state.status_line = "Benchmark done".to_string();
        Ok(())
    }

    fn run_update_check(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state.updates_state.output_lines.clear();
        self.state
            .updates_state
            .output_lines
            .push("Checking updates...".to_string());
        self.state.status_line = "Checking updates...".to_string();

        classification::run_check_updates_legacy()?;

        self.state
            .updates_state
            .output_lines
            .push("Complete".to_string());
        self.state.status_line = "Check complete".to_string();
        Ok(())
    }
}
