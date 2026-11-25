use crate::actions::Action;
use crate::links::{self, Link, PathResolver};
use crate::settings::BackendSettings;
use alacritty_terminal::event::{
    Event, EventListener, Notify, OnResize, WindowSize,
};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Direction, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::search::RegexSearch;
use alacritty_terminal::term::{
    self, cell::{Cell, Flags}, test::TermSize, viewport_to_point, Term, TermMode,
};
use alacritty_terminal::{tty, Grid};
use iced::keyboard::Modifiers;
use iced_core::Size;
use std::borrow::Cow;
use std::cmp::min;
use std::io::Result;
use std::ops::RangeInclusive;
use std::sync::Arc;
use tokio::sync::mpsc;

/// `text` as a regex that matches it literally.
fn escape_regex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if "\\.+*?()|[]{}^$#&-~".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[derive(Debug, Clone)]
pub enum Command {
    Write(Vec<u8>),
    Scroll(i32),
    Resize(Option<Size<f32>>, Option<Size<f32>>),
    SelectStart(SelectionType, (f32, f32)),
    SelectUpdate((f32, f32)),
    ProcessLink(LinkAction, Point),
    MouseReport(MouseButton, Modifiers, Point, bool),
    ProcessAlacrittyEvent(Event),
    /// Find `text` (plain text, not a regex; smart case) from the current selection, or from the bottom of the
    /// history when there is none: `older` goes up (towards older output), otherwise down. The match is selected
    /// and scrolled into view.
    Search { text: String, older: bool },
    /// Drop the selection (e.g. when the search is closed).
    ClearSelection,
    /// Show the history this many lines up from the bottom (0 = the bottom), e.g. from the scrollbar.
    ScrollTo(usize),
}

#[derive(Debug, Clone)]
pub enum MouseMode {
    Sgr,
    Normal(bool),
}

impl From<TermMode> for MouseMode {
    fn from(term_mode: TermMode) -> Self {
        if term_mode.contains(TermMode::SGR_MOUSE) {
            MouseMode::Sgr
        } else if term_mode.contains(TermMode::UTF8_MOUSE) {
            MouseMode::Normal(true)
        } else {
            MouseMode::Normal(false)
        }
    }
}

#[derive(Debug, Clone)]
pub enum MouseButton {
    LeftButton = 0,
    MiddleButton = 1,
    RightButton = 2,
    LeftMove = 32,
    MiddleMove = 33,
    RightMove = 34,
    NoneMove = 35,
    ScrollUp = 64,
    ScrollDown = 65,
    Other = 99,
}

#[derive(Debug, Clone)]
pub enum LinkAction {
    Clear,
    Hover,
    Open,
}

#[derive(Clone, Copy, Debug)]
pub struct TerminalSize {
    pub cell_width: u16,
    pub cell_height: u16,
    num_cols: u16,
    num_lines: u16,
    layout_width: f32,
    layout_height: f32,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cell_width: 1,
            cell_height: 1,
            num_cols: 80,
            num_lines: 50,
            layout_width: 80.0,
            layout_height: 50.0,
        }
    }
}

impl Dimensions for TerminalSize {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn columns(&self) -> usize {
        self.num_cols as usize
    }

    fn last_column(&self) -> Column {
        Column(self.num_cols as usize - 1)
    }

    fn bottommost_line(&self) -> Line {
        Line(self.num_lines as i32 - 1)
    }

    fn screen_lines(&self) -> usize {
        self.num_lines as usize
    }
}

impl From<TerminalSize> for WindowSize {
    fn from(size: TerminalSize) -> Self {
        Self {
            num_lines: size.num_lines,
            num_cols: size.num_cols,
            cell_width: size.cell_width,
            cell_height: size.cell_height,
        }
    }
}

pub struct Backend {
    term: Arc<FairMutex<Term<EventProxy>>>,
    size: TerminalSize,
    notifier: Notifier,
    last_content: RenderableContent,
    resolve_path: Option<PathResolver>,
    /// The shell's process, to find its current folder (unix only).
    shell_pid: Option<u32>,
}

