use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Length};

use crate::message::Message;
use crate::state::AppState;

/// Identifier used to programmatically scroll the media grid's horizontal
/// scrollable (e.g. to bring the currently selected entry into view).
pub static MEDIA_GRID_SCROLLABLE_ID: iced::widget::Id =
    iced::widget::Id::new("media_grid_scrollable");

/// Width of a single thumbnail card in the media grid.
pub const MEDIA_GRID_CARD_WIDTH: f32 = 60.0;
/// Horizontal spacing between adjacent thumbnail cards in the media grid row.
pub const MEDIA_GRID_CARD_SPACING: f32 = 8.0;
/// Extra vertical space appended at the bottom of the media grid so that the
/// horizontal scrollbar does not overlap with the file names below each
/// thumbnail.
const MEDIA_GRID_SCROLLBAR_CLEARANCE: f32 = 12.0;

pub fn media_grid_view(state: &AppState) -> Element<'_, Message> {
    let filtered = state.filtered_media_entries();

    if filtered.is_empty() {
        return container(
            text(if state.search_query.is_empty() {
                state.l10n.tr("ui-no-images")
            } else {
                state.l10n.tr("ui-no-results")
            })
            .size(14),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    // Left navigation button
    let prev_btn = button(
        text(char::from(lucide_icons::Icon::ChevronLeft))
            .font(iced::Font::with_name("lucide"))
            .size(16),
    )
    .on_press(Message::GoLeft);

    // Right navigation button
    let next_btn = button(
        text(char::from(lucide_icons::Icon::ChevronRight))
            .font(iced::Font::with_name("lucide"))
            .size(16),
    )
    .on_press(Message::GoRight);

    let mut entries_row = row![].spacing(MEDIA_GRID_CARD_SPACING);

    for (i, entry) in filtered.iter().enumerate() {
        let is_selected = state.selected_index == Some(i);

        let thumbnail_content: Element<'_, Message> =
            if let Some(handle) = state.thumbnail_cache.peek(&entry.path) {
                iced::widget::image(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                text("[IMG]").size(12).into()
            };

        let thumbnail = container(thumbnail_content)
            .center_x(MEDIA_GRID_CARD_WIDTH)
            .center_y(50)
            .width(Length::Fixed(MEDIA_GRID_CARD_WIDTH))
            .height(Length::Fixed(50.0))
            .style(move |theme: &iced::Theme| {
                let palette = theme.palette();
                let bg = if is_selected {
                    palette.primary
                } else {
                    Color::from_rgb(0.08, 0.08, 0.1)
                };
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: if is_selected {
                            palette.text
                        } else {
                            Color::from_rgb(0.2, 0.2, 0.25)
                        },
                    },
                    ..iced::widget::container::Style::default()
                }
            });

        let file_name = text(&entry.file_name)
            .size(10)
            .shaping(iced::widget::text::Shaping::Advanced);
        let card = column![thumbnail, file_name]
            .align_x(Alignment::Center)
            .spacing(2)
            .width(Length::Fixed(MEDIA_GRID_CARD_WIDTH));

        let idx = i;
        let entry_button = button(card)
            .on_press(Message::SelectEntry(idx))
            .style(iced::widget::button::text);

        entries_row = entries_row.push(entry_button);
    }

    // Wrap the row of cards in a column with empty space at the bottom. This
    // pushes the file names (which sit below each thumbnail) clear of the
    // horizontal scrollbar, so the bar can no longer cover them.
    let scrollable_content = column![
        entries_row,
        space().height(Length::Fixed(MEDIA_GRID_SCROLLBAR_CLEARANCE))
    ];

    let scrollable_row = scrollable(scrollable_content)
        .id(MEDIA_GRID_SCROLLABLE_ID.clone())
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::default(),
        ))
        .on_scroll(|viewport| {
            let offset = viewport.absolute_offset();
            Message::GridScrolled(
                offset,
                viewport.bounds().width,
                viewport.content_bounds().width,
            )
        });

    container(
        row![
            prev_btn,
            container(scrollable_row).width(Length::Fill),
            next_btn
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}
