#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::collections::{HashMap, HashSet};

use iced::{
    advanced::graphics::core::window,
    application::{Appearance, StyleSheet},
    theme::{self, Button},
    widget::{self, button, container, horizontal_space, row, text, themer, Container, Scrollable},
    Application, Length, Settings, Theme,
};

#[derive(Debug, Clone)]
enum Message {
    WindowResized(iced::Size),
    WindowMoved(iced::window::Position),
    WindowClosed(iced::window::Id),
    ClickMe,
    Hover(usize),
}

#[derive(Debug)]
struct WindowPos {
    pos: iced::window::Position,
    size: iced::Size,
}

impl WindowPos {
    const DEFAULT: Self = Self {
        pos: iced::window::Position::Centered,
        size: iced::Size::new(800.0, 600.0),
    };

    fn save(&self) -> String {
        match self.pos {
            window::Position::Default => {
                format!(
                    "default\n{width}, {height}",
                    width = self.size.width as i32,
                    height = self.size.height as i32,
                )
            }
            window::Position::Centered => {
                format!(
                    "centered\n{width}, {height}",
                    width = self.size.width as i32,
                    height = self.size.height as i32,
                )
            }
            window::Position::Specific(iced::Point { x, y }) => {
                format!(
                    "{x}, {y}\n{width}, {height}",
                    width = self.size.width as i32,
                    height = self.size.height as i32,
                    x = x as i32,
                    y = y as i32
                )
            }
        }
    }

    fn load(data: &str) -> Self {
        enum Pos {
            Centered,
            Default,
            Specific(i32, i32),
        }

        impl std::str::FromStr for Pos {
            type Err = &'static str;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                match input {
                    s if s.eq_ignore_ascii_case("default") => Ok(Self::Centered),
                    s if s.eq_ignore_ascii_case("centered") => Ok(Self::Default),
                    s => {
                        let mut iter = s.split_terminator(", ").map(|s| s.parse().ok()).flatten();
                        let x = iter.next();
                        let y = iter.next();

                        x.and_then(|x| Some((x, y?)))
                            .ok_or_else(|| "invalid point")
                            .map(|(x, y)| Self::Specific(x, y))
                    }
                }
            }
        }

        let mut lines = data.lines();
        let Some(pos) = lines.next() else {
            return Self::DEFAULT;
        };
        let pos = pos.parse::<Pos>().unwrap_or(Pos::Centered);

        let Some(line) = lines.next() else {
            return Self::DEFAULT;
        };

        let mut iter = line
            .split_terminator(", ")
            .flat_map(|s| s.parse::<i32>().ok());
        let x = iter.next();
        let y = iter.next();

        let size = x
            .and_then(|x| Some((x, y?)))
            .map(|(x, y)| iced::Size::new(x as f32, y as f32))
            .unwrap_or(Self::DEFAULT.size);

        Self {
            pos: match pos {
                Pos::Centered => iced::window::Position::Centered,
                Pos::Default => iced::window::Position::Default,
                Pos::Specific(x, y) => {
                    iced::window::Position::Specific(iced::Point::new(x as _, y as _))
                }
            },
            size,
        }
    }
}

impl Default for WindowPos {
    fn default() -> Self {
        Self {
            pos: iced::window::Position::Centered,
            size: iced::Size::new(800.0, 600.0),
        }
    }
}

#[derive(Default)]
struct FontThingyState {
    fonts: Fonts,
    window_pos: WindowPos,
}

struct FontThingy {
    state: FontThingyState,
    hovered: Option<usize>,
}

impl FontThingy {
    // const PLACEHOLDER: &'static str = "The lazy brown dog jumped over the fence";
    const PLACEHOLDER: &'static str = "Blocking waiting for file lock on build directory";
}

impl Application for FontThingy {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = FontThingyState;

    fn new(state: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                state,
                hovered: None,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Font thingy")
    }

    fn theme(&self) -> Self::Theme {
        Theme::KanagawaWave
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::WindowClosed(id) => {
                let _ = std::fs::write("settings.txt", self.state.window_pos.save());
                return iced::window::close(id);
            }
            Message::WindowResized(size) => self.state.window_pos.size = size,
            Message::WindowMoved(pos) => self.state.window_pos.pos = pos,
            Message::ClickMe => {
                self.hovered.take();
                self.state.fonts.rebuild()
            }
            Message::Hover(i) => {
                self.hovered = Some(i);
                // eprintln!("hovered: {:#?}", self.state.fonts.fonts.get(i));
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        widget::column([
            widget::container(widget::text(Self::PLACEHOLDER).size(32))
                .width(Length::Fill)
                .center_x()
                .center_y()
                .height(Length::FillPortion(2))
                .into(),
            widget::button("click me").on_press(Message::ClickMe).into(),
            Container::new(
                Scrollable::new(widget::lazy(&self.state.fonts.fonts, |fonts| {
                    widget::Column::with_children(fonts.iter().enumerate().map(|(i, font)| {
                        widget::mouse_area(font.as_list_element())
                            .on_enter(Message::Hover(i))
                            .into()
                    }))
                }))
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
            )
            .center_x()
            .into(),
        ])
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::event::listen_with(|ev, _| match ev {
            iced::Event::Window(id, window::Event::CloseRequested) => {
                Some(Message::WindowClosed(id))
            }
            iced::Event::Window(_id, window::Event::Resized { width, height }) => Some(
                Message::WindowResized(iced::Size::new(width as _, height as _)),
            ),
            iced::Event::Window(_id, window::Event::Moved { x, y }) => Some(Message::WindowMoved(
                iced::window::Position::Specific(iced::Point::new(x as _, y as _)),
            )),
            _ => None,
        })
    }
}

fn main() -> iced::Result {
    let window_pos = std::fs::read_to_string("settings.txt")
        .ok()
        .map(|data| WindowPos::load(&data))
        .unwrap_or(WindowPos {
            pos: iced::window::Position::Centered,
            size: iced::Size::new(800.0, 600.0),
        });

    FontThingy::run(Settings {
        window: window::Settings {
            exit_on_close_request: false,
            size: window_pos.size,
            position: window_pos.pos,
            ..window::Settings::default()
        },
        ..Default::default()
    })
}

type FontId = iced::advanced::graphics::text::cosmic_text::fontdb::ID;

#[derive(Debug, Clone, PartialOrd, Hash)]
struct Font {
    id: FontId,
    name: String,
}

impl Font {
    fn as_list_element<'a, M: Clone + 'static>(&self) -> iced::Element<'static, M> {
        row([
            horizontal_space().into(),
            text(&self.name)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .into(),
            horizontal_space().into(),
        ])
        .into()
    }
}

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

#[derive(Default, PartialEq, PartialOrd, Hash)]
struct Fonts {
    fonts: Vec<Font>,
}

impl ToString for Font {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl Fonts {
    fn rebuild(&mut self) {
        self.fonts.clear();

        let system = iced::advanced::graphics::text::font_system();
        let mut raw = system.write().unwrap();
        let raw = raw.raw();
        let db = raw.db();

        let mut seen = HashSet::new();
        for face in db.faces() {
            let id = face.id;
            for (name, _) in &face.families {
                if seen.insert(name) {
                    self.fonts.push(Font {
                        id,
                        name: name.clone(),
                    });
                }
            }
        }
    }
}
