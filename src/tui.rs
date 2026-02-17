use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, Paragraph, Tabs, Wrap,
        canvas::{Canvas, Line as CanvasLine, Shape},
    },
};
use std::{error::Error, io, time::Duration};
use tokio::sync::mpsc;

use crate::controller::{Color as LightColor, Controller};

#[derive(Clone, Copy, PartialEq)]
enum ActiveTab {
    Color,
    Pattern,
    Mic,
}

struct App {
    // State
    power: bool,
    color: LightColor,
    pattern: u8,
    mic_sensitivity: u8,

    // UI State
    active_tab: ActiveTab,

    // Color Tab Selection
    color_selection: usize, // 0: R, 1: G, 2: B

    // Communication
    tx: mpsc::Sender<[u8; 9]>,
}

impl App {
    fn new(tx: mpsc::Sender<[u8; 9]>) -> Self {
        Self {
            power: true,
            color: LightColor {
                r: 255,
                g: 255,
                b: 0,
            }, // Batmobile Yellow default
            pattern: 0,
            mic_sensitivity: 0,
            active_tab: ActiveTab::Color,
            color_selection: 0,
            tx,
        }
    }

    async fn send_command(&self, payload: [u8; 9]) {
        if let Err(e) = self.tx.send(payload).await {
            eprintln!("Error sending command: {}", e);
        }
    }

    async fn toggle_power(&mut self) {
        self.power = !self.power;
        self.send_command(Controller::power(self.power)).await;
    }

    async fn set_color(&mut self) {
        self.send_command(Controller::color(LightColor {
            r: self.color.r,
            g: self.color.g,
            b: self.color.b,
        }))
        .await;
    }

    async fn set_pattern(&mut self) {
        self.send_command(Controller::pattern(self.pattern)).await;
    }

    async fn set_mic(&mut self) {
        self.send_command(Controller::mic(self.mic_sensitivity))
            .await;
    }

    pub async fn on_key(&mut self, key: KeyEvent) -> bool {
        match self.active_tab {
            ActiveTab::Color | ActiveTab::Pattern | ActiveTab::Mic => {
                match key.code {
                    KeyCode::Tab | KeyCode::BackTab => {
                        let forward = key.code == KeyCode::Tab;
                        self.active_tab = match self.active_tab {
                            ActiveTab::Color => {
                                if forward {
                                    ActiveTab::Pattern
                                } else {
                                    ActiveTab::Mic
                                }
                            }
                            ActiveTab::Pattern => {
                                if forward {
                                    ActiveTab::Mic
                                } else {
                                    ActiveTab::Color
                                }
                            }
                            ActiveTab::Mic => {
                                if forward {
                                    ActiveTab::Color
                                } else {
                                    ActiveTab::Pattern
                                }
                            }
                        };
                    }
                    KeyCode::Char('q') => return true,
                    KeyCode::Char('p') => self.toggle_power().await,

                    // Tab specific inputs
                    _ => match self.active_tab {
                        ActiveTab::Color => self.handle_color_input(key.code).await,
                        ActiveTab::Pattern => self.handle_pattern_input(key.code).await,
                        ActiveTab::Mic => self.handle_mic_input(key.code).await,
                    },
                }
            }
        }
        false
    }

    async fn handle_color_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('1') => self.color_selection = 0,
            KeyCode::Char('2') => self.color_selection = 1,
            KeyCode::Char('3') => self.color_selection = 2,
            KeyCode::Up | KeyCode::Char('k') => {
                match self.color_selection {
                    0 => self.color.r = self.color.r.saturating_add(5),
                    1 => self.color.g = self.color.g.saturating_add(5),
                    2 => self.color.b = self.color.b.saturating_add(5),
                    _ => {}
                }
                self.set_color().await;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.color_selection {
                    0 => self.color.r = self.color.r.saturating_sub(5),
                    1 => self.color.g = self.color.g.saturating_sub(5),
                    2 => self.color.b = self.color.b.saturating_sub(5),
                    _ => {}
                }
                self.set_color().await;
            }
            _ => {}
        }
    }

    async fn handle_pattern_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.pattern = self.pattern.saturating_add(1);
                self.set_pattern().await;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.pattern = self.pattern.saturating_sub(1);
                self.set_pattern().await;
            }
            _ => {}
        }
    }

    async fn handle_mic_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.mic_sensitivity = self.mic_sensitivity.saturating_add(1);
                self.set_mic().await;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.mic_sensitivity = self.mic_sensitivity.saturating_sub(1);
                self.set_mic().await;
            }
            _ => {}
        }
    }
}

