use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Tabs},
    Frame, Terminal,
};

struct Req {
    id: usize,
    host: &'static str,
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

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
    pub history: Vec<Req>,
    pub history_state: TableState,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Intercept", "History", "Settings"],
            index: 0,
            history: vec![
                Req {
                    id: 0,
                    host: "https://test1.com",
                    method: "GET",
                    url: "/",
                    status: 200,
                    headers: vec![
                        "Accept-Language: en-US",
                        "Accept-Encoding: gzip, deflate"
                    ]
                },
                Req {
                    id: 1,
                    host: "https://test2.com",
                    method: "GET",
                    url: "/thing",
                    status: 404,
                    headers: vec![
                        "Accept-Language: en-US",
                        "User-Agent: Rust"
                    ]
                },
                Req {
                    id: 2,
                    host: "https://example.com",
                    method: "POST",
                    url: "/get_item?id=1&name=thisisareallylongname&date=2023/01/23&reallylongkey=withshortvalue",
                    status: 200,
                    headers: vec![
                        "Accept-Language: en-US",
                        "Accept-Encoding: gzip, deflate",
                        "Content-Type: application/x-www-form-urlencoded; charset=UTF-8",
                        "Content-Length: 236"
                    ]
                },
                Req {
                    id: 3,
                    host: "https://example.com",
                    method: "GET",
                    url: "/",
                    status: 300,
                    headers: vec![
                        "Accept-Language: en-US",
                        "Accept-Encoding: gzip, deflate",
                    ]
                },
            ],
            history_state: TableState::default(),
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
                if i >= self.history.len() - 1 {
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

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
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
        Some(i) => app.history[i].to_paragraph(),
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
