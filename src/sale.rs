//! View and edit sales
use iced::Element;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{tax::TaxGroup, Hotkey};

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
            name: String::from("New Sale"),
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
            .map(|item| item.price() * item.quantity() * item.tax_group.tax_rate())
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
pub enum Operation {
    Back,
    Save,
    StartEdit,
    Cancel,
}

pub type Action = crate::Action<Operation, Message>;

pub fn update(sale: &mut Sale, message: Message) -> Action {
    match message {
        Message::Show(msg) => match msg {
            show::Message::Back => Action::operation(Operation::Back),
            show::Message::StartEdit => Action::operation(Operation::StartEdit),
        },
        Message::Edit(msg) => match msg {
            edit::Message::Cancel => Action::operation(Operation::Cancel),
            edit::Message::Save => Action::operation(Operation::Save),
            edit::Message::NameChanged(name) => {
                sale.name = name;
                Action::none()
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
                        edit::ItemUpdate::Name(name) => item.name = name,
                        edit::ItemUpdate::Price(price) => {
                            item.price = if price.is_empty() {
                                None
                            } else {
                                price.parse().ok()
                            };
                        }
                        edit::ItemUpdate::Quantity(qty) => {
                            item.quantity = if qty.is_empty() {
                                None
                            } else {
                                qty.parse().ok()
                            };
                        }
                        edit::ItemUpdate::TaxGroup(group) => item.tax_group = group,
                    }
                }
                Action::none()
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

pub fn handle_hotkey(_: &Sale, mode: Mode, hotkey: Hotkey) -> Action {
    match hotkey {
        Hotkey::Escape => Action::operation(Operation::Cancel),
        _ => match mode {
            Mode::View => Action::none(),
            Mode::Edit => edit::handle_hotkey(hotkey).map(Message::Edit),
        },
    }
}
