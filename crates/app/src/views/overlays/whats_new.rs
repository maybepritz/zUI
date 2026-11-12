//! Обзор выпуска: главное изменение крупной карточкой с примером графика,
//! остальные — карточками в два столбца, кнопка «Продолжить» закреплена внизу.
use crate::components::{
    card::card,
    latency_chart::{LiveSeries, live_chart},
    page_title::page_title,
    palette, tone,
};
use crate::{i18n::tr, release, root::RootView};
use gpui_kit::assets::IconName as LucideIcon;
use gpui_kit::{
    component::{button::*, scroll::ScrollableElement as _, *},
    *,
};

struct NotesSection {
    title: String,
    items: Vec<String>,
}

/// Переводчики сохраняют привычный формат «заголовок и пункты» в одном сообщении Fluent.
fn sections(text: &str) -> Vec<NotesSection> {
    let mut sections: Vec<NotesSection> = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(title) = line.strip_prefix("## ") {
            sections.push(NotesSection {
                title: title.to_owned(),
                items: Vec::new(),
            });
        } else {
            if sections.is_empty() {
                sections.push(NotesSection {
                    title: tr("ui-what-s-new"),
                    items: Vec::new(),
                });
            }
            if let Some(section) = sections.last_mut() {
                section
                    .items
                    .push(line.strip_prefix("- ").unwrap_or(line).to_owned());
            }
        }
    }
    sections
}

/// Пункт заметок: точка-маркер и текст, который переносится под собой, а не под точкой.
fn item(text: String, cx: &App) -> Div {
    div()
        .h_flex()
        .items_start()
        .gap_3()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(
            div()
                .flex_shrink_0()
                .mt(px(8.))
                .size_1()
                .rounded_full()
                .bg(palette::muted()),
        )
        .child(div().flex_1().min_w_0().child(text))
}

fn change_card(section: NotesSection, cx: &App) -> Div {
    card(cx)
        .gap_3()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(section.title),
        )
        .children(section.items.into_iter().map(|text| item(text, cx)))
}

/// Пример графика для карточки главного изменения. Точки условные: это иллюстрация
/// того, как выглядит главная, а не измерение.
fn sample_chart(cx: &App) -> impl IntoElement {
    const SPAN: u64 = 15 * 60;
    let line = |id: &str, index: usize, values: [u64; 16]| LiveSeries {
        color: tone::service_color(id, index, cx),
        points: values
            .into_iter()
            .enumerate()
            .map(|(step, ms)| (step as u64 * 60, Some(ms)))
            .collect(),
    };
    live_chart(
        vec![
            line(
                "youtube",
                0,
                [
                    52, 48, 55, 47, 51, 49, 58, 50, 46, 53, 49, 57, 48, 52, 47, 50,
                ],
            ),
            line(
                "discord",
                1,
                [
                    118, 124, 109, 131, 115, 127, 112, 136, 121, 110, 129, 117, 133, 114, 125, 119,
                ],
            ),
        ],
        SPAN,
        SPAN,
        tone::warning(cx),
        palette::hairline(),
        palette::muted(),
        cx.theme().mode.is_dark(),
    )
}

/// Первая секция заметок — главное изменение выпуска: крупнее остальных и с примером.
fn featured_card(section: NotesSection, cx: &App) -> Div {
    card(cx)
        .gap_4()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_family(tone::MONO)
                        .text_color(palette::muted())
                        .child(tr("whats-new-featured")),
                )
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::MEDIUM)
                        .child(section.title),
                ),
        )
        .child(sample_chart(cx))
        .child(
            div()
                .v_flex()
                .gap_2()
                .children(section.items.into_iter().map(|text| item(text, cx))),
        )
}

pub fn render(cx: &mut Context<RootView>) -> AnyElement {
    let mut notes = sections(&tr("whats-new-notes")).into_iter();
    let featured = notes.next().map(|section| featured_card(section, cx));
    // Карточки одной высоты в строке; нечётная последняя занимает всю ширину.
    let cards = notes.fold(
        div().h_flex().flex_wrap().items_stretch().gap_4(),
        |row, section| row.child(change_card(section, cx).flex_1().min_w(px(280.))),
    );

    let hero = div()
        .v_flex()
        .w_full()
        .gap_2()
        .child(
            div()
                .text_xs()
                .font_family(tone::MONO)
                .text_color(palette::muted())
                .child(format!("{} · {}", tr("ui-what-s-new"), release::VERSION)),
        )
        .child(page_title(
            format!("zUI {}", release::codename()),
            Some(tr("whats-new-tagline").into()),
            Some(tr("whats-new-summary").into()),
            cx,
        ));

    let content = div()
        .v_flex()
        .w_full()
        .max_w(px(720.))
        .mx_auto()
        .gap_5()
        .child(hero)
        .children(featured)
        .child(cards);

    let footer = div()
        .w_full()
        .flex_shrink_0()
        .border_t_1()
        .border_color(palette::hairline())
        .px_6()
        .py_4()
        .child(
            div()
                .h_flex()
                .w_full()
                .max_w(px(720.))
                .mx_auto()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_xs()
                        .font_family(tone::MONO)
                        .text_color(palette::muted())
                        .child(tr("whats-new-footer")),
                )
                .child(
                    Button::new("dismiss-whats-new")
                        .primary()
                        .icon(LucideIcon::ArrowRight)
                        .label(tr("ui-continue"))
                        .on_click(cx.listener(|this, _, _, cx| this.on_dismiss_whats_new(cx))),
                ),
        );

    div()
        .v_flex()
        .size_full()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .overflow_y_scrollbar()
                .id("whats-new-scroll")
                .child(div().w_full().p_6().child(content)),
        )
        .child(footer)
        .into_any_element()
}
