use crate::app::AppContext;
use crate::error::AppResult;
use crate::usecase;
use crate::usecase::ProfileView;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use std::io::{self, Stdout};
use std::time::Duration;

const TICK_INTERVAL_MS: u64 = 100;
const MARQUEE_SPEED_TICKS: u64 = 4;
const MARQUEE_HOLD_TICKS: u64 = 8;
const MARQUEE_GAP_CHARS: usize = 6;

pub async fn pick_profile_for_connect(ctx: &AppContext) -> AppResult<Option<ProfileView>> {
    let profiles = usecase::profile::list_profiles(ctx).await?;
    if profiles.is_empty() {
        return Ok(None);
    }

    let mut state = PickerState::new(profiles);
    run_picker(&mut state)
}

struct PickerState {
    profiles: Vec<ProfileView>,
    filtered: Vec<usize>,
    selected: usize,
    query: String,
    marquee_tick: u64,
}

impl PickerState {
    fn new(profiles: Vec<ProfileView>) -> Self {
        let mut state = Self {
            profiles,
            filtered: Vec::new(),
            selected: 0,
            query: String::new(),
            marquee_tick: 0,
        };
        state.refilter();
        state
    }

    fn reset_marquee(&mut self) {
        self.marquee_tick = 0;
    }

    fn bump_marquee(&mut self) {
        self.marquee_tick = self.marquee_tick.wrapping_add(1);
    }

    fn refilter(&mut self) {
        let q = self.query.trim().to_ascii_lowercase();

        self.filtered = self
            .profiles
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if q.is_empty() {
                    return true;
                }

                let target = format!("{} {} {} {} {}", p.label, p.user, p.host, p.port, p.mode,)
                    .to_ascii_lowercase();

                target.contains(&q)
            })
            .map(|(idx, _)| idx)
            .collect();

        if self.filtered.is_empty() {
            self.selected = 0;
            return;
        }

        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }

        self.reset_marquee();
    }

    fn selected_profile(&self) -> Option<ProfileView> {
        self.filtered
            .get(self.selected)
            .and_then(|idx| self.profiles.get(*idx))
            .cloned()
    }

    fn select_prev(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
            self.reset_marquee();
        }
    }

    fn select_next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
            self.reset_marquee();
        }
    }

    fn select_first(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = 0;
            self.reset_marquee();
        }
    }

    fn select_last(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = self.filtered.len() - 1;
            self.reset_marquee();
        }
    }
}

fn run_picker(state: &mut PickerState) -> AppResult<Option<ProfileView>> {
    let mut terminal = setup_terminal()?;
    let result = picker_loop(&mut terminal, state);

    let restore_result = restore_terminal(&mut terminal);

    match (result, restore_result) {
        (Ok(selection), Ok(())) => Ok(selection),
        (Err(e), Ok(())) => Err(e),
        (Ok(_), Err(e)) => Err(e.into()),
        (Err(primary), Err(_)) => Err(primary),
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn picker_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut PickerState,
) -> AppResult<Option<ProfileView>> {
    loop {
        terminal.draw(|f| draw_picker(f, state))?;
        state.bump_marquee();

        if !event::poll(Duration::from_millis(TICK_INTERVAL_MS))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if matches!(key.kind, KeyEventKind::Release) {
                continue;
            }

            if let Some(outcome) = handle_key(key, state) {
                return Ok(outcome);
            }
        }
    }
}

fn handle_key(key: KeyEvent, state: &mut PickerState) -> Option<Option<ProfileView>> {
    match key.code {
        KeyCode::Esc => Some(None),
        KeyCode::Enter => Some(state.selected_profile()),
        KeyCode::Up => {
            state.select_prev();
            None
        }
        KeyCode::Down => {
            state.select_next();
            None
        }
        KeyCode::Home => {
            state.select_first();
            None
        }
        KeyCode::End => {
            state.select_last();
            None
        }
        KeyCode::Backspace => {
            state.query.pop();
            state.refilter();
            None
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(None),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.query.clear();
            state.refilter();
            None
        }
        KeyCode::Char('k') if key.modifiers.is_empty() => {
            state.select_prev();
            None
        }
        KeyCode::Char('j') if key.modifiers.is_empty() => {
            state.select_next();
            None
        }
        KeyCode::Char(ch) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
            state.query.push(ch);
            state.refilter();
            None
        }
        _ => None,
    }
}