impl Backend {
    pub fn new(
        id: u64,
        pty_event_proxy_sender: mpsc::Sender<Event>,
        settings: BackendSettings,
    ) -> Result<Self> {
        let scrolling_history = settings.scrollback;
        let pty_config = tty::Options {
            shell: Some(tty::Shell::new(settings.program, settings.args)),
            working_directory: settings.working_directory,
            env: settings.env,
            ..tty::Options::default()
        };

        let config = term::Config {
            scrolling_history,
            ..term::Config::default()
        };
        let terminal_size = TerminalSize::default();
        let pty = tty::new(&pty_config, terminal_size.into(), id)?;
        #[cfg(unix)]
        let shell_pid = Some(pty.child().id());
        #[cfg(not(unix))]
        let shell_pid = None;

        let event_proxy = EventProxy(pty_event_proxy_sender);

        let mut term = Term::new(config, &terminal_size, event_proxy.clone());
        if !settings.preamble.is_empty() {
            // Before the event loop starts: the program's first output always comes after it.
            let mut processor: alacritty_terminal::vte::ansi::Processor = alacritty_terminal::vte::ansi::Processor::new();
            processor.advance(&mut term, &settings.preamble);
        }

        let cursor = term.grid_mut().cursor_cell().clone();

        let initial_content = RenderableContent {
            grid: term.grid().clone(),
            selectable_range: None,
            terminal_mode: *term.mode(),
            terminal_size,
            cursor: cursor.clone(),
            hovered_hyperlink: None,
            hovered_link: None,
        };

        let term = Arc::new(FairMutex::new(term));

        let pty_event_loop =
            EventLoop::new(term.clone(), event_proxy, pty, false, false)?;

        let notifier = Notifier(pty_event_loop.channel());

        let _ = pty_event_loop.spawn();

        Ok(Self {
            term: term.clone(),
            size: terminal_size,
            notifier,
            last_content: initial_content,
            resolve_path: settings.resolve_path,
            shell_pid,
        })
    }

    pub fn handle(&mut self, cmd: Command) -> Action {
        let mut action = Action::default();
        let term = self.term.clone();
        let mut term = term.lock();
        match cmd {
            Command::ProcessAlacrittyEvent(event) => {
                match event {
                    Event::Exit => {
                        action = Action::Shutdown;
                    },
                    Event::Title(title) => {
                        action = Action::ChangeTitle(title);
                    },
                    Event::PtyWrite(pty) => {
                        self.notifier.notify(pty.into_bytes())
                    },
                    _ => {},
                };
            },
            Command::Write(input) => {
                self.write(input);
                term.scroll_display(Scroll::Bottom);
            },
            Command::Scroll(delta) => {
                self.scroll(&mut term, delta);
            },
            Command::Resize(layout_size, font_measure) => {
                self.resize(&mut term, layout_size, font_measure);
            },
            Command::SelectStart(selection_type, (x, y)) => {
                self.start_selection(&mut term, selection_type, x, y);
            },
            Command::SelectUpdate((x, y)) => {
                self.update_selection(&mut term, x, y);
            },
            Command::ProcessLink(link_action, point) => {
                if let Some(link) = self.process_link_action(&term, link_action, point) {
                    action = Action::OpenLink(link);
                }
            },
            Command::MouseReport(button, modifiers, point, pressed) => {
                self.process_mouse_report(button, modifiers, point, pressed);
            },
            Command::Search { text, older } => {
                action = Action::SearchResult(Self::search(&mut term, &text, older));
            },
            Command::ClearSelection => {
                term.selection = None;
            },
            Command::ScrollTo(target) => {
                let target = target.min(term.grid().history_size());
                let delta = target as i64 - term.grid().display_offset() as i64;
                if delta != 0 {
                    term.scroll_display(Scroll::Delta(delta as i32));
                }
            },
        };

        action
    }

