//! A read-only view of a sale.
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::Length::Fill;
use iced::{Alignment, Element, Length};

use super::Sale;

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    StartEdit,
}

pub fn view(sale: &Sale) -> Element<Message> {
    let header = row![
        button("←").width(40).on_press(Message::Back),
        text(&sale.name).size(16),
        horizontal_space(),
        button("Edit").on_press(Message::StartEdit)
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let column_headers = row![
        text("Item Name").width(Fill),
        text("Qty").align_x(Alignment::Center).width(80.0),
        text("Price").align_x(Alignment::End).width(100.0),
        text("Tax Group").width(140.0),
        text("Total").align_x(Alignment::End).width(100.0),
    ]
    .spacing(2);

    let items_list = sale.items.iter().fold(
        column![column_headers].spacing(5).width(Length::Fill),
        |col, item| {
            col.push(
                container(
                    row![
                        text(&item.name).width(Fill),
                        text(item.quantity().to_string())
                            .align_x(Alignment::Center)
                            .width(80.0),
                        text(format!("${:.2}", item.price()))
                            .align_x(Alignment::End)
                            .width(100.0),
                        text(format!("{}", item.tax_group)).width(140.0),
                        text(format!("${:.2}", item.price() * item.quantity()))
                            .align_x(Alignment::End)
                            .width(100.0)
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
            text(format!(
                "{}%",
                sale.service_charge_percent.map_or(0.0, |p| p)
            )),
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
            text(format!("${:.2}", sale.gratuity_amount.unwrap_or(0.0))),
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
            container(scrollable(column![items_list,].spacing(10).padding(20)))
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
