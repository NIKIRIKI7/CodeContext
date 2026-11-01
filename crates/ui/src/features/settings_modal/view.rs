use core::system::ports::hotkey::HotkeyRegistrationStatus;
use core::system::ports::shell::ShellRegistrationStatus;
use core::system::ports::updater::{
    SelfInstallStatus, SemanticVersion, UpdateChannel, UpdateStage,
};
use gpui::{
    App, ClickEvent, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, Rgba, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};

use crate::features::settings_modal::state::SettingsModalState;
use crate::locale::{AppLocale, UiLabels};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::{UiIcon, render_vector_checkbox};
use crate::theme::{
    FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens, ThemeVariant, format_hotkey_for_current_os,
};

#[derive(Debug, Clone)]
pub struct SettingsModalView {
    state: Entity<SettingsModalState>,
    locale: AppLocale,
    theme_variant: ThemeVariant,
}

impl SettingsModalView {
    pub fn new(
        state: Entity<SettingsModalState>,
        locale: AppLocale,
        theme_variant: ThemeVariant,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_this, _model, cx| {
            cx.notify();
        })
        .detach();

        Self { state, locale, theme_variant }
    }

    pub fn sync_preferences(
        &mut self,
        locale: AppLocale,
        theme_variant: ThemeVariant,
        cx: &mut Context<Self>,
    ) {
        if self.locale != locale || self.theme_variant != theme_variant {
            self.locale = locale;
            self.theme_variant = theme_variant;
            cx.notify();
        }
    }

    fn render_appearance_section(&self, theme: &'static ThemeTokens) -> impl IntoElement + use<> {
        let labels = self.locale.labels();
        let colors = &theme.colors;
        let state_dark = self.state.clone();
        let state_light = self.state.clone();
        let state_ru = self.state.clone();
        let state_en = self.state.clone();

        let theme_switcher = render_segmented_pair(
            "settings-theme-dark",
            labels.btn_theme_dark,
            self.theme_variant == ThemeVariant::Dark,
            move |_ev, _win, cx| {
                state_dark.update(cx, |s, cx| {
                    s.set_theme_variant(ThemeVariant::Dark);
                    cx.notify();
                });
            },
            "settings-theme-light",
            labels.btn_theme_light,
            self.theme_variant == ThemeVariant::Light,
            move |_ev, _win, cx| {
                state_light.update(cx, |s, cx| {
                    s.set_theme_variant(ThemeVariant::Light);
                    cx.notify();
                });
            },
            theme,
        );

        let lang_switcher = render_segmented_pair(
            "settings-lang-ru",
            "Русский",
            self.locale == AppLocale::Ru,
            move |_ev, _win, cx| {
                state_ru.update(cx, |s, cx| {
                    s.set_locale(AppLocale::Ru);
                    cx.notify();
                });
            },
            "settings-lang-en",
            "English",
            self.locale == AppLocale::En,
            move |_ev, _win, cx| {
                state_en.update(cx, |s, cx| {
                    s.set_locale(AppLocale::En);
                    cx.notify();
                });
            },
            theme,
        );

        div()
            .w_full()
            .p(px(theme.spacing.xl))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.sm))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.settings_section_appearance),
            )
            .child(render_setting_row(labels.settings_theme_label, theme_switcher, theme))
            .child(render_setting_row(labels.settings_language_label, lang_switcher, theme))
    }

    fn render_system_section(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let snap = self.state.read(cx);
        let default_hotkey = snap.global_config().global_hotkey.clone();
        let shell_status = snap.shell_status().clone();
        let hotkey_status = snap.hotkey_status().clone();
        let is_recording = snap.is_recording_hotkey();

        let (shell_row_label, is_shell_active) = match shell_status {
            ShellRegistrationStatus::RegisteredHealthy => (labels.settings_context_menu_on, true),
            ShellRegistrationStatus::StaleExecutablePath { .. } => {
                (labels.settings_context_menu_stale, false)
            }
            ShellRegistrationStatus::NotRegistered => (labels.settings_context_menu_off, false),
        };

        let (hotkey_row_label, hotkey_badge, is_hk_active) = if is_recording {
            (labels.settings_hotkey_prefix, self.locale.hotkey_recording_label().to_owned(), true)
        } else {
            match hotkey_status {
                HotkeyRegistrationStatus::RegisteredActive { shortcut } => {
                    (labels.settings_hotkey_prefix, format_hotkey_for_current_os(&shortcut), true)
                }
                HotkeyRegistrationStatus::ConflictWithOtherApp { shortcut } => (
                    labels.settings_hotkey_conflict_prefix,
                    format_hotkey_for_current_os(&shortcut),
                    false,
                ),
                HotkeyRegistrationStatus::UnsupportedByDisplayServer { hint_command } => {
                    (labels.settings_hotkey_wayland_hint, hint_command.to_string(), false)
                }
                HotkeyRegistrationStatus::NotRegistered
                | HotkeyRegistrationStatus::InvalidShortcut { .. } => (
                    labels.settings_hotkey_prefix,
                    format_hotkey_for_current_os(&default_hotkey),
                    false,
                ),
            }
        };

        let state_hk = self.state.clone();
        let state_shell = self.state.clone();

        let hk_button = UiButton::new("settings-cycle-hotkey", SharedString::from(hotkey_badge))
            .icon(UiIcon::Keyboard)
            .variant(if is_recording { ButtonVariant::Primary } else { ButtonVariant::FilterPill })
            .fixed_width(196.0)
            .with_theme(theme)
            .active(is_hk_active)
            .on_click(move |_ev, _win, cx| {
                state_hk.update(cx, |s, cx| {
                    s.toggle_recording_hotkey();
                    cx.notify();
                });
            });

        let (channel_selector, update_button) = self.build_update_controls(theme, cx);

        let shell_button = render_compact_toggle_pill(
            "settings-toggle-shell-menu",
            is_shell_active,
            self.locale,
            theme,
            move |_ev, _win, cx| {
                state_shell.update(cx, |s, cx| {
                    let _ = s.toggle_shell_context_menu();
                    cx.notify();
                });
            },
        );

        div()
            .w_full()
            .p(px(theme.spacing.xl))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.sm))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.settings_section_system),
            )
            .child(render_setting_row(hotkey_row_label, hk_button, theme))
            .child(render_setting_row(shell_row_label, shell_button, theme))
            .child(render_setting_row(
                labels.settings_update_channel_label,
                channel_selector,
                theme,
            ))
            .child(render_setting_row(labels.settings_version_row_label, update_button, theme))
    }

    fn build_update_controls(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> (gpui::Div, UiButton) {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let snap = self.state.read(cx);
        let active_channel = snap.update_channel();
        let update_button_label = format_update_action_label(
            labels,
            snap.current_version(),
            snap.self_install_status(),
            snap.update_stage(),
        );
        let is_update_cta = matches!(snap.update_stage(), UpdateStage::UpdateAvailable { .. })
            || !snap.self_install_status().is_installed();

        let state_ch_stable = self.state.clone();
        let state_ch_beta = self.state.clone();
        let state_ch_alpha = self.state.clone();
        let channel_selector = div()
            .w(px(220.0))
            .h(px(theme.layout.control_height))
            .p(px(2.0))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_app)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.0))
            .child(render_segment_item(
                "settings-ch-stable",
                labels.settings_channel_stable,
                active_channel == UpdateChannel::Stable,
                move |_ev, _win, cx| {
                    spawn_channel_switch_and_check(
                        state_ch_stable.clone(),
                        UpdateChannel::Stable,
                        cx,
                    );
                },
                theme,
            ))
            .child(render_segment_item(
                "settings-ch-beta",
                labels.settings_channel_beta,
                active_channel == UpdateChannel::Beta,
                move |_ev, _win, cx| {
                    spawn_channel_switch_and_check(state_ch_beta.clone(), UpdateChannel::Beta, cx);
                },
                theme,
            ))
            .child(render_segment_item(
                "settings-ch-alpha",
                labels.settings_channel_alpha,
                active_channel == UpdateChannel::Alpha,
                move |_ev, _win, cx| {
                    spawn_channel_switch_and_check(
                        state_ch_alpha.clone(),
                        UpdateChannel::Alpha,
                        cx,
                    );
                },
                theme,
            ));

        let state_upd = self.state.clone();
        let update_button =
            UiButton::new("settings-trigger-update", SharedString::from(update_button_label))
                .variant(if is_update_cta {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::FilterPill
                })
                .fixed_width(220.0)
                .with_theme(theme)
                .on_click(move |_ev, _win, cx| {
                    spawn_update_or_self_install_action(state_upd.clone(), cx);
                });

        (channel_selector, update_button)
    }

    fn render_safety_section(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let cfg = self.state.read(cx).global_config();
        let auto_read_clipboard = cfg.auto_read_clipboard_on_apply;
        let create_backups = cfg.create_backups_by_default;
        let state_clip = self.state.clone();
        let state_bak = self.state.clone();

        let clip_btn = render_compact_toggle_pill(
            "settings-toggle-auto-clip",
            auto_read_clipboard,
            self.locale,
            theme,
            move |_ev, _win, cx| {
                state_clip.update(cx, |s, cx| {
                    let _ = s.toggle_auto_read_clipboard();
                    cx.notify();
                });
            },
        );

        let bak_btn = render_compact_toggle_pill(
            "settings-toggle-backups",
            create_backups,
            self.locale,
            theme,
            move |_ev, _win, cx| {
                state_bak.update(cx, |s, cx| {
                    let _ = s.toggle_create_backups_by_default();
                    cx.notify();
                });
            },
        );

        div()
            .w_full()
            .p(px(theme.spacing.xl))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.sm))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.settings_section_safety),
            )
            .child(render_setting_row(labels.settings_auto_clipboard, clip_btn, theme))
            .child(render_setting_row(labels.settings_create_backups, bak_btn, theme))
    }
}