    fn search<T: EventListener>(terminal: &mut Term<T>, text: &str, older: bool) -> bool {
        if text.is_empty() {
            return false;
        }
        let Ok(mut regex) = RegexSearch::new(&escape_regex(text)) else {
            return false;
        };
        let current = terminal.selection.as_ref().and_then(|s| s.to_range(terminal));
        let last_column = Column(terminal.columns().saturating_sub(1));
        let origin = match (current, older) {
            (Some(range), true) => range.start.sub(terminal, alacritty_terminal::index::Boundary::None, 1),
            (Some(range), false) => range.end.add(terminal, alacritty_terminal::index::Boundary::None, 1),
            (None, true) => Point::new(terminal.bottommost_line(), last_column),
            (None, false) => Point::new(terminal.topmost_line(), Column(0)),
        };
        let (direction, side) = if older { (Direction::Left, Side::Right) } else { (Direction::Right, Side::Left) };
        let Some(found) = terminal.search_next(&mut regex, origin, direction, side, None) else {
            return false;
        };
        terminal.scroll_to_point(*found.start());
        let mut selection = Selection::new(SelectionType::Simple, *found.start(), Side::Left);
        selection.update(*found.end(), Side::Right);
        terminal.selection = Some(selection);
        true
    }

    fn process_link_action(
        &mut self,
        terminal: &Term<EventProxy>,
        link_action: LinkAction,
        point: Point,
    ) -> Option<Link> {
        match link_action {
            LinkAction::Hover => {
                self.hover(terminal, point);
                None
            },
            LinkAction::Clear => {
                self.last_content.hovered_hyperlink = None;
                self.last_content.hovered_link = None;
                None
            },
            // Opening is the embedder's job (`Action::OpenLink`): the backend never launches programs.
            LinkAction::Open => {
                self.hover(terminal, point);
                self.last_content.hovered_link.clone()
            },
        }
    }

    /// The link under `point`: an OSC 8 hyperlink first, then a URL or an existing path in the text.
    fn hover(&mut self, terminal: &Term<EventProxy>, point: Point) {
        let found = self.link_at(terminal, point);
        self.last_content.hovered_hyperlink = found.as_ref().map(|(range, _)| range.clone());
        self.last_content.hovered_link = found.map(|(_, link)| link);
    }

    fn link_at(
        &self,
        terminal: &Term<EventProxy>,
        point: Point,
    ) -> Option<(RangeInclusive<Point>, Link)> {
        let grid = terminal.grid();
        if point.line < grid.topmost_line()
            || point.line > grid.bottommost_line()
            || point.column.0 >= grid.columns()
        {
            return None;
        }
        let cells = logical_line(grid, point.line);
        let index = cells.iter().position(|(p, _)| *p == point)?;

        // OSC 8: the cells around the mouse with the same hyperlink.
        if let Some(link) = grid[point].hyperlink() {
            let same = |p: &Point| grid[*p].hyperlink().as_ref() == Some(&link);
            let start = (0..=index).rev().take_while(|&i| same(&cells[i].0)).last().unwrap_or(index);
            let end = (index..cells.len()).take_while(|&i| same(&cells[i].0)).last().unwrap_or(index);
            let target = self.resolve(links::Found::Url(link.uri().to_string()))?;
            return Some((cells[start].0..=cells[end].0, target));
        }

        let text: String = cells.iter().map(|(_, c)| *c).collect();
        let hit = links::at(&text, index)?;
        let target = self.resolve(hit.found)?;
        Some((cells[hit.start].0..=cells[hit.end - 1].0, target))
    }

    /// A URL stays a URL (`file://` becomes its path); a path must exist.
    fn resolve(&self, found: links::Found) -> Option<Link> {
        let (path, line, column) = match found {
            links::Found::Url(url) => match links::file_url_path(&url) {
                Some(path) => (path, None, None),
                None if url.starts_with("file:") => return None,
                None => return Some(Link::Url(url)),
            },
            links::Found::Path { path, line, column } => (path, line, column),
        };
        let resolver = self.resolve_path.as_ref()?;
        let path = (resolver.0)(&path, self.shell_pid)?;
        Some(Link::Path { path, line, column })
    }

