use crate::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::HashSet;
use std::{
    io::{self, Result as IoResult},
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table, Tabs},
    Frame, Terminal,
};

struct AppState {
    passwords: Vec<PasswordData>,
    tag: Tag,
    list: ListData<PasswordData>,
    preview: ListData<DisplayValue>,
    view: SelectedView,
    logs: Vec<Log>,
    lock: Instant,
}

struct Tag {
    selected: SelectedTag,
    tags: Vec<String>,
}

enum SelectedTag {
    All,
    Archive,
    Tag(usize),
}

impl Tag {
    fn next(&mut self) {
        self.selected = match self.selected {
            SelectedTag::All => SelectedTag::Archive,
            SelectedTag::Archive => {
                if self.tags.is_empty() {
                    SelectedTag::All
                } else {
                    SelectedTag::Tag(0)
                }
            }
            SelectedTag::Tag(i) => {
                if i < self.tags.len() - 1 {
                    SelectedTag::Tag(i + 1)
                } else {
                    SelectedTag::All
                }
            }
        }
    }

    fn prev(&mut self) {
        self.selected = match self.selected {
            SelectedTag::All => {
                if self.tags.is_empty() {
                    SelectedTag::Archive
                } else {
                    SelectedTag::Tag(self.tags.len() - 1)
                }
            }
            SelectedTag::Archive => SelectedTag::All,
            SelectedTag::Tag(i) => {
                if i == 0 {
                    SelectedTag::Archive
                } else {
                    SelectedTag::Tag(i - 1)
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SelectedView {
    List,
    Preview,
}

struct ListData<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> ListData<T> {
    fn new(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct Log {
    time: String,
    message: String,
}

impl Log {
    fn new(message: String) -> Self {
        Self {
            time: time_now(),
            message,
        }
    }
}

pub fn start(mut passwords: Vec<PasswordData>) -> IoResult<()> {
    passwords.sort_by(|a, b| {
        let a = a.pin.unwrap_or_default();
        let b = b.pin.unwrap_or_default();
        b.cmp(&a)
    });

    let tags = passwords
        .iter()
        .flat_map(|item| item.tags.clone().unwrap_or_default())
        .collect::<HashSet<String>>()
        .into_iter()
        .collect::<Vec<String>>();

    let logs = vec![
        Log::new("Loading completed".to_string()),
        Log::new(format!("Total {} passwords", passwords.len())),
    ];

    let state = AppState {
        passwords,
        tag: Tag {
            selected: SelectedTag::All,
            tags,
        },
        list: ListData::new(vec![]),
        preview: ListData::new(vec![]),
        view: SelectedView::List,
        logs,
        lock: Instant::now(),
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let rst = run_app(&mut terminal, state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    rst
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut state: AppState) -> io::Result<()> {
    loop {
        if state.lock.elapsed().as_secs() > 60 {
            return Ok(());
        }

        terminal.draw(|f| ui(f, &mut state))?;

        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                state.lock = Instant::now();
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('a') => {
                        state.tag.prev();
                        state.view = SelectedView::List;
                        state.list.unselect();
                        state.preview.state.select(None);
                    }
                    KeyCode::Char('d') => {
                        state.tag.next();
                        state.view = SelectedView::List;
                        state.list.unselect();
                        state.preview.state.select(None);
                    }
                    KeyCode::Down => match state.view {
                        SelectedView::List => state.list.next(),
                        SelectedView::Preview => state.preview.next(),
                    },
                    KeyCode::Up => match state.view {
                        SelectedView::List => state.list.prev(),
                        SelectedView::Preview => state.preview.prev(),
                    },
                    KeyCode::Left => {
                        if state.list.state.selected().is_some() {
                            match state.view {
                                SelectedView::List => state.list.unselect(),
                                SelectedView::Preview => {
                                    state.preview.unselect();
                                    state.view = SelectedView::List;
                                }
                            }
                        } else if !state.list.items.is_empty() {
                            state.list.state.select(Some(0));
                        }
                    }
                    KeyCode::Right => {
                        if state.list.state.selected().is_some() {
                            match state.view {
                                SelectedView::List => {
                                    state.view = SelectedView::Preview;
                                    state.preview.next();
                                }
                                SelectedView::Preview => {
                                    state.preview.unselect();
                                    state.view = SelectedView::List;
                                }
                            }
                        } else if !state.list.items.is_empty() {
                            state.list.state.select(Some(0));
                        }
                    }
                    KeyCode::Enter => {
                        if state.view == SelectedView::Preview {
                            if let Some(i) = state.preview.state.selected() {
                                let item = &state.preview.items[i];
                                if state.logs.len() > 5 {
                                    state.logs.remove(0);
                                }
                                let msg = match copy_text(&item.copy_value) {
                                    Ok(_) => format!("Copied '{}'", item.key),
                                    Err(msg) => format!("Failed '{}'", msg),
                                };
                                state.logs.push(Log::new(msg));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(f.size().height - 11),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(f.size());
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(layout[1]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(layout[2]);

    let tags_area = layout[0];
    let [list_area, preview_area] = [main[0], main[1]];
    let [log_area, help_area] = [bottom[0], bottom[1]];

    state.list.items = state
        .passwords
        .iter()
        .filter(|item| match state.tag.selected {
            SelectedTag::All => item.archive != Some(true),
            SelectedTag::Archive => item.archive == Some(true),
            SelectedTag::Tag(i) => {
                if item.archive != Some(true) {
                    let tag = state.tag.tags[i].clone();
                    let tags = item.tags.clone().unwrap_or_default();
                    return tags.contains(&tag);
                }
                false
            }
        })
        .cloned()
        .collect();

    f.render_widget(draw_tags(&state.tag), tags_area);

    f.render_stateful_widget(
        draw_list(&state.list.items, &state.tag),
        list_area,
        &mut state.list.state,
    );

    if let Some(i) = state.list.state.selected() {
        state.preview.items = state.list.items[i].to_display_value();
        let preview = draw_preview(&state.list.items[i]);
        f.render_stateful_widget(preview, preview_area, &mut state.preview.state);
    }

    f.render_widget(draw_logs(&state.logs), log_area);

    f.render_widget(draw_help(), help_area);
}

fn draw_tags<'a>(tag: &Tag) -> Tabs<'a> {
    let style = Style::default().fg(Color::White);

    let mut tags = vec![
        Spans::from(Span::styled(" All ", style)),
        Spans::from(Span::styled(" Archived ", style)),
    ];
    tags.extend(
        tag.tags
            .iter()
            .map(|tag| Spans::from(Span::styled(format!(" {} ", tag), style))),
    );

    let selected = match tag.selected {
        SelectedTag::All => 0,
        SelectedTag::Archive => 1,
        SelectedTag::Tag(i) => i + 2,
    };

    Tabs::new(tags)
        .block(Block::default().borders(Borders::ALL).title(" Tags "))
        .select(selected)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(54, 132, 246))
                .add_modifier(Modifier::BOLD),
        )
}

fn draw_list<'a>(passwords: &[PasswordData], tag: &Tag) -> List<'a> {
    let items: Vec<ListItem> = passwords
        .iter()
        .map(|item| {
            ListItem::new(vec![
                Spans::from(Span::styled(
                    item.name(true),
                    Style::default().fg(Color::White),
                )),
                Spans::from(Span::styled(item.user(), Style::default().fg(Color::White))),
                Spans::from(""),
            ])
        })
        .collect();
    let title = match tag.selected {
        SelectedTag::All => " All ".to_string(),
        SelectedTag::Archive => " Archive ".to_string(),
        SelectedTag::Tag(i) => format!(" {} ", tag.tags[i]),
    };

    List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(54, 132, 246))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ↪ ")
}

fn draw_preview(data: &PasswordData) -> List {
    let items = data
        .to_display_value()
        .into_iter()
        .map(|item| {
            ListItem::new(vec![
                Spans::from(Span::styled(item.key, Style::default().fg(Color::White))),
                Spans::from(Span::styled(item.value, Style::default().fg(Color::White))),
                Spans::from(""),
            ])
        })
        .collect::<Vec<ListItem>>();

    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", data.name(false))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(54, 132, 246))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ↪ ")
}

fn draw_logs<'a>(logs: &[Log]) -> List<'a> {
    let items = logs
        .iter()
        .map(|log| {
            ListItem::new(vec![Spans::from(Span::raw(format!(
                "{} {}",
                log.time, log.message
            )))])
        })
        .collect::<Vec<ListItem>>();

    List::new(items).block(Block::default().borders(Borders::ALL).title(" Logs "))
}

fn draw_help<'a>() -> Table<'a> {
    let rows = [
        ("A/D", "Toggle tag"),
        ("Up/Down", "Toggle selected"),
        ("Left/Right", "Toggle list/password"),
        ("Enter", "Copy value"),
        ("Q", "Quit"),
    ]
    .map(|(name, info)| {
        Row::new(vec![
            Cell::from(Span::styled(name, Style::default().fg(Color::LightCyan))),
            Cell::from(Span::styled(info, Style::default().fg(Color::Gray))),
        ])
    });
    Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1)
}
