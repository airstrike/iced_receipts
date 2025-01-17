use iced::event;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::widget::focus_next;
use iced::{Element, Size, Subscription, Task};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

mod common;
mod list;
mod sale;
mod tax;

pub use common::Action;
use sale::Sale;

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .window_size(Size::new(800.0, 600.0))
        .theme(App::theme)
        .antialiasing(true)
        .centered()
        .subscription(App::subscription)
        .run_with(App::new)
}

#[derive(Debug)]
enum Screen {
    List,
    Sale(sale::Mode, Option<usize>),
}

#[derive(Debug)]
enum Message {
    List(list::Message),
    Sale(Option<usize>, sale::Message),
    Hotkey(Hotkey),
}

#[derive(Debug)]
enum Operation {
    Sale(Option<usize>, sale::Operation),
}

struct App {
    screen: Screen,
    sales: HashMap<usize, sale::Sale>,
    draft: (Option<usize>, sale::Sale),
    next_sale_id: AtomicUsize,
}

impl App {
    fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }

    fn title(&self) -> String {
        match self.screen {
            Screen::List => "iced Receipts".to_string(),
            Screen::Sale(mode, id) => {
                let sale_name = if self.draft.0 == id {
                    self.draft.1.name.clone()
                } else {
                    self.sales[&id.unwrap()].name.clone()
                };

                let sale_name = format!(
                    "{} {}",
                    if sale_name.is_empty() {
                        "Untitled sale"
                    } else {
                        &sale_name
                    },
                    id.map_or("".to_string(), |id| format!("(#{id})"))
                );

                match mode {
                    sale::Mode::View => format!("iced Receipts • {}", sale_name),
                    sale::Mode::Edit => format!("iced Receipts • {} • Edit", sale_name),
                }
            }
        }
    }

    fn new() -> (Self, Task<Message>) {
        let initial_id = 0;
        (
            Self {
                screen: Screen::List,
                sales: HashMap::new(),
                draft: (None, Sale::default()),
                next_sale_id: AtomicUsize::new(initial_id + 1),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::List(list::Message::NewSale) => {
                self.draft = (None, Sale::default());
                self.screen = Screen::Sale(sale::Mode::Edit, None);
                return focus_next();
            }
            Message::List(list::Message::SelectSale(id)) => {
                self.screen = Screen::Sale(sale::Mode::View, Some(id));
            }
            Message::Hotkey(hotkey) => match self.screen {
                Screen::List => {}
                Screen::Sale(mode, sale_id) => {
                    let sale = if self.draft.0 == sale_id {
                        &mut self.draft.1
                    } else {
                        self.sales
                            .get_mut(&sale_id.unwrap())
                            .expect("Sale should exist")
                    };

                    let action = sale::handle_hotkey(sale, mode, hotkey)
                        .map_operation(move |o| Operation::Sale(sale_id, o))
                        .map(move |m| Message::Sale(sale_id, m));

                    let operation_task = if let Some(operation) = action.operation {
                        self.perform(operation)
                    } else {
                        Task::none()
                    };

                    return operation_task.chain(action.task);
                }
            },
            Message::Sale(sale_id, msg) => {
                let sale = if self.draft.0 == sale_id {
                    &mut self.draft.1
                } else {
                    self.sales
                        .get_mut(&sale_id.unwrap())
                        .expect("Sale should exist")
                };

                let action = sale::update(sale, msg)
                    .map_operation(move |o| Operation::Sale(sale_id, o))
                    .map(move |m| Message::Sale(sale_id, m));

                let operation_task = if let Some(operation) = action.operation {
                    self.perform(operation)
                } else {
                    Task::none()
                };

                return operation_task.chain(action.task);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::List => list::view(&self.sales).map(Message::List),
            Screen::Sale(mode, id) => {
                let sale = if self.draft.0 == *id {
                    &self.draft.1
                } else {
                    &self.sales[&id.unwrap()]
                };
                sale::view(sale, *mode).map(|msg| Message::Sale(*id, msg))
            }
        }
    }

    fn perform(&mut self, operation: Operation) -> Task<Message> {
        match operation {
            Operation::Sale(sale_id, operation) => match operation {
                sale::Operation::Back => match self.screen {
                    Screen::List => {}
                    Screen::Sale(mode, _) => match mode {
                        sale::Mode::Edit => self.screen = Screen::Sale(sale::Mode::View, sale_id),
                        sale::Mode::View => self.screen = Screen::List,
                    },
                },
                sale::Operation::Save => {
                    let final_id = match self.draft.0 {
                        Some(id) => {
                            // Editing existing sale
                            self.sales.insert(id, self.draft.1.clone());
                            id
                        }
                        None => {
                            // Creating new sale
                            let new_id = self.next_sale_id.fetch_add(1, Ordering::SeqCst);
                            self.sales.insert(new_id, std::mem::take(&mut self.draft.1));
                            self.draft.1 = Sale::default();
                            new_id
                        }
                    };
                    self.screen = Screen::Sale(sale::Mode::View, Some(final_id));
                }
                sale::Operation::StartEdit => {
                    if let Some(id) = sale_id {
                        // Start editing existing sale
                        self.draft = (Some(id), self.sales[&id].clone());
                    }
                    self.screen = Screen::Sale(sale::Mode::Edit, sale_id);
                }
                sale::Operation::Cancel => {
                    match sale_id {
                        Some(id) => {
                            // Restore draft from original sale
                            self.draft = (Some(id), self.sales[&id].clone());
                        }
                        None => {
                            // Reset to blank draft
                            self.draft = (None, Sale::default());
                        }
                    }
                    self.screen = Screen::Sale(sale::Mode::View, sale_id);
                }
            },
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(handle_event)
    }
}

#[derive(Debug)]
pub enum Hotkey {
    Escape,
    Tab(Modifiers),
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
            Key::Named(Named::Escape) => Some(Message::Hotkey(Hotkey::Escape)),
            Key::Named(Named::Tab) => Some(Message::Hotkey(Hotkey::Tab(modifiers))),
            _ => None,
        },
        _ => None,
    }
}
