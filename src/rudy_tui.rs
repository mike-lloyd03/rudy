use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tokio::sync::mpsc::Receiver;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Tabs},
    Frame, Terminal,
};

pub struct Req {
    id: usize,
    host: String,
    method: &'static str,
    url: &'static str,
    status: usize,
    headers: Vec<&'static str>,
}

impl Req {
    fn to_row(&self) -> Row<'static> {
        Row::new(vec![
            self.id.to_string(),
            self.host.to_string(),
            self.method.to_string(),
            self.url.to_string(),
            self.status.to_string(),
        ])
    }

    fn to_paragraph(&self) -> Paragraph<'static> {
        let mut p = format!("{} {}\n", self.method, self.url);
        p += &format!("Host: {}\n", self.host);
        for h in &self.headers {
            p += &format!("{}\n", h)
        }

        Paragraph::new(p)
    }
}

pub struct App<'a> {
    pub titles: Vec<&'static str>,
    pub index: usize,
    pub history: Vec<Req>,
    pub history_state: TableState,
    pub receiver: &'a mut Receiver<String>,
}

impl<'a> App<'a> {
    pub fn new(rx: &'a mut Receiver<String>) -> Self {
        App {
            titles: vec!["Intercept", "History", "Settings"],
            index: 0,
            history: vec![],
            history_state: TableState::default(),
            receiver: rx,
        }
    }

    pub fn next_tab(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous_tab(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn go_to_tab(&mut self, index: usize) {
        self.index = index;
    }

    pub fn next_hist_item(&mut self) {
        let i = match self.history_state.selected() {
            Some(i) => {
                if !self.history.is_empty() && i >= self.history.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.history_state.select(Some(i));
    }

    pub fn prev_hist_item(&mut self) {
        let i = match self.history_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.history_state.select(Some(i));
    }
}

pub async fn run<'a>(app: App<'a>) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    // let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

pub async fn run_app<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'a>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Tab => app.next_tab(),
                KeyCode::BackTab => app.previous_tab(),
                KeyCode::Char('i') => app.go_to_tab(0),
                KeyCode::Char('h') => app.go_to_tab(1),
                KeyCode::Char('s') => app.go_to_tab(2),
                KeyCode::Char('j') => app.next_hist_item(),
                KeyCode::Char('k') => app.prev_hist_item(),
                _ => {}
            }
        } else {
            while let Some(message) = app.receiver.recv().await {
                let id = app.history.len() + 1;
                let method = "GET";
                let url = "http://test.com";
                let status = 200;
                let headers = vec![""];
                app.history.push(Req {
                    id,
                    host: message,
                    method,
                    url,
                    status,
                    headers,
                })
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);

    let block = Block::default().style(Style::default());
    f.render_widget(block, size);

    let titles = app
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Red)),
                Span::styled(rest, Style::default().fg(Color::Blue)),
            ])
        })
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Rudy"))
        .select(app.index)
        .style(Style::default().fg(Color::Red))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);

    match app.index {
        0 => f.render_widget(
            Block::default().title("Inner 0").borders(Borders::ALL),
            chunks[1],
        ),
        1 => render_history(f, app, chunks[1]),
        2 => f.render_widget(
            Block::default().title("Inner 2").borders(Borders::ALL),
            chunks[1],
        ),
        _ => unreachable!(),
    };
}

fn render_history<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    // Outer block
    let block = Block::default()
        .title("History")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
    f.render_widget(block, area);

    let h_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(10), Constraint::Min(40)].as_ref())
        .split(area);

    // History list view
    let rows: Vec<Row> = app.history.iter().map(|r| r.to_row()).collect();
    let table = Table::new(rows)
        .style(Style::default().fg(Color::White))
        .header(
            Row::new(vec!["ID", "Host", "Method", "URL", "Status"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .widths(&[
            Constraint::Length(5),
            Constraint::Min(30),
            Constraint::Min(10),
            Constraint::Min(40),
            Constraint::Min(10),
        ])
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green)
                .bg(Color::Gray),
        );
    f.render_stateful_widget(table, h_chunks[0], &mut app.history_state);

    // History detail view
    let detail = match app.history_state.selected() {
        Some(i) => {
            if !app.history.is_empty() {
                app.history[i].to_paragraph()
            } else {
                Paragraph::new("")
            }
        }
        None => Paragraph::new(""),
    }
    .block(
        Block::default()
            .title("Request")
            .title_alignment(Alignment::Center)
            .borders(Borders::TOP),
    );
    f.render_widget(detail, h_chunks[1]);
}
