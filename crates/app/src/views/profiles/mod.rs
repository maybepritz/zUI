//! Saved profiles on Home; all mutations route through root handlers.
pub(crate) mod form;
use crate::{
    components::section::{button_row, section},
    i18n::{tr, tr_args},
    root::RootView,
    state::AppState,
};
use form::ProfileDialog;
use gpui_kit::{
    component::{
        button::*,
        group_box::{GroupBox, GroupBoxVariants},
        label::Label,
        *,
    },
    prelude::FluentBuilder as _,
    *,
};
use z_core::profiles::ProfileDns;

pub(crate) fn render(state: &AppState, cx: &mut Context<RootView>) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let busy = state.profile_operations_busy();
    let rows: Vec<AnyElement> = state
        .settings
        .profiles
        .iter()
        .map(|profile| {
            let active = state.settings.selected_profile_id.as_deref() == Some(profile.id.as_str());
            let dns = match &profile.dns {
                ProfileDns::Keep => tr("profile-dns-keep"),
                ProfileDns::Automatic => tr("profile-dns-automatic"),
                ProfileDns::Preset(key) => z_core::dns::presets()
                    .iter()
                    .find(|p| &p.key == key)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| key.clone()),
            };
            let details = tr_args(
                "profile-details",
                &[
                    ("strategy", profile.strategy.clone().into()),
                    ("dns", dns.into()),
                ],
            );
            let services = tr_args(
                "profile-services",
                &[
                    (
                        "zapret",
                        tr(if profile.startup.zapret {
                            "profile-on"
                        } else {
                            "profile-off"
                        })
                        .into(),
                    ),
                    (
                        "telegram",
                        tr(if profile.startup.telegram {
                            "profile-on"
                        } else {
                            "profile-off"
                        })
                        .into(),
                    ),
                ],
            );
            let id = profile.id.clone();
            let actions = button_row().child(
                Button::new(format!("apply-profile-{id}"))
                    .primary()
                    .small()
                    .label(tr(if active {
                        "profile-active"
                    } else {
                        "profile-apply"
                    }))
                    .disabled(busy || active)
                    .on_click(cx.listener(move |this, _, _, cx| this.on_apply_profile(&id, cx))),
            );
            let actions = [
                (
                    "update",
                    "profile-update",
                    ProfileDialog::Update(profile.id.clone()),
                ),
                (
                    "rename",
                    "profile-rename",
                    ProfileDialog::Rename(profile.id.clone()),
                ),
                (
                    "delete",
                    "profile-delete",
                    ProfileDialog::Delete(profile.id.clone()),
                ),
            ]
            .into_iter()
            .fold(actions, |actions, (action, label, kind)| {
                actions.child(
                    Button::new(format!("{action}-profile-{}", profile.id))
                        .outline()
                        .small()
                        .label(tr(label))
                        .disabled(busy)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.on_profile_dialog(kind.clone(), window, cx)
                        })),
                )
            });
            GroupBox::new()
                .outline()
                .id(format!("profile-{}", profile.id))
                .child(
                    div()
                        .v_flex()
                        .gap_2()
                        .child(
                            Label::new(profile.name.clone())
                                .text_sm()
                                .font_weight(FontWeight::BOLD),
                        )
                        .child(Label::new(details).text_xs().text_color(muted))
                        .child(Label::new(services).text_xs().text_color(muted))
                        .child(actions),
                )
                .into_any_element()
        })
        .collect();
    section(tr("profiles-title"))
        .id("profiles")
        .child(Label::new(tr("profiles-hint")).text_xs().text_color(muted))
        .child(
            Button::new("profile-create")
                .outline()
                .label(tr("profile-create"))
                .disabled(busy || state.zapret.selected().is_none())
                .on_click(cx.listener(|this, _, window, cx| {
                    this.on_profile_dialog(ProfileDialog::Create, window, cx)
                })),
        )
        .when(rows.is_empty(), |section| {
            section.child(Label::new(tr("profiles-empty")).text_sm().text_color(muted))
        })
        .children(rows)
        .when(!state.profiles.status.is_empty(), |section| {
            section.child(Label::new(state.profiles.status.clone()).text_sm())
        })
        .into_any_element()
}
