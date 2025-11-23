use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

use crate::models::result::SearchResult;

const MAX_SECTION_HEIGHT: u16 = 12;

/// Application state for the TUI interface.
pub struct AppState {
    pub sections: Vec<(String, Vec<SearchResult>)>,
    pub section_states: Vec<ListState>,
    pub active_section: usize,
    pub selected_result: Option<SearchResult>,
}

impl AppState {
    /// Creates a new application state with the given search result sections.
    ///
    /// # Arguments
    ///
    /// * `sections` - A vector of (backend_name, results) tuples
    pub fn new(sections: Vec<(String, Vec<SearchResult>)>) -> Self {
        let states = sections
            .iter()
            .map(|(_, items)| {
                let mut state = ListState::default();
                if !items.is_empty() {
                    state.select(Some(0));
                }
                state
            })
            .collect();

        Self {
            sections,
            section_states: states,
            active_section: 0,
            selected_result: None,
        }
    }
}

/// Runs the TUI interface, allowing the user to browse and select packages.
///
/// # Arguments
///
/// * `app` - The application state containing search results
///
/// # Returns
///
/// Returns `Ok(())` when the user exits or selects a package, or an error
/// if something goes wrong with the terminal interface.
pub fn run_tui(app: &mut AppState) -> io::Result<()> {
    // Enter alternate screen mode and enable raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the TUI loop
    let result = loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        break Ok(());
                    }

                    KeyCode::Char('j') | KeyCode::Down => move_down(app),
                    KeyCode::Char('k') | KeyCode::Up => move_up(app),
                    KeyCode::Char('g') => jump_top(app),
                    KeyCode::Char('G') => jump_bottom(app),

                    KeyCode::Char('h') | KeyCode::Left => prev_section(app),
                    KeyCode::Char('l') | KeyCode::Right => next_section(app),
                    KeyCode::Tab => next_section(app),
                    KeyCode::BackTab => prev_section(app),

                    KeyCode::Enter => {
                        if let Some(item) = get_selected_item(app) {
                            app.selected_result = Some(item);
                            break Ok(());
                        }
                    }

                    _ => {}
                }
            }
        }
    };

    // Drop terminal before leaving alternate screen
    drop(terminal);
    
    // Always restore terminal state
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    
    result
}

// ---------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------

fn next_section(app: &mut AppState) {
    app.active_section = (app.active_section + 1) % app.sections.len();
}

fn prev_section(app: &mut AppState) {
    if app.active_section == 0 {
        app.active_section = app.sections.len() - 1;
    } else {
        app.active_section -= 1;
    }
}

fn move_down(app: &mut AppState) {
    let (_, items) = &app.sections[app.active_section];
    if items.is_empty() { return; }

    let state = &mut app.section_states[app.active_section];
    let i = match state.selected() {
        Some(i) if i + 1 < items.len() => i + 1,
        _ => return,
    };
    state.select(Some(i));
}

fn move_up(app: &mut AppState) {
    let (_, items) = &app.sections[app.active_section];
    if items.is_empty() { return; }

    let state = &mut app.section_states[app.active_section];
    let i = match state.selected() {
        Some(i) if i > 0 => i - 1,
        _ => return,
    };
    state.select(Some(i));
}

fn jump_top(app: &mut AppState) {
    let (_, items) = &app.sections[app.active_section];
    if items.is_empty() { return; }
    app.section_states[app.active_section].select(Some(0));
}

fn jump_bottom(app: &mut AppState) {
    let (_, items) = &app.sections[app.active_section];
    if items.is_empty() { return; }
    app.section_states[app.active_section].select(Some(items.len() - 1));
}

fn get_selected_item(app: &mut AppState) -> Option<SearchResult> {
    let (_, items) = &app.sections[app.active_section];
    let state = &app.section_states[app.active_section];
    items.get(state.selected()?).cloned()
}

// ---------------------------------------------------------------------
// UI Rendering
// ---------------------------------------------------------------------

fn draw_ui(f: &mut ratatui::Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            app.sections
                .iter()
                .map(|(_, items)| Constraint::Length(section_height(items)))
                .collect::<Vec<_>>(),
        )
        .split(f.size());

    for (i, (title, items)) in app.sections.iter().enumerate() {
        let area = chunks[i];

        let is_active = i == app.active_section;

        let border_style = Style::default()
            .fg(Color::Indexed(2))
            .add_modifier(if is_active { Modifier::BOLD } else { Modifier::DIM });

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(title.clone(), border_style));

        if items.is_empty() {
            let text = Paragraph::new(" No packages matched ")
                .block(block);
            f.render_widget(text, area);
            continue;
        }

        let list_items: Vec<ListItem> = items
            .iter()
            .map(|pkg| {
                // Format: name [version] — description
                let version_info = pkg
                    .version
                    .as_ref()
                    .map(|v| format!(" [{}]", v))
                    .unwrap_or_default();
                
                let display_text = if pkg.description.is_empty() {
                    format!("{}{}", pkg.name, version_info)
                } else {
                    format!("{}{} — {}", pkg.name, version_info, pkg.description)
                };
                
                ListItem::new(Line::from(Span::raw(display_text)))
            })
            .collect();

        let list = List::new(list_items)
            .block(block)
            .highlight_symbol(" ")
            .highlight_style(
                Style::default()
                    .fg(Color::Indexed(2))
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut app.section_states[i]);
    }
}

fn section_height(items: &Vec<SearchResult>) -> u16 {
    if items.is_empty() {
        return 3;
    }

    let size = items.len() as u16 + 2;
    size.min(MAX_SECTION_HEIGHT)
}