pub async fn run(tx: mpsc::Sender<[u8; 9]>) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(tx);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.on_key(key).await {
                        break;
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let title = Paragraph::new("🦇 BAT-COMPUTER - LIGHT CONTROLLER 🦇")
        .block(title_block)
        .alignment(ratatui::layout::Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, chunks[0]);

    // Tabs
    let titles = vec!["Color", "Pattern", "Mic"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Modules"))
        .select(match app.active_tab {
            ActiveTab::Color => 0,
            ActiveTab::Pattern => 1,
            ActiveTab::Mic => 2,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(tabs, chunks[1]);

    // Content
    match app.active_tab {
        ActiveTab::Color => draw_color_tab(f, app, chunks[2]),
        ActiveTab::Pattern => draw_pattern_tab(f, app, chunks[2]),
        ActiveTab::Mic => draw_mic_tab(f, app, chunks[2]),
    }

    // Footer
    let footer_text = match app.active_tab {
        ActiveTab::Color => {
            "Tab: Next | Shift+Tab: Prev | q: Quit | 1/2/3: Select R/G/B | ↑/↓: Adjust Value"
        }
        ActiveTab::Pattern => "Tab: Next | Shift+Tab: Prev | q: Quit | ↑/↓: Adjust Pattern Index",
        ActiveTab::Mic => "Tab: Next | Shift+Tab: Prev | q: Quit | ↑/↓: Adjust Sensitivity",
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[3]);
}

fn draw_color_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let draw_gauge =
        |f: &mut Frame, label: &str, value: u8, color: Color, selected: bool, area: Rect| {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(label)
                .style(if selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });

            let gauge = Gauge::default()
                .block(block)
                .gauge_style(Style::default().fg(color))
                .ratio(value as f64 / 255.0);
            f.render_widget(gauge, area);
        };

    draw_gauge(
        f,
        "Red (1)",
        app.color.r,
        Color::Red,
        app.color_selection == 0,
        chunks[0],
    );
    draw_gauge(
        f,
        "Green (2)",
        app.color.g,
        Color::Green,
        app.color_selection == 1,
        chunks[1],
    );
    draw_gauge(
        f,
        "Blue (3)",
        app.color.b,
        Color::Blue,
        app.color_selection == 2,
        chunks[2],
    );

    // Preview
    let preview_block = Block::default().borders(Borders::ALL).title("Preview");

    let canvas = Canvas::default()
        .block(preview_block)
        .x_bounds([-80.0, 80.0])
        .y_bounds([20.0, 90.0])
        .paint(move |ctx| {
            let color = Color::Rgb(app.color.r, app.color.g, app.color.b);
            let points = vec![
                (0.0, 30.0),   // Tail
                (20.0, 40.0),  // Lower Wing 1
                (40.0, 50.0),  // Lower Wing 2
                (50.0, 40.0),  // Lower Wing 3
                (60.0, 60.0),  // Wing Tip
                (40.0, 80.0),  // Upper Wing
                (20.0, 55.0),  // Head Side
                (15.0, 70.0),  // Ear Tip
                (5.0, 50.0),   // Head Top Center
                (-5.0, 50.0),  // Head Top Center
                (-15.0, 70.0), // Ear Tip
                (-20.0, 55.0), // Head Side
                (-40.0, 80.0), // Upper Wing
                (-60.0, 60.0), // Wing Tip
                (-50.0, 40.0), // Lower Wing 3
                (-40.0, 50.0), // Lower Wing 2
                (-20.0, 40.0), // Lower Wing 1
                (0.0, 30.0),   // Tail
            ];

            ctx.draw(&FilledPolygon { points, color });
        });

    f.render_widget(canvas, chunks[3]);
}

fn draw_pattern_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Pattern Selector")
        .style(Style::default().fg(Color::Yellow));

    let text = vec![
        Line::from(Span::styled(
            format!("Current Pattern Index: {}", app.pattern),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Use UP/DOWN keys to change the pattern."),
        Line::from("Patterns are hardware defined."),
    ];

    let p = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_mic_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Microphone Sensitivity")
        .style(Style::default().fg(Color::Yellow));

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(app.mic_sensitivity as f64 / 255.0)
        .label(format!("Sensitivity: {}", app.mic_sensitivity));

    f.render_widget(gauge, area);
}

struct FilledPolygon {
    points: Vec<(f64, f64)>,
    color: Color,
}

impl Shape for FilledPolygon {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        if self.points.len() < 3 {
            return;
        }

        let (min_y, max_y) = self
            .points
            .iter()
            .fold((f64::MAX, f64::MIN), |(min, max), &(_, y)| {
                (min.min(y), max.max(y))
            });

        // Scanline fill
        let mut y = min_y;
        while y <= max_y {
            let mut intersections = vec![];
            for i in 0..self.points.len() {
                let p1 = self.points[i];
                let p2 = self.points[(i + 1) % self.points.len()];

                if (p1.1 <= y && p2.1 > y) || (p2.1 <= y && p1.1 > y) {
                    let x = p1.0 + (y - p1.1) / (p2.1 - p1.1) * (p2.0 - p1.0);
                    intersections.push(x);
                }
            }

            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            for i in (0..intersections.len()).step_by(2) {
                if i + 1 < intersections.len() {
                    CanvasLine {
                        x1: intersections[i],
                        y1: y,
                        x2: intersections[i + 1],
                        y2: y,
                        color: self.color,
                    }
                    .draw(painter);
                }
            }

            y += 0.5; // Step size (adjust for density)
        }

        // Draw Outline
        for i in 0..self.points.len() {
            let p1 = self.points[i];
            let p2 = self.points[(i + 1) % self.points.len()];
            CanvasLine {
                x1: p1.0,
                y1: p1.1,
                x2: p2.0,
                y2: p2.1,
                color: self.color,
            }
            .draw(painter);
        }
    }
}
