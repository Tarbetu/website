mod text;
use std::{cell::RefCell, io, rc::Rc};

use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    layout::{Alignment, Flex},
    prelude::*,
    style::{Color, Stylize},
    symbols::border,
    widgets::*,
    Frame, Terminal,
};

use ratzilla::{
    event::{KeyCode, KeyEvent},
    DomBackend, WebRenderer,
};

use web_time::{Duration, Instant};

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(log::Level::Debug);

    App::run(Rc::new(RefCell::new(App::default())))
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
enum ListStatus {
    #[default]
    About = 0,
    Portfolio,
    Whoami, // List some hobbies and interests
    LycianProject,
    Interests,
    Music,
    EchoesFromMyMania,
    KaraTilkiHiyerarsisi,
    TechnicalDetails,
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
    fn max_introduction() -> usize {
        6
    }
}

#[derive(Debug)]
struct App {
    status: AppStatus,
    last_instant: Instant,
    intro_finalized: bool,
    list_status: ListStatus,
    list_state: ListState,
    locked_in: bool,
    scrollbar_state: ScrollbarState,
    scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            status: AppStatus::default(),
            last_instant: Instant::now(),
            intro_finalized: false,
            list_state: ListState::default().with_selected(Some(0)),
            list_status: ListStatus::default(),
            scrollbar_state: ScrollbarState::default(),
            scroll: 0,
            locked_in: false,
        }
    }
}

impl App {
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

            app.render(frame);
        });
        Ok(())
    }

    fn handle_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter if !self.locked_in => self.locked_in = true,
            KeyCode::Esc if self.locked_in => self.locked_in = false,
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.select_next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.select_previous();
            }
            _ => {}
        }
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
                self.render_welcoming(frame);
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
            IntroductionIdle => IntroductionIdle,
            _ => self.status,
        }
    }

    fn render_welcoming(&mut self, frame: &mut Frame) {
        self.render_list_view(frame);
        // self.render_introduction(frame, "Welcome", Color::Red);
    }

    fn render_list_view(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(main_area);

        App::render_header(frame, header_area);
        self.render_footer(frame, footer_area);
        self.render_list(frame, list_area);
        // self.render_selected_item(item_area);
    }

    fn render_header(frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new(text::TARBETU).bold().centered(), area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(if !self.locked_in {
                "Use ↓↑ to move, Enter to locked in"
            } else {
                "Use ↓↑ to move, Esc to return menu"
            })
            .centered(),
            area,
        )
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let list_block = Block::new()
            .borders(Borders::ALL)
            .border_set(if !self.locked_in {
                symbols::border::DOUBLE
            } else {
                symbols::border::EMPTY
            })
            .on_dark_gray()
            .fg(if !self.locked_in {
                Color::White
            } else {
                Color::Gray
            });

        let items = vec![
            ListItem::new("About"),
            ListItem::new("Portfolio"),
            ListItem::new("Whoami"),
            ListItem::new("Lycian Project"),
            ListItem::new("Interests"),
            ListItem::new("Some music"),
            ListItem::new("Echoes from my mania"),
            ListItem::new("Kara Tilki Hiyerarşisi"),
            ListItem::new("Technical Details"),
        ];

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Style::default().fg(Color::Magenta))
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, area, &mut self.list_state);
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
