use std::sync::OnceLock;

use serde::Deserialize;
use unic_langid::{LanguageIdentifier, langid};

pub const EMBEDDED_LOCALES_JSON: &str = include_str!("../assets/locales.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppLocale {
    #[default]
    Ru,
    En,
}

impl AppLocale {
    #[inline]
    pub const fn language_id(self) -> LanguageIdentifier {
        match self {
            Self::Ru => langid!("ru-RU"),
            Self::En => langid!("en-US"),
        }
    }

    #[must_use]
    #[inline]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Ru => Self::En,
            Self::En => Self::Ru,
        }
    }

    #[inline]
    pub const fn short_code(self) -> &'static str {
        match self {
            Self::Ru => "RU",
            Self::En => "EN",
        }
    }

    #[inline]
    pub fn labels(self) -> &'static UiLabels {
        let (ru_labels, en_labels) = loaded_locales();
        match self {
            Self::Ru => ru_labels,
            Self::En => en_labels,
        }
    }

    #[inline]
    pub const fn toggle_state_label(self, enabled: bool) -> &'static str {
        match (self, enabled) {
            (Self::Ru, true) => "Вкл",
            (Self::Ru, false) => "Выкл",
            (Self::En, true) => "On",
            (Self::En, false) => "Off",
        }
    }

    #[inline]
    pub const fn hotkey_recording_label(self) -> &'static str {
        match self {
            Self::Ru => "Нажмите клавиши… (Esc)",
            Self::En => "Press keys… (Esc)",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct UiLabels {
    pub app_title: &'static str,
    pub tab_context_packer: &'static str,
    pub tab_change_applier: &'static str,
    pub nav_pack_short: &'static str,
    pub nav_apply_short: &'static str,
    pub btn_open_folder_prompt: &'static str,
    pub btn_settings: &'static str,
    pub btn_close_modal: &'static str,
    pub btn_project_meta: &'static str,
    pub btn_theme_dark: &'static str,
    pub btn_theme_light: &'static str,
    pub btn_copy_context: &'static str,
    pub btn_copied: &'static str,
    pub btn_save_global: &'static str,
    pub btn_save_project_cfg: &'static str,
    pub btn_rescan: &'static str,
    pub btn_select_all: &'static str,
    pub btn_deselect_all: &'static str,
    pub btn_expand_all: &'static str,
    pub btn_collapse_all: &'static str,
    pub btn_dismiss_banner: &'static str,
    pub btn_paste_clipboard: &'static str,
    pub btn_autofix: &'static str,
    pub btn_apply_changes: &'static str,
    pub btn_undo: &'static str,
    pub tree_header: &'static str,
    pub tree_filter_placeholder: &'static str,
    pub rules_header: &'static str,
    pub rules_active_suffix: &'static str,
    pub rules_section_fs: &'static str,
    pub rules_section_transform: &'static str,
    pub rules_section_volumes: &'static str,
    pub preview_header: &'static str,
    pub empty_project_hint: &'static str,
    pub empty_preview_hint: &'static str,
    pub no_project_selected: &'static str,
    pub origin_default: &'static str,
    pub origin_global: &'static str,
    pub origin_local_project: &'static str,
    pub rule_gitignore: &'static str,
    pub rule_minify: &'static str,
    pub rule_crlf_to_lf: &'static str,
    pub rule_ascii_tree: &'static str,
    pub rule_symlinks: &'static str,
    pub chunk_limit_label: &'static str,
    pub chunk_single_volume: &'static str,
    pub chunk_selector_prefix: &'static str,
    pub chunk_volume_prefix: &'static str,
    pub include_globs_default: &'static str,
    pub exclude_globs_default: &'static str,
    pub secret_unlocked_session: &'static str,
    pub secret_locked: &'static str,
    pub unit_files: &'static str,
    pub unit_selected: &'static str,
    pub unit_lines: &'static str,
    pub unit_tokens: &'static str,
    pub unit_chars: &'static str,
    pub unit_blocks: &'static str,
    pub unit_secrets_hidden: &'static str,
    pub unit_binaries_skipped: &'static str,
    pub applier_input_header: &'static str,
    pub applier_preamble_stripped: &'static str,
    pub applier_truncated_closed: &'static str,
    pub applier_parsed_files_prefix: &'static str,
    pub applier_no_parsed_files: &'static str,
    pub applier_empty_diff_hint: &'static str,
    pub applier_sandbox_strip_ok: &'static str,
    pub applier_sandbox_blocked: &'static str,
    pub applier_truncated_warning: &'static str,
    pub applier_conflict_warning: &'static str,
    pub applier_valid_block: &'static str,
    pub applier_parser_engine: &'static str,
    pub applier_autobackup_label: &'static str,
    pub applier_backup_footer_title: &'static str,
    pub applier_backup_on: &'static str,
    pub applier_backup_off: &'static str,
    pub applier_backup_compact_on: &'static str,
    pub applier_backup_compact_off: &'static str,
    pub applier_summary_prefix: &'static str,
    pub applier_encoding_lf: &'static str,
    pub applier_encoding_crlf: &'static str,
    pub applier_status_applied_prefix: &'static str,
    pub applier_status_undone_prefix: &'static str,
    pub applier_status_error_prefix: &'static str,
    pub status_sandbox_label: &'static str,
    pub status_sandbox_on: &'static str,
    pub status_backup_label: &'static str,
    pub status_backup_path: &'static str,
    pub status_ready: &'static str,
    pub settings_modal_title: &'static str,
    pub settings_section_appearance: &'static str,
    pub settings_section_system: &'static str,
    pub settings_section_safety: &'static str,
    pub settings_theme_label: &'static str,
    pub settings_language_label: &'static str,
    pub settings_hotkey_prefix: &'static str,
    pub settings_hotkey_conflict_prefix: &'static str,
    pub settings_hotkey_wayland_hint: &'static str,
    pub settings_auto_clipboard: &'static str,
    pub settings_create_backups: &'static str,
    pub settings_context_menu_on: &'static str,
    pub settings_context_menu_off: &'static str,
    pub settings_context_menu_stale: &'static str,
    pub settings_update_channel_label: &'static str,
    pub settings_version_row_label: &'static str,
    pub settings_channel_stable: &'static str,
    pub settings_channel_beta: &'static str,
    pub settings_channel_alpha: &'static str,
    pub updater_btn_self_install: &'static str,
    pub updater_btn_check: &'static str,
    pub updater_status_up_to_date: &'static str,
    pub updater_btn_apply_prefix: &'static str,
    pub updater_btn_apply_action: &'static str,
    pub updater_status_ready_restart: &'static str,
    pub updater_status_failed: &'static str,
    pub update_modal_title: &'static str,
    pub update_modal_subtitle: &'static str,
    pub update_modal_current_label: &'static str,
    pub update_modal_new_label: &'static str,
    pub update_modal_channel_label: &'static str,
    pub btn_update_now: &'static str,
    pub btn_update_later: &'static str,
    pub meta_modal_title: &'static str,
    pub meta_section_metrics: &'static str,
    pub meta_section_limits: &'static str,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
struct LocalesBundle {
    ru: UiLabels,
    en: UiLabels,
}

static LOADED_LOCALES: OnceLock<(UiLabels, UiLabels)> = OnceLock::new();

fn loaded_locales() -> &'static (UiLabels, UiLabels) {
    LOADED_LOCALES.get_or_init(|| {
        if let Ok(external_json) = std::fs::read_to_string("crates/ui/assets/locales.json")
            .or_else(|_| std::fs::read_to_string("assets/locales.json"))
        {
            let leaked_json: &'static str = Box::leak(external_json.into_boxed_str());
            if let Ok(bundle) = serde_json::from_str::<LocalesBundle>(leaked_json) {
                return (bundle.ru, bundle.en);
            }
        }
        serde_json::from_str::<LocalesBundle>(EMBEDDED_LOCALES_JSON)
            .map_or_else(|_| (UiLabels::default(), UiLabels::default()), |b| (b.ru, b.en))
    })
}

#[cfg(test)]
mod tests {
    use super::AppLocale;

    #[test]
    fn loads_ru_and_en_labels_directly_from_json() {
        assert_eq!(AppLocale::Ru.labels().app_title, "CodeContext");
        assert_eq!(AppLocale::En.labels().app_title, "CodeContext");
        assert_eq!(AppLocale::Ru.labels().btn_copy_context, "Копировать");
        assert_eq!(AppLocale::En.labels().btn_copy_context, "Copy");
    }
}
