mod text;
use std::{cell::RefCell, io, rc::Rc};

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
enum AppStatus {
    #[default]
    IntroductionStart,
    Introduction(usize),
    IntroductionIdle,
    Welcoming,
}

impl AppStatus {
    fn max_introduction() -> usize {
        6
    }
}

#[derive(Debug, Copy, Clone)]
struct App {
    status: AppStatus,
    last_instant: Instant,
    intro_finalized: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            status: AppStatus::default(),
            last_instant: Instant::now(),
            intro_finalized: false,
        }
    }
}

impl App {
    fn run(app: Rc<RefCell<Self>>) -> io::Result<()> {
        let backend = DomBackend::new()?;
        let terminal = Terminal::new(backend)?;

        app.borrow_mut().last_instant = Instant::now();

        let event_app = app.clone();
        terminal.on_key_event(move |_| {
            let mut app = event_app.borrow_mut();
            if !app.intro_finalized {
                app.intro_finalized = true;
                app.status = AppStatus::Welcoming;
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

    fn render<'a>(&self, frame: &mut Frame<'a>) {
        use AppStatus::*;

        match self.status {
            IntroductionStart => {
                self.render_introduction(frame, text::NAME1, <&str>::blue);
            }
            Introduction(0) => {
                self.render_introduction(frame, text::NAME1, <&str>::black);
            }
            Introduction(1) => {
                self.render_introduction(frame, text::NAME2, <&str>::green);
            }
            Introduction(2) => {
                self.render_introduction(frame, text::NAME2, <&str>::black);
            }
            Introduction(3) => {
                self.render_introduction(frame, text::NAME3, <&str>::red);
            }
            Introduction(4) => {
                self.render_introduction(frame, text::NAME3, <&str>::black);
            }
            Introduction(5) | IntroductionIdle => {
                self.render_introduction(frame, text::PRESS_ANY_KEY, <&str>::green);
            }
            Introduction(6) => {
                self.render_introduction(frame, text::PRESS_ANY_KEY, <&str>::black);
            }
            Welcoming => {
                self.render_welcoming(frame);
            }
            _ => {
                self.render_introduction(frame, text::PRESS_ANY_KEY, <&str>::green);
            }
        }
    }

    fn render_introduction<'a>(
        &self,
        frame: &mut Frame<'a>,
        text: &'a str,
        style: fn(&'a str) -> Span<'a>,
    ) {
        let ascii_art = Text::from(
            text.split("\n")
                .map(|line| Line::from(style(line)))
                .collect::<Vec<Line>>(),
        );

        let area = App::center(
            frame.area(),
            Constraint::Length(ascii_art.width() as u16),
            Constraint::Percentage(50),
        );

        frame.render_widget(ascii_art, area);
    }

    fn next_status(self) -> AppStatus {
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

    fn render_welcoming(&self, frame: &mut Frame) {
        self.render_introduction(frame, "Welcome", <&str>::red);
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