fn draw_picker(f: &mut ratatui::Frame, state: &PickerState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(f.area());

    let query_line = Paragraph::new(format!("Search: {}", state.query)).block(
        Block::default()
            .title("jmssh connect picker")
            .borders(Borders::ALL),
    );
    f.render_widget(query_line, chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(chunks[1]);

    let profiles_block = Block::default().title("Profiles").borders(Borders::ALL);
    let profiles_inner = profiles_block.inner(body_chunks[0]);
    f.render_widget(profiles_block, body_chunks[0]);

    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(profiles_inner);

    let list_width = list_chunks[1].width as usize;
    let header = format_header_row(list_width);
    let header_widget = Paragraph::new(header).style(
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header_widget, list_chunks[0]);

    let mut items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(visible_idx, idx)| {
            let p = &state.profiles[*idx];
            let selected = !state.filtered.is_empty() && state.selected == visible_idx;
            ListItem::new(Line::from(format_profile_row(
                p,
                list_width,
                selected,
                state.marquee_tick,
            )))
        })
        .collect();

    if items.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "No profiles match current query",
            Style::default().fg(Color::DarkGray),
        )])));
    }

    let list = List::new(items).highlight_style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let mut list_state = ListState::default();
    if !state.filtered.is_empty() {
        list_state.select(Some(state.selected));
    }

    f.render_stateful_widget(list, list_chunks[1], &mut list_state);

    let detail_widget = Paragraph::new(detail_lines(state))
        .block(
            Block::default()
                .title("Profile Detail")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(detail_widget, body_chunks[1]);

    let hint = Paragraph::new(
        "Enter confirm | Esc/Ctrl-C cancel | ↑/↓ or j/k move | Backspace edit | Ctrl-U clear",
    )
    .style(Style::default().fg(Color::DarkGray));

    f.render_widget(hint, chunks[2]);
}

fn format_header_row(max_width: usize) -> String {
    let sep = "  ";
    let sep_total = sep.len() * 2;

    let Some((label_w, dest_w, mode_w)) = compute_col_widths(max_width, sep_total) else {
        return "LABEL  DESTINATION  MODE".to_string();
    };

    let label = fit_col("LABEL", label_w);
    let dest = fit_col("DESTINATION", dest_w);
    let mode = fit_col("MODE", mode_w);
    format!("{label}{sep}{dest}{sep}{mode}")
}