#[allow(clippy::too_many_arguments)]
fn render_segmented_pair(
    left_id: &'static str,
    left_label: &'static str,
    left_active: bool,
    on_left: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    right_id: &'static str,
    right_label: &'static str,
    right_active: bool,
    on_right: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    theme: &'static ThemeTokens,
) -> impl IntoElement {
    let colors = &theme.colors;
    div()
        .w(px(220.0))
        .h(px(theme.layout.control_height))
        .p(px(2.0))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.0))
        .child(render_segment_item(left_id, left_label, left_active, on_left, theme))
        .child(render_segment_item(right_id, right_label, right_active, on_right, theme))
}

fn render_segment_item(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    theme: &'static ThemeTokens,
) -> impl IntoElement {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    div()
        .id(id)
        .flex_1()
        .h_full()
        .rounded(px(theme.radius.sm))
        .border_1()
        .border_color(if active { colors.border_active } else { colors.bg_app })
        .bg(if active { colors.bg_elevated } else { colors.bg_app })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_FAMILY_MONO)
        .text_xs()
        .text_color(if active { colors.text_primary } else { colors.text_secondary })
        .on_click(on_click)
        .child(label)
}

fn render_compact_toggle_pill(
    id: &'static str,
    enabled: bool,
    locale: AppLocale,
    theme: &'static ThemeTokens,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let colors = &theme.colors;
    let hover_bg = colors.bg_hover;
    let (check_background, check_border, check_foreground) = if enabled {
        (colors.badge_accent.bg, colors.cyan, colors.cyan)
    } else {
        (colors.bg_elevated, colors.border_active, colors.text_muted)
    };

    div()
        .id(id)
        .w(px(88.0))
        .h(px(theme.layout.control_height))
        .px(px(theme.spacing.md))
        .rounded(px(theme.radius.md))
        .bg(colors.bg_app)
        .border_1()
        .border_color(if enabled { colors.border_active } else { colors.border })
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .flex_shrink_0()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .on_click(on_click)
        .child(render_vector_checkbox(
            enabled,
            false,
            14.0,
            check_background,
            check_border,
            check_foreground,
            theme.radius.sm,
        ))
        .child(
            div()
                .font_family(FONT_FAMILY_MONO)
                .text_xs()
                .text_color(if enabled { colors.text_primary } else { colors.text_secondary })
                .child(locale.toggle_state_label(enabled)),
        )
}

