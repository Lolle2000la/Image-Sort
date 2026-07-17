use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Alignment, Color, Element, Length};

use crate::message::{MediaMessage, Message};
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
    let prev_btn = container(
        button(
            text(char::from(lucide_icons::Icon::ChevronLeft))
                .font(iced::Font::with_name("lucide"))
                .size(16),
        )
        .on_press(Message::Media(MediaMessage::GoLeft)),
    )
    .id(iced::widget::Id::new("prev_btn"));

    // Right navigation button
    let next_btn = container(
        button(
            text(char::from(lucide_icons::Icon::ChevronRight))
                .font(iced::Font::with_name("lucide"))
                .size(16),
        )
        .on_press(Message::Media(MediaMessage::GoRight)),
    )
    .id(iced::widget::Id::new("next_btn"));

    let mut entries_row = row![].spacing(MEDIA_GRID_CARD_SPACING);

    for (i, entry) in filtered.iter().enumerate() {
        let is_selected = state.selected_index == Some(i);

        let thumbnail_content: Element<'_, Message> = if let Some(handle) =
            state.thumbnail_cache.peek(&entry.path)
        {
            let img = iced::widget::image(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill);

            let maybe_icon = match entry.media_type {
                media_sort_core::media_type::MediaType::Video => Some(lucide_icons::Icon::Film),
                media_sort_core::media_type::MediaType::Audio => Some(lucide_icons::Icon::Music),
                _ => None,
            };

            if let Some(icon) = maybe_icon {
                let icon_overlay = container(
                    text(char::from(icon))
                        .font(iced::Font::with_name("lucide"))
                        .size(16)
                        .color(Color::from_rgb(0.95, 0.95, 0.95)),
                )
                .padding([2, 4])
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.0, 0.0, 0.0, 0.8,
                    ))),
                    border: iced::Border {
                        radius: 2.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..iced::widget::container::Style::default()
                });

                let overlay = container(icon_overlay)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::End)
                    .align_y(Alignment::Start)
                    .padding(2);

                iced::widget::stack![img, overlay].into()
            } else {
                img.into()
            }
        } else {
            let fallback_icon = match entry.media_type {
                media_sort_core::media_type::MediaType::Audio => lucide_icons::Icon::Music,
                media_sort_core::media_type::MediaType::Video => lucide_icons::Icon::Film,
                media_sort_core::media_type::MediaType::Image => lucide_icons::Icon::Image,
            };

            text(char::from(fallback_icon))
                .font(iced::Font::with_name("lucide"))
                .size(24)
                .color(Color::from_rgb(0.5, 0.5, 0.6))
                .into()
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
            .shaping(iced::widget::text::Shaping::Advanced)
            .wrapping(iced::widget::text::Wrapping::Glyph);

        let file_name_container = container(file_name)
            .width(Length::Fill)
            .height(Length::Fixed(26.0))
            .align_x(Alignment::Center)
            .clip(true);

        let card = column![thumbnail, file_name_container]
            .align_x(Alignment::Center)
            .spacing(2)
            .width(Length::Fixed(MEDIA_GRID_CARD_WIDTH));

        let idx = i;
        let card_id_str = format!("media_card_{}", idx);
        let entry_button = container(
            button(card)
                .on_press(Message::Media(MediaMessage::SelectEntry(idx)))
                .padding(0)
                .style(iced::widget::button::text),
        )
        .id(iced::widget::Id::new(Box::leak(
            card_id_str.into_boxed_str(),
        )));

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
            Message::Media(MediaMessage::GridScrolled(
                offset,
                viewport.bounds().width,
                viewport.content_bounds().width,
            ))
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
