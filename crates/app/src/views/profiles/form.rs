//! State and content of profile dialogs; persistence stays in AppState.
use crate::i18n::{self, Text, tr};
use gpui_kit::{
    component::{
        button::*,
        input::{Input, InputEvent, InputState},
        label::Label,
        *,
    },
    prelude::FluentBuilder as _,
    *,
};
use z_core::profiles::ProfileDns;

#[derive(Clone)]
pub(crate) enum ProfileDialog {
    Create,
    Rename(String),
    Update(String),
    Delete(String),
}
impl ProfileDialog {
    pub(crate) fn id(&self) -> Option<&str> {
        match self {
            Self::Create => None,
            Self::Rename(id) | Self::Update(id) | Self::Delete(id) => Some(id),
        }
    }
    pub(crate) fn title(&self) -> &'static str {
        match self {
            Self::Create => "profile-create",
            Self::Rename(_) => "profile-rename",
            Self::Update(_) => "profile-update",
            Self::Delete(_) => "profile-delete",
        }
    }
    pub(crate) fn confirm(&self) -> &'static str {
        match self {
            Self::Create => "profile-save",
            Self::Rename(_) => "profile-rename",
            Self::Update(_) => "profile-update",
            Self::Delete(_) => "profile-delete",
        }
    }
    pub(crate) fn has_name(&self) -> bool {
        matches!(self, Self::Create | Self::Rename(_))
    }
    fn has_dns(&self) -> bool {
        matches!(self, Self::Create | Self::Update(_))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DnsChoice {
    Current,
    Keep,
    Automatic,
}

pub(crate) struct ProfileForm {
    pub input: Entity<InputState>,
    pub error: Option<Text>,
    kind: ProfileDialog,
    name: String,
    dns: DnsChoice,
    _change: Subscription,
}
impl ProfileForm {
    pub(crate) fn new(
        kind: ProfileDialog,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input =
            cx.new(|cx| InputState::new(window, cx).placeholder(tr("profile-name-placeholder")));
        input.update(cx, |input, cx| input.set_value(name.clone(), window, cx));
        let change = cx.subscribe(&input, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.error = None;
                cx.notify();
            }
        });
        Self {
            input,
            error: None,
            kind,
            name,
            dns: DnsChoice::Current,
            _change: change,
        }
    }
    pub(crate) fn dns_override(&self) -> Option<ProfileDns> {
        match self.dns {
            DnsChoice::Current => None,
            DnsChoice::Keep => Some(ProfileDns::Keep),
            DnsChoice::Automatic => Some(ProfileDns::Automatic),
        }
    }
}
impl Render for ProfileForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hint = match self.kind {
            ProfileDialog::Create => i18n::text("profile-create-hint"),
            ProfileDialog::Rename(_) => i18n::text("profile-rename-hint"),
            ProfileDialog::Update(_) => {
                i18n::text_args("profile-update-hint", &[("name", self.name.clone().into())])
            }
            ProfileDialog::Delete(_) => {
                i18n::text_args("profile-delete-hint", &[("name", self.name.clone().into())])
            }
        };
        let choices = [
            (DnsChoice::Current, "profile-dns-current"),
            (DnsChoice::Keep, "profile-dns-keep"),
            (DnsChoice::Automatic, "profile-dns-automatic"),
        ];
        div()
            .v_flex()
            .w_full()
            .gap_4()
            .child(
                Label::new(hint)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            )
            .when(self.kind.has_name(), |body| {
                body.child(
                    div()
                        .v_flex()
                        .gap_2()
                        .child(Label::new(tr("profile-name")).text_sm())
                        .child(
                            Input::new(&self.input)
                                .aria_label(tr("profile-name"))
                                .w_full(),
                        ),
                )
            })
            .when(self.kind.has_dns(), |body| {
                body.child(
                    div()
                        .v_flex()
                        .gap_2()
                        .child(Label::new(tr("profile-dns-label")).text_sm())
                        .child(div().h_flex().flex_wrap().gap_2().children(
                            choices.into_iter().map(|(choice, key)| {
                                let button =
                                    Button::new(key).label(tr(key)).toggled(self.dns == choice);
                                let button = if self.dns == choice {
                                    button.primary()
                                } else {
                                    button.outline()
                                };
                                button.on_click(cx.listener(move |this, _, _, cx| {
                                    this.dns = choice;
                                    this.error = None;
                                    cx.notify();
                                }))
                            }),
                        )),
                )
            })
            .when_some(self.error.clone(), |body, error| {
                body.child(Label::new(error).text_sm().text_color(cx.theme().danger))
            })
    }
}
