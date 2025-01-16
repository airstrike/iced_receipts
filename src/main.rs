use iced::event;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::{Element, Size, Subscription, Task};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

mod action;
mod list;
mod sale;
mod tax;

pub use action::Action;
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
    Sale(sale::Mode, usize),
}

#[derive(Debug)]
enum Message {
    List(list::Message),
    Sale(usize, sale::Message),
    Hotkey(Hotkey),
}

#[derive(Debug)]
enum Operation {
    Sale(usize, sale::Operation),
}

struct App {
    screen: Screen,
    sales: HashMap<usize, sale::Sale>,
    pending_sale: (usize, sale::Sale),
    next_sale_id: AtomicUsize,
}

impl App {
    fn theme(&self) -> iced::Theme {
        iced::Theme::Light
    }

    fn title(&self) -> String {
        match self.screen {
            Screen::List => "iced • Receipt Breakdown".to_string(),
            Screen::Sale(mode, id) => {
                let sale_name = if id == self.pending_sale.0 {
                    "New Sale".to_string()
                } else {
                    self.sales[&id].name.clone()
                };
                match mode {
                    sale::Mode::View => format!("iced • {}", sale_name),
                    sale::Mode::Edit => format!("iced • {} • Edit", sale_name),
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
                pending_sale: (initial_id, Sale::default()),
                next_sale_id: AtomicUsize::new(initial_id + 1),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::List(list::Message::NewSale) => {
                self.screen = Screen::Sale(sale::Mode::Edit, self.pending_sale.0);
            }
            Message::List(list::Message::SelectSale(id)) => {
                self.screen = Screen::Sale(sale::Mode::View, id);
            }
            Message::Hotkey(hotkey) => match self.screen {
                Screen::List => {}
                Screen::Sale(mode, sale_id) => {
                    let sale = if sale_id == self.pending_sale.0 {
                        &mut self.pending_sale.1
                    } else {
                        self.sales.get_mut(&sale_id).expect("Sale should exist")
                    };

                    // Let the sale module handle the hotkey and return an Action
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
                let sale = if sale_id == self.pending_sale.0 {
                    &mut self.pending_sale.1
                } else {
                    self.sales.get_mut(&sale_id).expect("Sale should exist")
                };

                // Let the sale module handle the message and return an Action
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
                let sale = if *id == self.pending_sale.0 {
                    &self.pending_sale.1
                } else {
                    &self.sales[id]
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
                    Screen::Sale(mode, sale_id) => match mode {
                        sale::Mode::Edit => self.screen = Screen::Sale(sale::Mode::View, sale_id),
                        sale::Mode::View => self.screen = Screen::List,
                    },
                },
                sale::Operation::Save => {
                    if sale_id == self.pending_sale.0 {
                        // take ownership of the current pending sale
                        // and replace it with a new default blank sale
                        // before inserting this one into the sales map
                        let current_sale =
                            std::mem::replace(&mut self.pending_sale.1, Sale::default());

                        let current_id = std::mem::replace(
                            &mut self.pending_sale.0,
                            self.next_sale_id.fetch_add(1, Ordering::SeqCst),
                        );

                        self.sales.insert(current_id, current_sale);
                        self.screen = Screen::Sale(sale::Mode::View, current_id);
                    } else {
                        self.screen = Screen::Sale(sale::Mode::View, sale_id);
                    }
                }
                sale::Operation::StartEdit => {
                    self.screen = Screen::Sale(sale::Mode::Edit, sale_id);
                }
                sale::Operation::Cancel => {
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