fn spawn_update_or_self_install_action(state_upd: Entity<SettingsModalState>, cx: &mut App) {
    let (container, channel, repo_url, is_portable, has_available) = {
        let snap = state_upd.read(cx);
        (
            snap.container().clone(),
            snap.update_channel(),
            snap.global_config().update_repo_url.clone(),
            !snap.self_install_status().is_installed(),
            matches!(snap.update_stage(), UpdateStage::UpdateAvailable { .. }),
        )
    };
    cx.spawn(async move |cx| {
        cx.background_executor()
            .spawn(async move {
                use core::system::ports::shell::{ShellIntegrationPort as _, ShellMenuConfig};
                use core::system::ports::updater::UpdaterPort as _;
                if is_portable {
                    if let Ok(SelfInstallStatus::InstalledCanonical { install_path }) =
                        container.updater().perform_self_install()
                    {
                        let cfg = ShellMenuConfig::with_executable(install_path);
                        let _ = container.shell_integration().register_with_config(&cfg);
                    }
                } else if has_available {
                    let _ = container.updater().apply_staged_update();
                } else {
                    let _ = container.updater().check_for_updates(channel, &repo_url);
                }
            })
            .await;
        let _ = state_upd.update(cx, |s, cx| {
            s.sync_from_container();
            cx.notify();
        });
    })
    .detach();
}