    fn process_mouse_report(
        &self,
        button: MouseButton,
        modifiers: Modifiers,
        point: Point,
        pressed: bool,
    ) {
        let mut mods = 0;
        if modifiers.contains(Modifiers::SHIFT) {
            mods += 4;
        }
        if modifiers.contains(Modifiers::ALT) {
            mods += 8;
        }
        if modifiers.contains(Modifiers::COMMAND) {
            mods += 16;
        }

        match MouseMode::from(self.last_content.terminal_mode) {
            MouseMode::Sgr => {
                self.sgr_mouse_report(point, button as u8 + mods, pressed)
            },
            MouseMode::Normal(is_utf8) => {
                if pressed {
                    self.normal_mouse_report(
                        point,
                        button as u8 + mods,
                        is_utf8,
                    )
                } else {
                    self.normal_mouse_report(point, 3 + mods, is_utf8)
                }
            },
        }
    }

    fn sgr_mouse_report(&self, point: Point, button: u8, pressed: bool) {
        let c = if pressed { 'M' } else { 'm' };

        let msg = format!(
            "\x1b[<{};{};{}{}",
            button,
            point.column + 1,
            point.line + 1,
            c
        );

        self.notifier.notify(msg.as_bytes().to_vec());
    }

    fn normal_mouse_report(&self, point: Point, button: u8, is_utf8: bool) {
        let Point { line, column } = point;
        let max_point = if is_utf8 { 2015 } else { 223 };

        if line >= max_point || column >= max_point {
            return;
        }

        let mut msg = vec![b'\x1b', b'[', b'M', 32 + button];

        let mouse_pos_encode = |pos: usize| -> Vec<u8> {
            let pos = 32 + 1 + pos;
            let first = 0xC0 + pos / 64;
            let second = 0x80 + (pos & 63);
            vec![first as u8, second as u8]
        };

        if is_utf8 && column >= Column(95) {
            msg.append(&mut mouse_pos_encode(column.0));
        } else {
            msg.push(32 + 1 + column.0 as u8);
        }

        if is_utf8 && line >= 95 {
            msg.append(&mut mouse_pos_encode(line.0 as usize));
        } else {
            msg.push(32 + 1 + line.0 as u8);
        }

        self.notifier.notify(msg);
    }

    fn start_selection(
        &mut self,
        terminal: &mut Term<EventProxy>,
        selection_type: SelectionType,
        x: f32,
        y: f32,
    ) {
        let location = Self::selection_point(
            x,
            y,
            &self.size,
            terminal.grid().display_offset(),
        );
        terminal.selection = Some(Selection::new(
            selection_type,
            location,
            self.selection_side(x),
        ));
    }

    fn update_selection(
        &mut self,
        terminal: &mut Term<EventProxy>,
        x: f32,
        y: f32,
    ) {
        let display_offset = terminal.grid().display_offset();
        if let Some(ref mut selection) = terminal.selection {
            let location =
                Self::selection_point(x, y, &self.size, display_offset);
            selection.update(location, self.selection_side(x));
        }
    }

    pub fn selection_point(
        x: f32,
        y: f32,
        terminal_size: &TerminalSize,
        display_offset: usize,
    ) -> Point {
        let col = (x as usize) / (terminal_size.cell_width as usize);
        let col = min(Column(col), Column(terminal_size.num_cols as usize - 1));

        let line = (y as usize) / (terminal_size.cell_height as usize);
        let line = min(line, terminal_size.num_lines as usize - 1);

        viewport_to_point(display_offset, Point::new(line, col))
    }

    fn selection_side(&self, x: f32) -> Side {
        let cell_x = x as usize % self.size.cell_width as usize;
        let half_cell_width = (self.size.cell_width as f32 / 2.0) as usize;

        if cell_x > half_cell_width {
            Side::Right
        } else {
            Side::Left
        }
    }

