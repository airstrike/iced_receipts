//! List sales and navigate to sale details or editing
use iced::widget::{button, column, container, horizontal_space, row, text};
use iced::Alignment::Center;
use iced::{Element, Fill};
use std::collections::HashMap;

use crate::Sale;

#[derive(Debug, Clone)]
pub enum Message {
    NewSale,
    SelectSale(usize),
}

pub fn view(sales: &HashMap<usize, Sale>) -> Element<'_, Message> {
    let main_content: Element<_> = if sales.is_empty() {
        container(
            button(
                text("Create your first sale →")
                    .shaping(text::Shaping::Advanced),
            )
            .on_press(Message::NewSale),
        )
        .center(Fill)
        .into()
    } else {
        let mut sales_list = column![].spacing(10).width(Fill);

        for (id, sale) in sales {
            let total = sale.calculate_total();
            sales_list = sales_list.push(
                button(
                    row![column![
                        text(format!("{}", sale.name)).size(13),
                        text(format!("Total: ${:.2}", total)).size(12).style(
                            |theme: &iced::Theme| text::Style {
                                color: Some(
                                    theme.palette().text.scale_alpha(0.8)
                                ),
                            }
                        )
                    ]
                    .width(Fill)
                    .padding(10)]
                    .width(Fill),
                )
                .style(button::secondary)
                .on_press(Message::SelectSale(*id))
                .width(Fill),
            );
        }

        column![
            row![
                horizontal_space(),
                button(text("New Sale").size(14))
                    .style(button::success)
                    .on_press(Message::NewSale),
            ]
            .align_y(Center),
            sales_list,
        ]
        .spacing(20)
        .width(Fill)
        .into()
    };

    container(column![main_content].spacing(20).width(Fill).height(Fill))
        .padding(20)
        .into()
}