fn spawn_channel_switch_and_check(
    state: Entity<SettingsModalState>,
    channel: UpdateChannel,
    cx: &mut App,
) {
    let (container, repo_url) = state.update(cx, |s, cx| {
        let _ = s.select_update_channel(channel);
        cx.notify();
        (s.container().clone(), s.global_config().update_repo_url.clone())
    });

    cx.spawn(async move |cx| {
        cx.background_executor()
            .spawn(async move {
                use core::system::ports::updater::UpdaterPort as _;
                let _ = container.updater().check_for_updates(channel, &repo_url);
            })
            .await;
        let _ = state.update(cx, |s, cx| {
            s.sync_from_container();
            cx.notify();
        });
    })
    .detach();
}

fn format_update_action_label(
    labels: &'static UiLabels,
    current_version: SemanticVersion,
    install_status: &SelfInstallStatus,
    stage: &UpdateStage,
) -> String {
    if matches!(install_status, SelfInstallStatus::PortableMode { .. }) {
        return format!("v{current_version} · {}", labels.updater_btn_self_install);
    }
    match stage {
        UpdateStage::Idle => format!("v{current_version} · {}", labels.updater_btn_check),
        UpdateStage::UpToDate { version } => {
            format!("v{version} · {}", labels.updater_status_up_to_date)
        }
        UpdateStage::UpdateAvailable { version, .. } => {
            format!(
                "{} v{version} · {}",
                labels.updater_btn_apply_prefix, labels.updater_btn_apply_action
            )
        }
        UpdateStage::ReadyToRestart { version } => {
            format!("v{version} · {}", labels.updater_status_ready_restart)
        }
        UpdateStage::Failed { .. } => {
            format!("v{current_version} · {}", labels.updater_status_failed)
        }
    }
}

fn render_setting_row(
    label: &'static str,
    control: impl IntoElement,
    theme: &'static ThemeTokens,
) -> impl IntoElement {
    div()
        .w_full()
        .min_h(px(32.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(theme.spacing.lg))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .font_family(FONT_FAMILY_UI)
                .text_xs()
                .text_color(theme.colors.text_primary)
                .child(label),
        )
        .child(control)
}

impl Render for SettingsModalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_variant.tokens();
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let state_close = self.state.clone();
        let appearance_sec = self.render_appearance_section(theme);
        let system_sec = self.render_system_section(theme, cx);
        let safety_sec = self.render_safety_section(theme, cx);
        let backdrop = Rgba { r: 0.02, g: 0.03, b: 0.05, a: 0.72 };

        div()
            .id("settings-modal-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(backdrop)
            .flex()
            .items_center()
            .justify_center()
            .p(px(theme.spacing.xxl))
            .child(
                div()
                    .w(px(580.0))
                    .p(px(theme.spacing.xxl))
                    .rounded(px(theme.radius.lg))
                    .bg(colors.bg_panel)
                    .border_1()
                    .border_color(colors.border_active)
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.lg))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .font_family(FONT_FAMILY_UI)
                                    .text_sm()
                                    .text_color(colors.text_primary)
                                    .child(labels.settings_modal_title),
                            )
                            .child(
                                UiButton::icon_only("settings-close-modal-btn", UiIcon::X)
                                    .variant(ButtonVariant::Ghost)
                                    .with_theme(theme)
                                    .on_click(move |_ev, _win, cx| {
                                        state_close.update(cx, |s, cx| {
                                            s.request_close();
                                            cx.notify();
                                        });
                                    }),
                            ),
                    )
                    .child(appearance_sec)
                    .child(system_sec)
                    .child(safety_sec),
            )
    }
}