fn detail_lines(state: &PickerState) -> Vec<Line<'static>> {
    let Some(p) = state.selected_profile() else {
        return vec![
            Line::from(Span::styled(
                "No profile selected",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from("Use ↑/↓ or j/k to pick one"),
        ];
    };

    let tags = p.tags.clone().unwrap_or_else(|| "-".to_string());
    let note = p.note.clone().unwrap_or_else(|| "-".to_string());
    let endpoint = format!("{}@{}:{}", p.user, p.host, p.port);

    vec![
        Line::from(vec![
            Span::styled("Label: ", Style::default().fg(Color::Gray)),
            Span::raw(p.label),
        ]),
        Line::from(vec![
            Span::styled("Mode : ", Style::default().fg(Color::Gray)),
            Span::raw(p.mode),
        ]),
        Line::from(vec![
            Span::styled("User : ", Style::default().fg(Color::Gray)),
            Span::raw(p.user),
        ]),
        Line::from(vec![
            Span::styled("Host : ", Style::default().fg(Color::Gray)),
            Span::raw(p.host),
        ]),
        Line::from(vec![
            Span::styled("Port : ", Style::default().fg(Color::Gray)),
            Span::raw(p.port.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Dest : ", Style::default().fg(Color::Gray)),
            Span::raw(endpoint),
        ]),
        Line::from(vec![
            Span::styled("Tags : ", Style::default().fg(Color::Gray)),
            Span::raw(tags),
        ]),
        Line::from(vec![
            Span::styled("Note : ", Style::default().fg(Color::Gray)),
            Span::raw(note),
        ]),
    ]
}

fn format_profile_row(
    p: &ProfileView,
    max_width: usize,
    selected: bool,
    marquee_tick: u64,
) -> String {
    if max_width == 0 {
        return String::new();
    }

    let sep = "  ";
    let sep_total = sep.len() * 2;
    let Some((label_w, dest_w, mode_w)) = compute_col_widths(max_width, sep_total) else {
        let compact = format!("{}@{}:{} ({})", p.user, p.host, p.port, p.mode);
        return truncate_with_ellipsis(&compact, max_width);
    };

    let label = fit_col(&p.label, label_w);
    let dest_raw = format!("{}@{}:{}", p.user, p.host, p.port);
    let dest = fit_dest_col(&dest_raw, dest_w, selected, marquee_tick);
    let mode = fit_col(&p.mode, mode_w);

    format!("{label}{sep}{dest}{sep}{mode}")
}

fn compute_col_widths(max_width: usize, sep_total: usize) -> Option<(usize, usize, usize)> {
    let mut label_w = 18usize;
    let mut dest_w = 28usize;
    let mut mode_w = 9usize;

    let min_label_w = 10usize;
    let min_dest_w = 12usize;
    let min_mode_w = 6usize;

    let min_total = min_label_w + min_dest_w + min_mode_w + sep_total;
    if max_width < min_total {
        return None;
    }

    let current_total = label_w + dest_w + mode_w + sep_total;
    if current_total > max_width {
        let mut overflow = current_total - max_width;

        let take_dest = overflow.min(dest_w.saturating_sub(min_dest_w));
        dest_w -= take_dest;
        overflow -= take_dest;

        let take_label = overflow.min(label_w.saturating_sub(min_label_w));
        label_w -= take_label;
        overflow -= take_label;

        let take_mode = overflow.min(mode_w.saturating_sub(min_mode_w));
        mode_w -= take_mode;
        overflow -= take_mode;

        if overflow > 0 {
            return None;
        }
    }

    Some((label_w, dest_w, mode_w))
}

fn fit_col(s: &str, width: usize) -> String {
    let t = truncate_with_ellipsis(s, width);
    let len = t.chars().count();
    if len >= width {
        return t;
    }

    format!("{t}{}", " ".repeat(width - len))
}

fn fit_dest_col(s: &str, width: usize, selected: bool, marquee_tick: u64) -> String {
    if !selected {
        return fit_col(s, width);
    }

    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        return fit_col(s, width);
    }

    if marquee_tick < MARQUEE_HOLD_TICKS {
        return fit_col(s, width);
    }

    let mut ring = chars;
    ring.extend(std::iter::repeat_n(' ', MARQUEE_GAP_CHARS));
    let ring_len = ring.len();
    if ring_len == 0 {
        return fit_col(s, width);
    }

    let step = ((marquee_tick - MARQUEE_HOLD_TICKS) / MARQUEE_SPEED_TICKS) as usize;
    let start = step % ring_len;

    let mut out = String::with_capacity(width);
    for i in 0..width {
        out.push(ring[(start + i) % ring_len]);
    }
    out
}

fn truncate_with_ellipsis(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len <= width {
        return s.to_string();
    }

    if width == 0 {
        return String::new();
    }

    if width <= 3 {
        return ".".repeat(width);
    }

    let mut out = s.chars().take(width - 3).collect::<String>();
    out.push_str("...");
    out
}
