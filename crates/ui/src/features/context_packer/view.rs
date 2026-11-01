use core::packing::aggregates::project::Project;
use gpui::{
    ClipboardItem, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    UniformListScrollHandle, Window, div,
};

use crate::features::context_packer::components::file_tree::FileTreeComponent;
use crate::features::context_packer::components::live_preview::{
    LivePreviewComponent, LivePreviewProps,
};
use crate::features::context_packer::components::rules_editor::RulesEditorComponent;
use crate::features::context_packer::state::ContextPackerState;
use crate::locale::AppLocale;
use crate::shared::containers::split_pane::SplitPane;
use crate::shared::primitives::warning_banner::WarningBanner;
use crate::theme::ThemeVariant;

#[derive(Debug, Clone)]
pub struct ContextPackerView {
    state: Entity<ContextPackerState>,
    locale: AppLocale,
    theme_variant: ThemeVariant,
    tree_scroll_handle: UniformListScrollHandle,
    preview_scroll_handle: UniformListScrollHandle,
}

impl ContextPackerView {
    pub fn new(
        state: Entity<ContextPackerState>,
        locale: AppLocale,
        theme_variant: ThemeVariant,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_this, _model, cx| {
            cx.notify();
        })
        .detach();
        Self {
            state,
            locale,
            theme_variant,
            tree_scroll_handle: UniformListScrollHandle::new(),
            preview_scroll_handle: UniformListScrollHandle::new(),
        }
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

    #[inline]
    pub const fn state(&self) -> &Entity<ContextPackerState> {
        &self.state
    }
}

impl Render for ContextPackerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme_variant.tokens();
        let labels = self.locale.labels();
        let snapshot = self.state.read(cx);
        let banner_items = snapshot.active_banner_items_localized(self.locale);
        let tree_projection = snapshot.tree_projection().clone();
        let search_query = snapshot.search_query().to_owned();
        let ui_status = snapshot.ui_status();
        let all_collapsed = snapshot.are_all_dirs_collapsed();
        let active_preset = snapshot.effective_preset().clone();

        let (total_bytes, packed_files, blocked_secrets, skipped_binaries, total_chunks) =
            snapshot.pack_result().map_or_else(
                || {
                    let est = snapshot.project().map_or(0, Project::estimated_pack_capacity_bytes);
                    let sel = snapshot.project().map_or(0, Project::selected_files_count);
                    (est, sel, 0, 0, 1)
                },
                |res| {
                    (
                        res.total_bytes as u64,
                        res.packed_files_count,
                        res.blocked_secrets_count,
                        res.skipped_binary_count,
                        res.chunks.len().max(1),
                    )
                },
            );

        let preview_props = LivePreviewProps {
            buffer: snapshot.preview_buffer().clone(),
            total_bytes,
            packed_files_count: packed_files,
            blocked_secrets_count: blocked_secrets,
            skipped_binary_count: skipped_binaries,
            active_chunk_index: snapshot.active_chunk_index(),
            total_chunks,
            copied_recently: ui_status.copied_recently,
            labels,
            theme,
        };

        let banner = self.build_warning_banner(banner_items);
        let file_tree = self.build_file_tree(
            tree_projection,
            search_query,
            ui_status.search_focused,
            all_collapsed,
        );
        let rules_editor = self.build_rules_editor(active_preset, ui_status.filters_popover_open);
        let live_preview = self.build_live_preview(preview_props);

        let left_column = div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(div().flex_1().overflow_hidden().child(file_tree))
            .child(rules_editor);

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.colors.bg_app)
            .overflow_hidden()
            .child(
                div().flex_1().overflow_hidden().child(
                    SplitPane::new(theme.layout.packer_split_ratio, left_column, live_preview)
                        .with_theme(theme),
                ),
            )
            .child(banner)
    }
}

impl ContextPackerView {
    fn build_warning_banner(
        &self,
        items: Vec<crate::shared::primitives::warning_banner::WarningBannerItem>,
    ) -> WarningBanner {
        let state = self.state.clone();
        WarningBanner::new(items)
            .with_context(self.locale.labels(), self.theme_variant.tokens())
            .on_dismiss(move |_ev, _win, cx| {
                state.update(cx, |s, cx| {
                    s.dismiss_warnings();
                    cx.notify();
                });
            })
    }

