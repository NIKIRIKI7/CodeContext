use core::preset::models::resolution::{EditionProfile, UiChromeSetting};
use gpui::{Context, IntoElement, ParentElement as _, SharedString, Styled as _, div, px};

use crate::features::settings_modal::view::SettingsModalView;
use crate::shared::containers::form_controls::{
    render_compact_toggle_pill, render_labeled_setting_row, render_segmented_tab,
};
use crate::shared::containers::virtual_text::{LineHighlightMode, VirtualTextView};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_UI, ThemeTokens};

impl SettingsModalView {
    pub(crate) fn render_modules_and_panels_section(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let labels = self.locale.labels();
        let colors = &theme.colors;
        let cfg = self.state.read(cx).global_config().clone();
        let plugins = self.state.read(cx).list_plugins();

        let edition_bar = self.render_edition_profile_bar(cfg.edition, theme);
        let chrome_card = self.render_chrome_elements_card(&cfg, theme);
        let plugins_card = self.render_plugins_card(&plugins, theme);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(theme.spacing.md))
            .child(
                div()
                    .w_full()
                    .p(px(theme.spacing.lg))
                    .rounded(px(theme.radius.md))
                    .bg(colors.bg_elevated)
                    .border_1()
                    .border_color(colors.border)
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.xs))
                    .child(
                        div()
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_muted)
                            .child(labels.settings_edition_title),
                    )
                    .child(edition_bar),
            )
            .child(chrome_card)
            .child(plugins_card)
    }

    fn render_edition_profile_bar(
        &self,
        active: EditionProfile,
        theme: &'static ThemeTokens,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let profiles: &[(&str, &str, EditionProfile)] = &[
            ("ed-prof-full", labels.settings_edition_full, EditionProfile::Full),
            ("ed-prof-lite", labels.settings_edition_lite, EditionProfile::Lite),
            ("ed-prof-custom", labels.settings_edition_custom, EditionProfile::Custom),
        ];

        let mut bar = div()
            .w_full()
            .h(px(30.0))
            .p(px(2.0))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_app)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.0));

        for &(id, title, prof) in profiles {
            let state_p = self.state.clone();
            bar = bar.child(render_segmented_tab(
                id,
                title,
                active == prof,
                false,
                true,
                theme,
                move |_ev, _win, cx| {
                    state_p.update(cx, |s, cx| {
                        let _ = s.select_edition_profile(prof);
                        cx.notify();
                    });
                },
            ));
        }
        bar
    }

    fn render_chrome_elements_card(
        &self,
        cfg: &core::preset::models::resolution::GlobalConfig,
        theme: &'static ThemeTokens,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let chrome_rows: &[(&str, &str, UiChromeSetting)] = &[
            ("chrome-activity-rail", labels.chrome_activity_rail, UiChromeSetting::ActivityRail),
            ("chrome-top-bar", labels.chrome_top_bar, UiChromeSetting::TopBar),
            ("chrome-options-footer", labels.chrome_options_footer, UiChromeSetting::OptionsFooter),
            ("chrome-file-sizes", labels.chrome_file_sizes, UiChromeSetting::FileSizes),
            (
                "settings-toggle-bottom-bar",
                labels.settings_show_bottom_bar,
                UiChromeSetting::BottomBar,
            ),
            (
                "settings-toggle-term-presets",
                labels.chrome_term_presets,
                UiChromeSetting::TerminalPresets,
            ),
        ];

        let mut card = div()
            .w_full()
            .p(px(theme.spacing.lg))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xs))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.settings_cat_panels),
            );

        for &(id, row_label, setting) in chrome_rows {
            let state_c = self.state.clone();
            let setting_enabled = cfg.ui_chrome.is_enabled(setting);
            card = card.child(render_labeled_setting_row(
                row_label,
                render_compact_toggle_pill(
                    id,
                    setting_enabled,
                    self.locale,
                    theme,
                    move |_ev, _win, cx| {
                        state_c.update(cx, |s, cx| {
                            let _ = s.toggle_ui_chrome(setting);
                            cx.notify();
                        });
                    },
                ),
                theme,
            ));
        }
        card
    }

    fn render_plugins_card(
        &self,
        plugins: &[core::system::plugins::protocol::PluginDescriptor],
        theme: &'static ThemeTokens,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let mut card = div()
            .w_full()
            .p(px(theme.spacing.lg))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xs))
            .child(
                div()
                    .font_family(FONT_FAMILY_UI)
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(labels.settings_plugins_title),
            );

        for (idx, desc) in plugins.iter().enumerate() {
            let state_pl = self.state.clone();
            let plugin_id = desc.manifest.id.clone();
            let plugin_enabled = desc.enabled;
            card = card.child(
                div()
                    .w_full()
                    .min_h(px(28.0))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(theme.spacing.md))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .font_family(FONT_FAMILY_UI)
                            .text_xs()
                            .text_color(colors.text_primary)
                            .child(SharedString::from(desc.manifest.name.clone())),
                    )
                    .child(render_compact_toggle_pill(
                        if idx == 0 { "plugin-toggle-0" } else { "plugin-toggle-1" },
                        plugin_enabled,
                        self.locale,
                        theme,
                        move |_ev, _win, cx| {
                            let id_owned = plugin_id.clone();
                            state_pl.update(cx, |s, cx| {
                                s.toggle_plugin(&id_owned);
                                cx.notify();
                            });
                        },
                    )),
            );
        }
        card
    }

    pub(crate) fn render_live_json_editor(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let snap = self.state.read(cx);
        let buffer = snap.json_buffer().clone();
        let (editing_idx, editing_draft) = snap
            .editing_json_line()
            .map_or((None, None), |(idx, s)| (Some(idx), Some(s.to_owned())));
        let err_opt = snap.json_error().map(str::to_owned);

        let state_line = self.state.clone();
        let state_reload = self.state.clone();

        let status_badge = if let Some(ref err) = err_opt {
            Badge::new(SharedString::from(format!("JSON Error: {err}")), BadgeVariant::Danger)
                .with_theme(theme)
        } else {
            Badge::new(labels.settings_json_valid, BadgeVariant::Success).with_theme(theme)
        };

        let top_strip = div()
            .w_full()
            .h(px(30.0))
            .px(px(theme.spacing.md))
            .bg(colors.bg_elevated)
            .border_b_1()
            .border_color(colors.border)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(status_badge)
            .child(
                UiButton::new("settings-json-reload-btn", labels.settings_json_reload_btn)
                    .icon(UiIcon::RotateCcw)
                    .variant(ButtonVariant::Ghost)
                    .with_theme(theme)
                    .on_click(move |_ev, _win, cx| {
                        state_reload.update(cx, |s, cx| {
                            s.refresh_json_buffer();
                            cx.notify();
                        });
                    }),
            );

        let text_view = VirtualTextView::new("settings-live-json-virtual-view", buffer)
            .with_selected_line(editing_idx)
            .with_editing_line_draft(editing_draft)
            .with_highlight_mode(LineHighlightMode::PackedContext)
            .with_context(labels, theme)
            .on_line_click(move |line_idx, _win, cx| {
                state_line.update(cx, |s, cx| {
                    s.start_editing_json_line(line_idx);
                    cx.notify();
                });
            });

        div()
            .size_full()
            .rounded(px(theme.radius.md))
            .border_1()
            .border_color(colors.border)
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(top_strip)
            .child(div().flex_1().overflow_hidden().child(text_view))
    }
}
