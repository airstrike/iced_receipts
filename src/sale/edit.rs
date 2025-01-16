//! Edit new and existing sales
use iced::widget::{
    button, column, container, focus_next, focus_previous, horizontal_space, pick_list, row,
    scrollable, text, text_input,
};
use iced::Length::Fill;
use iced::{Alignment, Element, Length};

use crate::Hotkey;

use super::{Sale, TaxGroup};

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    NameChanged(String),
    AddItem,
    RemoveItem(usize),
    UpdateItem(usize, ItemUpdate),
    UpdateServiceCharge(f32),
    UpdateGratuity(f32),
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum ItemUpdate {
    Name(String),
    Price(String),
    Quantity(String),
    TaxGroup(TaxGroup),
}

pub fn view(sale: &Sale) -> Element<Message> {
    let header = row![
        button("←").on_press(Message::Back),
        text_input("Sale Name", &sale.name)
            .on_input(Message::NameChanged)
            .padding(5),
        horizontal_space(),
        row![
            button("Cancel")
                .on_press(Message::Cancel)
                .style(button::danger),
            button("Save")
                .on_press(Message::Save)
                .style(button::success),
        ]
        .spacing(10)
    ]
    .spacing(5)
    .align_y(Alignment::Center);

    let column_headers = row![
        text("Item Name").width(Fill),
        text("Qty").align_x(Alignment::Center).width(80.0),
        text("Price").align_x(Alignment::End).width(100.0),
        text("Tax Group").width(140.0),
        text("Total").align_x(Alignment::End).width(100.0),
        horizontal_space().width(25),
    ]
    .spacing(2)
    .padding([0, 10]);

    let items_list = sale.items.iter().fold(
        column![column_headers].spacing(5).width(Length::Fill),
        |col, item| {
            col.push(
                container(
                    row![
                        text_input("Item name", &item.name)
                            .on_input(|s| Message::UpdateItem(item.id.clone(), ItemUpdate::Name(s)))
                            .width(Fill)
                            .padding(5),
                        text_input("Quantity", &item.quantity_string())
                            .align_x(Alignment::Center)
                            .on_input(|s| Message::UpdateItem(
                                item.id.clone(),
                                ItemUpdate::Quantity(s)
                            ))
                            .width(80.0)
                            .padding(5),
                        text_input("Price", &item.price_string())
                            .align_x(Alignment::End)
                            .on_input(|s| Message::UpdateItem(
                                item.id.clone(),
                                ItemUpdate::Price(s)
                            ))
                            .width(100.0)
                            .padding(5),
                        pick_list(&TaxGroup::ALL[..], Some(item.tax_group), move |tax_group| {
                            Message::UpdateItem(item.id.clone(), ItemUpdate::TaxGroup(tax_group))
                        })
                        .width(140.0),
                        text(format!("${:.2}", item.price() * item.quantity()))
                            .align_x(Alignment::End)
                            .width(100.0),
                        button("×")
                            .width(25.0)
                            .on_press(Message::RemoveItem(item.id.clone()))
                            .style(button::danger)
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .style(container::rounded_box)
                .padding(0),
            )
        },
    );

    let totals = column![
        row![
            text("Subtotal").width(150.0),
            horizontal_space(),
            text(format!("${:.2}", sale.calculate_subtotal()))
        ],
        row![
            text("Service Charge").width(150.0),
            row![
                text_input(
                    "0.0",
                    &sale
                        .service_charge_percent
                        .map_or(String::new(), |p| format!("{:.1}", p)),
                )
                .width(60.0)
                .padding(5)
                .on_input(|s| Message::UpdateServiceCharge(if s.is_empty() {
                    0.0
                } else {
                    s.parse().ok().unwrap_or(0.0)
                })),
                text("%")
            ]
            .spacing(5),
            horizontal_space(),
            text(format!("${:.2}", sale.calculate_service_charge()))
        ],
        row![
            text("Tax").width(150.0),
            horizontal_space(),
            text(format!("${:.2}", sale.calculate_tax()))
        ],
        row![
            text("Gratuity").width(150.0),
            text_input(
                "0.00",
                &sale
                    .gratuity_amount
                    .map_or(String::new(), |g| format!("{:.2}", g)),
            )
            .width(100.0)
            .padding(5)
            .on_input(|s| Message::UpdateGratuity(if s.is_empty() {
                0.0
            } else {
                s.parse().ok().unwrap_or(0.0)
            })),
            horizontal_space(),
            text(format!("${:.2}", sale.gratuity_amount.unwrap_or(0.0)))
        ],
        row![
            text("Total").width(150.0).size(16),
            horizontal_space(),
            text(format!("${:.2}", sale.calculate_total())).size(16)
        ]
    ]
    .spacing(2)
    .width(Length::Fill);

    container(
        column![
            header,
            container(scrollable(
                column![
                    button("+ Add Item")
                        .on_press(Message::AddItem)
                        .style(button::primary),
                    items_list,
                ]
                .spacing(10)
                .padding(20)
            ))
            .height(Length::Fill)
            .style(container::rounded_box),
            container(totals).padding(20).style(container::rounded_box)
        ]
        .spacing(20)
        .height(Length::Fill),
    )
    .padding(20)
    .into()
}

pub fn handle_hotkey<T>(hotkey: Hotkey) -> crate::Action<T, Message> {
    match hotkey {
        Hotkey::Tab(modifier) => {
            if modifier.shift() {
                crate::Action::task(focus_previous())
            } else {
                crate::Action::task(focus_next())
            }
        }
        _ => crate::Action::none(),
    }
}