    fn resize(
        &mut self,
        terminal: &mut Term<EventProxy>,
        layout_size: Option<Size<f32>>,
        font_measure: Option<Size<f32>>,
    ) {
        if let Some(size) = layout_size {
            self.size.layout_height = size.height;
            self.size.layout_width = size.width;
        };

        if let Some(size) = font_measure {
            self.size.cell_height = size.height as u16;
            self.size.cell_width = size.width as u16;
        }

        let lines = (self.size.layout_height / self.size.cell_height as f32)
            .floor() as u16;
        // The scrollbar has its own strip on the right: text never runs under it.
        let cols = ((self.size.layout_width - crate::view::SCROLLBAR_WIDTH).max(0.0)
            / self.size.cell_width as f32)
            .floor() as u16;
        if lines > 0 && cols > 0 {
            self.size.num_lines = lines;
            self.size.num_cols = cols;
            self.notifier.on_resize(self.size.into());
            terminal.resize(TermSize::new(
                self.size.num_cols as usize,
                self.size.num_lines as usize,
            ));
        }
    }

    fn write<I: Into<Cow<'static, [u8]>>>(&self, input: I) {
        self.notifier.notify(input);
    }

    fn scroll(&mut self, terminal: &mut Term<EventProxy>, delta_value: i32) {
        if delta_value != 0 {
            let scroll = Scroll::Delta(delta_value);
            if terminal
                .mode()
                .contains(TermMode::ALTERNATE_SCROLL | TermMode::ALT_SCREEN)
            {
                let line_cmd = if delta_value > 0 { b'A' } else { b'B' };
                let mut content = vec![];

                for _ in 0..delta_value.abs() {
                    content.push(0x1b);
                    content.push(b'O');
                    content.push(line_cmd);
                }

                self.notifier.notify(content);
            } else {
                terminal.grid_mut().scroll_display(scroll);
            }
        }
    }

    /// The last `max_lines` lines of text (history and screen), without colors; trailing blank lines dropped.
    pub fn text(&self, max_lines: usize) -> String {
        let term = self.term.lock();
        Self::last_lines(&term, max_lines)
    }

    fn last_lines<T: EventListener>(term: &Term<T>, max_lines: usize) -> String {
        if max_lines == 0 {
            return String::new();
        }
        let bottom = term.bottommost_line();
        let top = Line((bottom.0 - max_lines as i32 + 1).max(term.topmost_line().0));
        let last_column = Column(term.columns().saturating_sub(1));
        let text = term.bounds_to_string(Point::new(top, Column(0)), Point::new(bottom, last_column));
        let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
        let end = lines.iter().rposition(|l| !l.is_empty()).map_or(0, |i| i + 1);
        lines[..end].join("\n")
    }

    pub fn selectable_content(&self) -> String {
        let content = self.renderable_content();
        let mut result = String::new();
        if let Some(range) = content.selectable_range {
            for indexed in content.grid.display_iter() {
                if range.contains(indexed.point) {
                    result.push(indexed.c);
                }
            }
        }
        result
    }

    pub fn sync(&mut self) {
        let term = self.term.clone();
        let mut term = term.lock();
        self.internal_sync(&mut term);
    }

    fn internal_sync(&mut self, terminal: &mut Term<EventProxy>) {
        let selectable_range = match &terminal.selection {
            Some(s) => s.to_range(terminal),
            None => None,
        };

        let cursor = terminal.grid_mut().cursor_cell().clone();
        self.last_content.grid = terminal.grid().clone();
        self.last_content.selectable_range = selectable_range;
        self.last_content.cursor = cursor.clone();
        self.last_content.terminal_mode = *terminal.mode();
        self.last_content.terminal_size = self.size;
    }

    /// The area the terminal was last sized for (the widget compares it with its own layout).
    pub fn layout_size(&self) -> Size<f32> {
        Size::new(self.size.layout_width, self.size.layout_height)
    }

    pub fn renderable_content(&self) -> &RenderableContent {
        &self.last_content
    }
}

pub struct RenderableContent {
    pub grid: Grid<Cell>,
    pub hovered_hyperlink: Option<RangeInclusive<Point>>,
    /// Where the hovered link leads (shown while `Ctrl` is held, IDE TEA RFC 0025).
    pub hovered_link: Option<Link>,
    pub selectable_range: Option<SelectionRange>,
    pub cursor: Cell,
    pub terminal_mode: TermMode,
    pub terminal_size: TerminalSize,
}

