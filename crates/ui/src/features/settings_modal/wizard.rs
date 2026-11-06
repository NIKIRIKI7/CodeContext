use core::preset::models::resolution::EditionProfile;
use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, px,
};

use crate::features::settings_modal::state::SetupWizardStep;
use crate::features::settings_modal::view::SettingsModalView;
use crate::shared::containers::form_controls::{
    render_compact_toggle_pill, render_labeled_setting_row, render_segmented_tab,
};
use crate::shared::primitives::badge::{Badge, BadgeVariant};
use crate::shared::primitives::button::{ButtonVariant, UiButton};
use crate::shared::primitives::icon::UiIcon;
use crate::theme::{FONT_FAMILY_MONO, FONT_FAMILY_UI, ThemeTokens};

impl SettingsModalView {
    pub(crate) fn render_setup_wizard_body(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = &theme.colors;
        let wiz = self.state.read(cx).wizard().clone();

        let step_bar = self.render_wizard_steps_bar(wiz.step, theme);
        let step_card = match wiz.step {
            SetupWizardStep::Step1SystemProbe => {
                self.render_wizard_step1_probe(theme, cx).into_any_element()
            }
            SetupWizardStep::Step2EditionSelect => {
                self.render_modules_and_panels_section(theme, cx).into_any_element()
            }
            SetupWizardStep::Step3TargetAndOs => {
                self.render_wizard_step3_os(&wiz, theme, cx).into_any_element()
            }
            SetupWizardStep::Step4Execute => {
                self.render_wizard_step4_execute(&wiz, theme, cx).into_any_element()
            }
        };

        let footer = self.render_wizard_footer(wiz.step, theme);

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(theme.spacing.md))
            .overflow_hidden()
            .child(step_bar)
            .child(
                div()
                    .id("setup-wizard-step-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_y_scroll()
                    .child(step_card),
            )
            .child(
                div()
                    .w_full()
                    .pt(px(theme.spacing.xs))
                    .border_t_1()
                    .border_color(colors.border)
                    .child(footer),
            )
    }

    fn render_wizard_steps_bar(
        &self,
        active_step: SetupWizardStep,
        theme: &'static ThemeTokens,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let steps: &[(&str, &str, SetupWizardStep)] = &[
            ("wiz-step-1", labels.wizard_step_1, SetupWizardStep::Step1SystemProbe),
            ("wiz-step-2", labels.wizard_step_2, SetupWizardStep::Step2EditionSelect),
            ("wiz-step-3", labels.wizard_step_3, SetupWizardStep::Step3TargetAndOs),
            ("wiz-step-4", labels.wizard_step_4, SetupWizardStep::Step4Execute),
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

        for &(id, title, step) in steps {
            let state_s = self.state.clone();
            bar = bar.child(render_segmented_tab(
                id,
                title,
                active_step == step,
                false,
                true,
                theme,
                move |_ev, _win, cx| {
                    state_s.update(cx, |s, cx| {
                        s.set_wizard_step(step);
                        cx.notify();
                    });
                },
            ));
        }
        bar
    }

