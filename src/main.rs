mod text;
use std::{cell::RefCell, io, rc::Rc};

use ratatui::{
    layout::{Alignment, Flex},
    prelude::*,
    style::{Color, Stylize},
    symbols::scrollbar,
    widgets::*,
    Frame, Terminal,
};

use ratzilla::{event::KeyCode, DomBackend, WebRenderer};

use web_time::{Duration, Instant};

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(log::Level::Debug);

    App::run(Rc::new(RefCell::new(App::default())))
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
enum AppStatus {
    #[default]
    IntroductionStart,
    Introduction(usize),
    IntroductionIdle,
    List,
}

impl AppStatus {
    const fn max_introduction() -> usize {
        6
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
enum Background {
    #[default]
    First,
    Second,
    Third,
}

impl Background {
    const fn pastel_orange() -> Color {
        Color::Rgb(255, 184, 108)
    }

    const fn electric() -> Color {
        Color::Rgb(139, 233, 253)
    }

    const fn pastel_pink() -> Color {
        Color::Rgb(255, 121, 198)
    }

    const fn colors(self) -> [Color; 3] {
        use Background::*;
        match self {
            First => [
                Background::pastel_orange(),
                Background::electric(),
                Background::pastel_pink(),
            ],
            Second => [
                Background::electric(),
                Background::pastel_pink(),
                Background::pastel_orange(),
            ],
            Third => [
                Background::pastel_pink(),
                Background::pastel_orange(),
                Background::electric(),
            ],
        }
    }

    fn render(self, frame: &mut Frame) {
        let colors = self.colors();

        let [upper_area, middle_area, lower_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(frame.area());

        let [upper_left_area, upper_center_area, upper_right_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(upper_area);

        let [middle_left_area, middle_center_area, middle_right_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(middle_area);

        let [lower_left_area, lower_center_area, lower_right_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(lower_area);

        frame.render_widget(Block::default().bg(colors[0]), upper_left_area);
        frame.render_widget(Block::default().bg(colors[1]), upper_center_area);
        frame.render_widget(Block::default().bg(colors[2]), upper_right_area);

        frame.render_widget(Block::default().bg(colors[1]), middle_left_area);
        frame.render_widget(Block::default().bg(colors[2]), middle_center_area);
        frame.render_widget(Block::default().bg(colors[0]), middle_right_area);

        frame.render_widget(Block::default().bg(colors[2]), lower_left_area);
        frame.render_widget(Block::default().bg(colors[0]), lower_center_area);
        frame.render_widget(Block::default().bg(colors[1]), lower_right_area);
    }

    fn next(self) -> Self {
        use Background::*;

        match self {
            First => Second,
            Second => Third,
            Third => First,
        }
    }
}

#[derive(Debug)]
struct App {
    title: &'static str,
    status: AppStatus,
    last_instant: Instant,
    intro_finalized: bool,
    list_state: ListState,
    locked_in: bool,
    scrollbar_state: ScrollbarState,
    scroll: u16,
    background: Background,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: text::TARBETU,
            status: AppStatus::default(),
            last_instant: Instant::now(),
            intro_finalized: false,
            list_state: ListState::default().with_selected(Some(0)),
            scrollbar_state: ScrollbarState::default(),
            scroll: 0,
            locked_in: false,
            background: Background::default(),
        }
    }
}

impl App {
    const fn menu_length() -> usize {
        7
    }

    // fn menu() -> Vec<ListItem<'static>> {
    fn menu() -> [ListItem<'static>; App::menu_length()] {
        [
            ListItem::new("./tarbetu"),
            ListItem::new("./portfolio"),
            ListItem::new("./translations"),
            ListItem::new("./lycian"),
            ListItem::new("./personal_soundtrack"),
            ListItem::new("./echoes_from_my_mania"),
            ListItem::new("./kara_tilki_hiyerarsisi"),
        ]
    }

    fn run(app: Rc<RefCell<Self>>) -> io::Result<()> {
        let backend = DomBackend::new()?;
        let terminal = Terminal::new(backend)?;

        app.borrow_mut().last_instant = Instant::now();

        let event_app = app.clone();
        terminal.on_key_event(move |event| {
            use AppStatus::*;

            let mut app = event_app.borrow_mut();
            match app.status {
                List => {
                    app.handle_event(event.code)
                    // app.list_state.select_next();
                }
                _ => {
                    app.intro_finalized = true;
                    app.status = AppStatus::List;
                }
            }
        });

        let render_app = app.clone();
        terminal.draw_web(move |frame| {
            let mut app = render_app.borrow_mut();
            let next_status = app.next_status();

            if !app.intro_finalized && app.last_instant.elapsed() >= Duration::from_millis(500) {
                if next_status == app.status {
                    app.intro_finalized = true;
                }

                app.status = app.next_status();
                app.last_instant = Instant::now();
            }

            if app.intro_finalized
                && app.status == AppStatus::IntroductionIdle
                && app.last_instant.elapsed() >= Duration::from_secs(2)
            {
                app.status = AppStatus::Introduction(AppStatus::max_introduction() - 1);
                app.intro_finalized = false;
                app.last_instant = Instant::now()
            }

            if app.intro_finalized && app.last_instant.elapsed() >= Duration::from_millis(500) {
                app.background = app.background.next();
                app.last_instant = Instant::now()
            }

            app.render(frame);
        });
        Ok(())
    }

    fn handle_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter if !self.locked_in => self.locked_in = true,
            KeyCode::Esc if self.locked_in => self.locked_in = false,
            KeyCode::Char('t') if self.title == text::TARBETU => {
                self.title = text::TARBETU1;
            }
            KeyCode::Char('a') if self.title == text::TARBETU1 => {
                self.title = text::TARBETU2;
            }
            KeyCode::Char('r') if self.title == text::TARBETU2 => {
                self.title = text::TARBETU3;
            }
            KeyCode::Char('b') if self.title == text::TARBETU3 => {
                self.title = text::TARBETU4;
            }
            KeyCode::Char('e') if self.title == text::TARBETU4 => {
                self.title = text::TARBETU5;
            }
            KeyCode::Char('t') if self.title == text::TARBETU5 => {
                self.title = text::TARBETU6;
            }
            KeyCode::Char('u') if self.title == text::TARBETU6 => {
                self.title = text::TARBETU7;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.locked_in {
                    self.scroll = self.scroll.saturating_sub(1);
                } else {
                    self.scroll = 0;
                    self.list_state.select_previous();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.locked_in {
                    self.scroll = self.scroll.saturating_add(1);
                } else {
                    self.scroll = 0;
                    self.list_state.select_next();
                }
            }
            _ => {}
        }

        self.scrollbar_state = self.scrollbar_state.position(self.scroll as usize);
    }

    fn render<'a>(&mut self, frame: &mut Frame<'a>) {
        use AppStatus::*;

        match self.status {
            IntroductionStart => {
                self.render_introduction(frame, text::NAME1, Color::Cyan);
            }
            Introduction(0) => {
                self.render_introduction(frame, text::NAME1, Color::LightCyan);
            }
            Introduction(1) => {
                self.render_introduction(frame, text::NAME2, Color::Yellow);
            }
            Introduction(2) => {
                self.render_introduction(frame, text::NAME2, Color::LightYellow);
            }
            Introduction(3) => {
                self.render_introduction(frame, text::NAME3, Color::Red);
            }
            Introduction(4) => {
                self.render_introduction(frame, text::NAME3, Color::LightRed);
            }
            Introduction(5) | IntroductionIdle => {
                self.render_introduction(frame, text::PRESS_ANY_KEY, Color::LightGreen);
            }
            Introduction(6) => {
                self.render_introduction(frame, text::PRESS_ANY_KEY, Color::Green);
            }
            List => {
                self.background.render(frame);
                self.render_list_view(frame);
            }
            _ => {
                self.render_introduction(frame, text::PRESS_ANY_KEY, Color::Green);
            }
        }
    }

    fn render_introduction<'a>(&self, frame: &mut Frame<'a>, text: &'a str, color: Color) {
        let ascii_art = Text::from(
            text.split("\n")
                .map(|line| Line::from(line.fg(color)))
                .collect::<Vec<Line>>(),
        );

        let area = App::center(
            frame.area(),
            Constraint::Length(ascii_art.width() as u16),
            Constraint::Percentage(50),
        );

        frame.render_widget(ascii_art, area);
    }

    fn next_status(&self) -> AppStatus {
        use AppStatus::*;

        match self.status {
            IntroductionStart => Introduction(0),
            Introduction(number) if number < AppStatus::max_introduction() => {
                Introduction(number + 1)
            }
            Introduction(_) => IntroductionIdle,
            _ => self.status,
        }
    }

    fn render_list_view(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [header_area, main_area, _, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [_, list_area, _, content_area, _] = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Length(30),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(main_area);

        self.clear_areas(frame, &[list_area, content_area]);

        self.render_header(frame, header_area);
        self.render_footer(frame, footer_area);
        self.render_list(frame, list_area);
        self.render_content(frame, content_area);
    }

    fn clear_areas(&self, frame: &mut Frame, areas: &[Rect]) {
        for area in areas {
            frame.render_widget(Clear, *area);
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new(self.title).bold().centered(), area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(if !self.locked_in {
                "Use ↓↑ or j/k to navigate, Enter to locked in"
            } else {
                "Use ↓↑ or j/k to scroll, Esc to return menu"
            })
            .centered(),
            area,
        )
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let list_block = Block::bordered()
            .border_set(if !self.locked_in {
                symbols::border::QUADRANT_OUTSIDE
            } else {
                symbols::border::EMPTY
            })
            .border_style(Style::default().fg(Color::LightMagenta))
            .bg(Color::Rgb(15, 15, 20))
            .fg(if !self.locked_in {
                Color::LightCyan
            } else {
                Color::Cyan
            });

        let list = List::new(App::menu())
            .block(list_block)
            .highlight_style(Style::default().fg(Color::LightMagenta))
            .highlight_symbol("▶ ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        let content_block = Block::bordered()
            .border_set(if self.locked_in {
                symbols::border::QUADRANT_OUTSIDE
            } else {
                symbols::border::EMPTY
            })
            .border_style(Style::default().fg(Color::LightMagenta))
            .bg(Color::Rgb(15, 15, 20))
            .padding(Padding::new(1, 2, 0, 0))
            .fg(Color::LightCyan);

        self.render_text(
            frame,
            content_block,
            area,
            match self.list_state.selected() {
                Some(0) => text::ABOUT,
                Some(1) => text::PORTFOLIO,
                Some(2) => text::TRANSLATIONS,
                Some(3) => text::LYCIAN_PROJECT,
                Some(4) => text::MUSIC,
                Some(5) => text::ECHOES,
                Some(6) => text::KTH,
                _ => "",
            },
        );
    }

    fn render_text(&mut self, frame: &mut Frame, block: Block, area: Rect, text: &'static str) {
        let lines: Vec<Line> = text
            .split('\n')
            .map(|line| {
                if line.starts_with("http") {
                    Line::from(
                        Span::from(line)
                            .fg(Color::LightBlue)
                            .style(Modifier::SLOW_BLINK),
                    )
                } else {
                    Line::from(line)
                }
            })
            .collect();

        self.scrollbar_state = self.scrollbar_state.content_length(lines.len());

        let text = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        frame.render_widget(text, area);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight).symbols(scrollbar::VERTICAL),
            area,
            &mut self.scrollbar_state,
        );
    }

    /// Centers a [`Rect`] within another [`Rect`] using the provided [`Constraint`]s.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ratatui::layout::{Constraint, Rect};
    ///
    /// let area = Rect::new(0, 0, 100, 100);
    /// let horizontal = Constraint::Percentage(20);
    /// let vertical = Constraint::Percentage(30);
    ///
    /// let centered = center(area, horizontal, vertical);
    /// ```
    fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
        let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
        area
    }
}
