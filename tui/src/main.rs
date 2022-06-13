use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, Tabs},
    Frame, Terminal,
};

struct Req {
    id: usize,
    host: &'static str,
    method: &'static str,
    url: &'static str,
    status: usize,
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
}

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
    pub history: Vec<Req>,
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
                },
                Req {
                    id: 1,
                    host: "https://test2.com",
                    method: "GET",
                    url: "/thing",
                    status: 404,
                },
                Req {
                    id: 2,
                    host: "https://example.com",
                    method: "POST",
                    url: "/get_item?id=1&name=thisisareallylongname&date=2023/01/23&reallylongkey=withshortvalue",
                    status: 200,
                },
                Req {
                    id: 3,
                    host: "https://example.com",
                    method: "GET",
                    url: "/",
                    status: 300,
                },
            ],
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    pub fn go_to_tab(&mut self, index: usize) {
        self.index = index;
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
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Tab => app.next(),
                KeyCode::BackTab => app.previous(),
                KeyCode::Char('i') => app.go_to_tab(0),
                KeyCode::Char('h') => app.go_to_tab(1),
                KeyCode::Char('s') => app.go_to_tab(2),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
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
        1 => f.render_widget(render_history(app), chunks[1]),
        2 => f.render_widget(
            Block::default().title("Inner 2").borders(Borders::ALL),
            chunks[1],
        ),
        _ => unreachable!(),
    };
}

fn render_history(app: &App) -> Table<'static> {
    let rows: Vec<Row> = app.history.iter().map(|r| r.to_row()).collect();
    Table::new(rows)
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
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ")
        .block(Block::default().title("History").borders(Borders::ALL))
}