impl Default for RenderableContent {
    fn default() -> Self {
        Self {
            grid: Grid::new(0, 0, 0),
            hovered_hyperlink: None,
            hovered_link: None,
            selectable_range: None,
            cursor: Cell::default(),
            terminal_mode: TermMode::empty(),
            terminal_size: TerminalSize::default(),
        }
    }
}

/// The characters of the logical line through `line` (screen lines joined where the text wrapped), with the cell
/// of each. Wide characters count once. At most 16 screen lines on each side.
fn logical_line(grid: &Grid<Cell>, line: Line) -> Vec<(Point, char)> {
    let last = grid.last_column();
    let wraps = |l: Line| grid[l][last].flags.contains(Flags::WRAPLINE);
    let mut first = line;
    while first > grid.topmost_line() && line.0 - first.0 < 16 && wraps(Line(first.0 - 1)) {
        first = Line(first.0 - 1);
    }
    let mut end = line;
    while end < grid.bottommost_line() && end.0 - line.0 < 16 && wraps(end) {
        end = Line(end.0 + 1);
    }
    let mut cells = Vec::new();
    for l in first.0..=end.0 {
        for c in 0..grid.columns() {
            let point = Point::new(Line(l), Column(c));
            let cell = &grid[point];
            if cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
                continue;
            }
            cells.push((point, cell.c));
        }
    }
    cells
}

impl Drop for Backend {
    fn drop(&mut self) {
        let _ = self.notifier.0.send(Msg::Shutdown);
    }
}

#[derive(Clone)]
pub struct EventProxy(mpsc::Sender<Event>);

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let _ = self.0.blocking_send(event);
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

    fn term_with(lines: &[&str]) -> Term<VoidListener> {
        let size = TermSize::new(20, 4);
        let mut term = Term::new(term::Config::default(), &size, VoidListener);
        let mut parser: Processor<StdSyncHandler> = Processor::new();
        for line in lines {
            parser.advance(&mut term, format!("{line}\r\n").as_bytes());
        }
        term
    }

    fn selected(term: &Term<VoidListener>) -> Option<String> {
        term.selection_to_string()
    }

    #[test]
    fn last_lines_take_history_and_drop_trailing_blanks() {
        let term = term_with(&["um", "dois  ", "tres", "quatro", "cinco"]);
        assert_eq!(Backend::last_lines(&term, 100), "um\ndois\ntres\nquatro\ncinco");
        assert_eq!(Backend::last_lines(&term, 4), "tres\nquatro\ncinco", "a linha em branco do cursor conta");
        assert_eq!(Backend::last_lines(&term, 0), "");
    }

    #[test]
    fn finds_newest_first_then_older_then_newer() {
        let mut term = term_with(&["erro 1", "ok", "ok", "erro 2", "ok", "ok"]);
        assert!(Backend::search(&mut term, "erro", true));
        assert_eq!(selected(&term).as_deref(), Some("erro"));
        let newest = term.selection.as_ref().unwrap().to_range(&term).unwrap().start;
        assert!(Backend::search(&mut term, "erro", true));
        let older = term.selection.as_ref().unwrap().to_range(&term).unwrap().start;
        assert!(older.line < newest.line, "subiu para a mais antiga: {older:?} < {newest:?}");
        assert!(Backend::search(&mut term, "erro", false));
        assert_eq!(term.selection.as_ref().unwrap().to_range(&term).unwrap().start, newest);
    }

    #[test]
    fn plain_text_and_smart_case() {
        let mut term = term_with(&["a.b axb", "Foo"]);
        assert!(Backend::search(&mut term, "a.b", true));
        assert_eq!(selected(&term).as_deref(), Some("a.b"));
        assert!(Backend::search(&mut term, "foo", true), "minúsculas: sem diferenciar");
        assert!(!Backend::search(&mut term, "FOO", true), "com maiúscula: diferencia");
        assert!(!Backend::search(&mut term, "", true));
    }
}