    fn render_wizard_step1_probe(
        &self,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let probe = self.state.read(cx).system_probe_report();
        let simd_str = match probe.simd_level {
            core::system::ports::updater::SimdAccelerationLevel::Avx2Bmi2 => {
                "AVX2 + BMI2 (x86-64-v3)"
            }
            core::system::ports::updater::SimdAccelerationLevel::ArmNeon => "ARM64 NEON SIMD",
            core::system::ports::updater::SimdAccelerationLevel::BaselineScalar => {
                "Baseline Scalar"
            }
        };

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
                    .text_color(colors.text_primary)
                    .child(labels.wizard_probe_header),
            )
            .child(render_labeled_setting_row(
                labels.wizard_probe_os,
                Badge::new(SharedString::from(probe.distro_or_edition), BadgeVariant::Accent)
                    .with_theme(theme),
                theme,
            ))
            .child(render_labeled_setting_row(
                labels.wizard_probe_cpu,
                Badge::new(
                    SharedString::from(format!("{} cores · {simd_str}", probe.logical_cores)),
                    BadgeVariant::Success,
                )
                .with_theme(theme),
                theme,
            ))
            .child(render_labeled_setting_row(
                labels.wizard_probe_display,
                Badge::new(
                    SharedString::from(format!(
                        "{} · {}",
                        probe.display_server, probe.file_manager
                    )),
                    BadgeVariant::Neutral,
                )
                .with_theme(theme),
                theme,
            ))
            .child(render_labeled_setting_row(
                labels.wizard_probe_target,
                Badge::new(SharedString::from(probe.recommended_target), BadgeVariant::Neutral)
                    .with_theme(theme),
                theme,
            ))
    }

    fn render_wizard_step3_os(
        &self,
        wiz: &crate::features::settings_modal::state::SetupWizardState,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let probe = self.state.read(cx).system_probe_report();
        let state_sh = self.state.clone();
        let state_pa = self.state.clone();
        let reg_shell = wiz.register_shell;
        let add_path = wiz.add_to_path;

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
                    .child(labels.wizard_os_header),
            )
            .child(
                div()
                    .p(px(theme.spacing.md))
                    .rounded(px(theme.radius.sm))
                    .bg(colors.bg_app)
                    .border_1()
                    .border_color(colors.border)
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(colors.cyan)
                    .child(SharedString::from(probe.default_install_dir.display().to_string())),
            )
            .child(render_labeled_setting_row(
                labels.wizard_os_register_menu,
                render_compact_toggle_pill(
                    "wiz-toggle-shell",
                    reg_shell,
                    self.locale,
                    theme,
                    move |_ev, _win, cx| {
                        state_sh.update(cx, |s, cx| {
                            s.toggle_wizard_register_shell();
                            cx.notify();
                        });
                    },
                ),
                theme,
            ))
            .child(render_labeled_setting_row(
                labels.wizard_os_add_path,
                render_compact_toggle_pill(
                    "wiz-toggle-path",
                    add_path,
                    self.locale,
                    theme,
                    move |_ev, _win, cx| {
                        state_pa.update(cx, |s, cx| {
                            s.toggle_wizard_add_path();
                            cx.notify();
                        });
                    },
                ),
                theme,
            ))
    }

    fn render_wizard_step4_execute(
        &self,
        wiz: &crate::features::settings_modal::state::SetupWizardState,
        theme: &'static ThemeTokens,
        cx: &Context<Self>,
    ) -> gpui::Div {
        let colors = &theme.colors;
        let labels = self.locale.labels();
        let cfg = self.state.read(cx).global_config().clone();
        let state_inst = self.state.clone();
        let state_uninst = self.state.clone();

        let edition_name = match cfg.edition {
            EditionProfile::Full => labels.settings_edition_full,
            EditionProfile::Lite => labels.settings_edition_lite,
            EditionProfile::Custom => labels.settings_edition_custom,
        };

        let status_text = wiz
            .status_message
            .clone()
            .unwrap_or_else(|| format!("{} ({edition_name})", labels.wizard_ready_prefix));

        div()
            .w_full()
            .p(px(theme.spacing.xl))
            .rounded(px(theme.radius.md))
            .bg(colors.bg_elevated)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(theme.spacing.md))
            .child(
                div()
                    .font_family(FONT_FAMILY_MONO)
                    .text_xs()
                    .text_color(colors.cyan)
                    .child(SharedString::from(status_text)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(theme.spacing.sm))
                    .child(
                        UiButton::new("wiz-exec-install-btn", labels.wizard_install_now_btn)
                            .icon(UiIcon::Check)
                            .variant(ButtonVariant::SuccessCta)
                            .with_theme(theme)
                            .on_click(move |_ev, _win, cx| {
                                state_inst.update(cx, |s, cx| {
                                    let _ = s.execute_wizard_install();
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        UiButton::new("wiz-exec-uninstall-btn", labels.wizard_uninstall_btn)
                            .icon(UiIcon::Trash)
                            .variant(ButtonVariant::Danger)
                            .with_theme(theme)
                            .on_click(move |_ev, _win, cx| {
                                state_uninst.update(cx, |s, cx| {
                                    let _ = s.execute_wizard_uninstall();
                                    cx.notify();
                                });
                            }),
                    ),
            )
    }

    fn render_wizard_footer(
        &self,
        step: SetupWizardStep,
        theme: &'static ThemeTokens,
    ) -> gpui::Div {
        let labels = self.locale.labels();
        let state_back = self.state.clone();
        let state_next = self.state.clone();

        let next_step = match step {
            SetupWizardStep::Step1SystemProbe => SetupWizardStep::Step2EditionSelect,
            SetupWizardStep::Step2EditionSelect => SetupWizardStep::Step3TargetAndOs,
            SetupWizardStep::Step3TargetAndOs | SetupWizardStep::Step4Execute => {
                SetupWizardStep::Step4Execute
            }
        };

        div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                UiButton::new("wiz-close-back-to-settings", labels.wizard_back_btn)
                    .variant(ButtonVariant::Secondary)
                    .with_theme(theme)
                    .on_click(move |_ev, _win, cx| {
                        state_back.update(cx, |s, cx| {
                            s.close_setup_wizard();
                            cx.notify();
                        });
                    }),
            )
            .child(
                UiButton::new("wiz-next-step-btn", labels.wizard_next_btn)
                    .variant(ButtonVariant::Primary)
                    .disabled(step == SetupWizardStep::Step4Execute)
                    .with_theme(theme)
                    .on_click(move |_ev, _win, cx| {
                        state_next.update(cx, |s, cx| {
                            s.set_wizard_step(next_step);
                            cx.notify();
                        });
                    }),
            )
    }
}
