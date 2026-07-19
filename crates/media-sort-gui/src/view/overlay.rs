use iced::widget::{Space, container, mouse_area, stack};
use iced::{Color, Element, Length};

use crate::message::Message;

/// Renders a centered modal dialog content element wrapped in click interception layers.
///
/// Clicking the surrounding dim background triggers the `on_background_press` message.
/// Clicking inside the card content itself does not propagate to the background.
pub fn modal_overlay<'a>(
    content: Element<'a, Message>,
    on_background_press: Message,
) -> Element<'a, Message> {
    let dim_background = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(on_background_press);

    stack![
        container(dim_background)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.0, 0.0, 0.0, 0.6,
                ))),
                ..Default::default()
            }),
        container(mouse_area(content).on_press(Message::NoOp))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
    ]
    .into()
}