    fn build_file_tree(
        &self,
        projection: crate::features::context_packer::components::file_tree::FlatTreeProjection,
        search_query: String,
        search_focused: bool,
        all_collapsed: bool,
    ) -> FileTreeComponent {
        let labels = self.locale.labels();
        let theme = self.theme_variant.tokens();
        let state_check = self.state.clone();
        let state_expand = self.state.clone();
        let state_secret = self.state.clone();
        let state_master = self.state.clone();
        let state_fold = self.state.clone();
        let state_focus_search = self.state.clone();
        let state_clear_search = self.state.clone();
        let state_open_proj = self.state.clone();
        let prompt_label = labels.btn_open_folder_prompt;

        FileTreeComponent::new(projection, labels)
            .with_theme(theme)
            .with_search_state(search_query, search_focused, all_collapsed)
            .track_scroll(self.tree_scroll_handle.clone())
            .on_open_project(move |_ev, _win, cx| {
                let rx = cx.prompt_for_paths(gpui::PathPromptOptions {
                    files: false,
                    directories: true,
                    multiple: false,
                    prompt: Some(gpui::SharedString::from(prompt_label)),
                });
                let state_for_spawn = state_open_proj.clone();
                cx.spawn(async move |cx| {
                    if let Ok(Ok(Some(paths))) = rx.await
                        && let Some(chosen_dir) = paths.into_iter().next()
                    {
                        let _ = state_for_spawn.update(cx, |s, cx| {
                            let _ = s.open_project(chosen_dir);
                            cx.notify();
                        });
                    }
                })
                .detach();
            })
            .on_toggle_check(move |path, is_dir, _win, cx| {
                let target_path = path.to_owned();
                state_check.update(cx, |s, cx| {
                    s.toggle_tree_node_checkbox(&target_path, is_dir);
                    cx.notify();
                });
            })
            .on_toggle_expand(move |dir_path, _win, cx| {
                let target_dir = dir_path.to_owned();
                state_expand.update(cx, |s, cx| {
                    s.toggle_directory_expanded(&target_dir);
                    cx.notify();
                });
            })
            .on_toggle_secret_unlock(move |secret_path, _win, cx| {
                let target_secret = secret_path.to_owned();
                state_secret.update(cx, |s, cx| {
                    s.toggle_secret_unlock_in_session(&target_secret);
                    cx.notify();
                });
            })
            .with_compact_header_actions(
                move |_ev, _win, cx| {
                    state_master.update(cx, |s, cx| {
                        s.toggle_select_all_files();
                        cx.notify();
                    });
                },
                move |_ev, _win, cx| {
                    state_fold.update(cx, |s, cx| {
                        s.toggle_expand_collapse_all();
                        cx.notify();
                    });
                },
                move |_ev, _win, cx| {
                    state_focus_search.update(cx, |s, cx| {
                        s.set_search_focused(true);
                        cx.notify();
                    });
                },
                move |_ev, _win, cx| {
                    state_clear_search.update(cx, |s, cx| {
                        s.clear_search_query();
                        cx.notify();
                    });
                },
            )
    }

    fn build_rules_editor(
        &self,
        preset: core::preset::models::preset::Preset,
        popover_open: bool,
    ) -> RulesEditorComponent {
        let labels = self.locale.labels();
        let theme = self.theme_variant.tokens();
        let state_rule = self.state.clone();
        let state_chunk = self.state.clone();
        let state_popover = self.state.clone();

        RulesEditorComponent::new(preset, labels)
            .with_theme(theme)
            .with_popover_open(popover_open)
            .on_toggle_rule(move |kind, _win, cx| {
                state_rule.update(cx, |s, cx| {
                    s.toggle_rule(kind);
                    cx.notify();
                });
            })
            .on_select_chunk_limit(move |limit_opt, _win, cx| {
                state_chunk.update(cx, |s, cx| {
                    s.set_max_chunk_bytes(limit_opt);
                    cx.notify();
                });
            })
            .on_toggle_popover(move |_ev, _win, cx| {
                state_popover.update(cx, |s, cx| {
                    s.toggle_filters_popover();
                    cx.notify();
                });
            })
    }

    fn build_live_preview(&self, props: LivePreviewProps) -> LivePreviewComponent {
        let state_chunk = self.state.clone();
        let state_open_meta = self.state.clone();
        let state_copy = self.state.clone();

        LivePreviewComponent::new(props)
            .track_scroll(self.preview_scroll_handle.clone())
            .on_select_chunk(move |idx, _win, cx| {
                state_chunk.update(cx, |s, cx| {
                    s.select_chunk(idx);
                    cx.notify();
                });
            })
            .on_open_meta(move |_ev, _win, cx| {
                state_open_meta.update(cx, |s, cx| {
                    s.request_open_meta_modal();
                    cx.notify();
                });
            })
            .on_copy(move |_ev, _win, cx| {
                let mut copied_text_opt: Option<String> = None;
                state_copy.update(cx, |s, cx| {
                    let _ = s.copy_active_chunk_to_clipboard();
                    copied_text_opt = Some(s.preview_buffer().as_str().to_owned());
                    cx.notify();
                });
                if let Some(text) = copied_text_opt {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            })
    }
}
