//! View and edit sales
use iced::widget::{focus_next, text_input};
use iced::Element;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::tax::TaxGroup;
use crate::{Action, Hotkey};

pub mod edit;
pub mod show;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    View,
    Edit,
}

#[derive(Debug, Clone)]
pub struct SaleItem {
    pub id: usize,
    pub name: String,
    price: Option<f32>,
    quantity: Option<u32>,
    pub tax_group: TaxGroup,
}

impl Default for SaleItem {
    fn default() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name: String::new(),
            price: None,
            quantity: None,
            tax_group: TaxGroup::Food,
        }
    }
}

impl SaleItem {
    pub fn price(&self) -> f32 {
        self.price.unwrap_or(0.0)
    }
    pub fn quantity(&self) -> f32 {
        self.quantity.unwrap_or(0) as f32
    }
    pub fn price_string(&self) -> String {
        self.price.map_or(String::new(), |p| format!("{:.2}", p))
    }
    pub fn quantity_string(&self) -> String {
        self.quantity.map_or(String::new(), |q| q.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Sale {
    pub items: Vec<SaleItem>,
    pub service_charge_percent: Option<f32>,
    pub gratuity_amount: Option<f32>,
    pub name: String,
}

impl Default for Sale {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            service_charge_percent: None,
            gratuity_amount: None,
            name: String::new(),
        }
    }
}

impl Sale {
    pub fn calculate_subtotal(&self) -> f32 {
        self.items
            .iter()
            .map(|item| item.price() * item.quantity())
            .sum()
    }

    pub fn calculate_tax(&self) -> f32 {
        self.items
            .iter()
            .map(|item| {
                item.price() * item.quantity() * item.tax_group.tax_rate()
            })
            .sum()
    }

    pub fn calculate_service_charge(&self) -> f32 {
        let subtotal = self.calculate_subtotal();
        match self.service_charge_percent {
            Some(percent) => subtotal * (percent / 100.0),
            None => 0.0,
        }
    }

    pub fn calculate_total(&self) -> f32 {
        let subtotal = self.calculate_subtotal();
        let tax = self.calculate_tax();
        let service_charge = self.calculate_service_charge();
        let gratuity = self.gratuity_amount.unwrap_or(0.0);

        subtotal + tax + service_charge + gratuity
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Show(show::Message),
    Edit(edit::Message),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Back,
    Save,
    StartEdit,
    Cancel,
}

pub fn update(
    sale: &mut Sale,
    message: Message,
) -> Action<Instruction, Message> {
    match message {
        Message::Show(msg) => match msg {
            show::Message::Back => Action::instruction(Instruction::Back),
            show::Message::StartEdit => {
                Action::instruction(Instruction::StartEdit)
                    .with_task(focus_next())
            }
        },
        Message::Edit(msg) => match msg {
            edit::Message::Cancel => Action::instruction(Instruction::Cancel),
            edit::Message::Save => Action::instruction(Instruction::Save),
            edit::Message::NameInput(name) => {
                sale.name = name;
                Action::none()
            }
            edit::Message::NameSubmit => {
                if sale.items.is_empty() {
                    sale.items.push(SaleItem::default());
                }
                Action::task(focus_next())
            }
            edit::Message::AddItem => {
                sale.items.push(SaleItem::default());
                Action::none()
            }
            edit::Message::RemoveItem(id) => {
                sale.items.retain(|item| item.id != id);
                Action::none()
            }
            edit::Message::UpdateItem(id, update) => {
                if let Some(item) = sale.items.iter_mut().find(|i| i.id == id) {
                    match update {
                        edit::Field::Name(name) => item.name = name,
                        edit::Field::Price(price) => {
                            item.price = if price.is_empty() {
                                None
                            } else {
                                price.parse().ok()
                            };
                        }
                        edit::Field::Quantity(qty) => {
                            item.quantity = if qty.is_empty() {
                                None
                            } else {
                                qty.parse().ok()
                            };
                        }
                        edit::Field::TaxGroup(group) => item.tax_group = group,
                    }
                }
                Action::none()
            }
            edit::Message::SubmitItem(id) => {
                // try to move to the next 'field' in this list. if all items
                // are filled out, add a new item and move to it instead
                if let Some(item) = sale.items.iter().find(|i| i.id == id) {
                    return if item.name.is_empty() {
                        Action::task(text_input::focus(edit::form_id(
                            "name", id,
                        )))
                    } else if item.quantity.is_none() {
                        Action::task(text_input::focus(edit::form_id(
                            "quantity", id,
                        )))
                    } else if item.price.is_none() {
                        Action::task(text_input::focus(edit::form_id(
                            "price", id,
                        )))
                    } else {
                        sale.items.push(SaleItem::default());
                        Action::task(text_input::focus(edit::form_id(
                            "name",
                            id + 1,
                        )))
                    };
                } else {
                    Action::none()
                }
            }
            edit::Message::UpdateServiceCharge(val) => {
                sale.service_charge_percent = Some(val);
                Action::none()
            }
            edit::Message::UpdateGratuity(val) => {
                sale.gratuity_amount = Some(val);
                Action::none()
            }
        },
    }
}

pub fn view(sale: &Sale, mode: Mode) -> Element<Message> {
    match mode {
        Mode::View => show::view(sale).map(Message::Show),
        Mode::Edit => edit::view(sale).map(Message::Edit),
    }
}

pub fn handle_hotkey(
    _: &Sale,
    mode: Mode,
    hotkey: Hotkey,
) -> Action<Instruction, Message> {
    match mode {
        Mode::View => show::handle_hotkey(hotkey).map(Message::Show),
        Mode::Edit => edit::handle_hotkey(hotkey).map(Message::Edit),
    }
}
