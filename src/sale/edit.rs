//! Edit new and existing sales
use iced::widget::operation::{focus_next, focus_previous};
use iced::widget::{
    button, column, container, pick_list, row, scrollable, space, text,
    text_input,
};
use iced::{Alignment, Element, Fill};

use super::{Action, Instruction, Sale, TaxGroup};
use crate::Hotkey;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    NameSubmit,
    AddItem,
    RemoveItem(usize),
    UpdateItem(usize, Field),
    SubmitItem(usize),
    UpdateServiceCharge(f32),
    UpdateGratuity(f32),
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum Field {
    Name(String),
    Price(String),
    Quantity(String),
    TaxGroup(TaxGroup),
}

pub fn view(sale: &Sale) -> Element<'_, Message> {
    let header = row![
        space::horizontal().width(40),
        text_input("Sale Name", &sale.name)
            .on_input(Message::NameInput)
            .on_submit(Message::NameSubmit)
            .padding(5),
        space::horizontal(),
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
        space::horizontal().width(25),
    ]
    .spacing(2)
    .padding([0, 10]);

    let items_list = sale.items.iter().fold(
        column![column_headers].spacing(5).width(Fill),
        |col, item| {
            col.push(
                container(
                    row![
                        text_input("Item name", &item.name)
                            .id(form_id("name", item.id))
                            .on_input(|s| Message::UpdateItem(
                                item.id,
                                Field::Name(s)
                            ))
                            .on_submit(Message::SubmitItem(item.id))
                            .width(Fill)
                            .padding(5),
                        text_input("Quantity", &item.quantity_string())
                            .id(form_id("quantity", item.id))
                            .align_x(Alignment::Center)
                            .on_input(|s| Message::UpdateItem(
                                item.id.clone(),
                                Field::Quantity(s)
                            ))
                            .on_submit(Message::SubmitItem(item.id))
                            .width(80.0)
                            .padding(5),
                        text_input("Price", &item.price_string())
                            .id(form_id("price", item.id))
                            .align_x(Alignment::End)
                            .on_input(|s| Message::UpdateItem(
                                item.id,
                                Field::Price(s)
                            ))
                            .on_submit(Message::SubmitItem(item.id))
                            .width(100.0)
                            .padding(5),
                        pick_list(
                            &TaxGroup::ALL[..],
                            Some(item.tax_group),
                            move |tax_group| {
                                Message::UpdateItem(
                                    item.id,
                                    Field::TaxGroup(tax_group),
                                )
                            }
                        )
                        .width(140.0),
                        text(format!("${:.2}", item.price() * item.quantity()))
                            .align_x(Alignment::End)
                            .width(100.0),
                        button(text("×").center())
                            .width(25.0)
                            .on_press(Message::RemoveItem(item.id))
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
            space::horizontal(),
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
                }))
                .on_submit(Message::Save),
                text("%")
            ]
            .spacing(5),
            space::horizontal(),
            text(format!("${:.2}", sale.calculate_service_charge()))
        ],
        row![
            text("Tax").width(150.0),
            space::horizontal(),
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
            }))
            .on_submit(Message::Save),
            space::horizontal(),
            text(format!("${:.2}", sale.gratuity_amount.unwrap_or(0.0)))
        ],
        row![
            text("Total").width(150.0).size(16),
            space::horizontal(),
            text(format!("${:.2}", sale.calculate_total())).size(16)
        ]
    ]
    .spacing(2)
    .width(Fill);

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
            .height(Fill)
            .style(container::rounded_box),
            container(totals).padding(20).style(container::rounded_box)
        ]
        .spacing(20)
        .height(Fill),
    )
    .padding(20)
    .into()
}

pub fn handle_hotkey(hotkey: Hotkey) -> Action<Instruction, Message> {
    match hotkey {
        Hotkey::Tab(modifier) => {
            if modifier.shift() {
                Action::task(focus_previous())
            } else {
                Action::task(focus_next())
            }
        }
        _ => Action::none(),
    }
}

pub fn form_id(field: &str, id: usize) -> iced::widget::Id {
    iced::widget::Id::from(format!("{}-{}", field, id))
}
