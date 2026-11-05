use core::packing::aggregates::project::Project;
use gpui::{
    ClipboardItem, Context, Entity, IntoElement, ParentElement as _, Render, ScrollStrategy,
    Styled as _, UniformListScrollHandle, Window, div,
};

use crate::features::context_packer::components::file_tree::FileTreeComponent;
use crate::features::context_packer::components::filter_modal::{
    FilterModalComponent, SystemPromptModalComponent,
};
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
        if let Some(tree_row) = snapshot.take_pending_tree_scroll() {
            self.tree_scroll_handle.scroll_to_item(tree_row, ScrollStrategy::Top);
        }
        if let Some(preview_line) = snapshot.take_pending_preview_scroll() {
            self.preview_scroll_handle.scroll_to_item_strict(preview_line, ScrollStrategy::Top);
        }
        let banner_items = snapshot.active_banner_items_localized(self.locale);
        let tree_projection = snapshot.tree_projection().clone();
        let focused_tree_index = snapshot.focused_tree_index();
        let sort_mode = snapshot.sort_mode();
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
            system_prompt_mode: active_preset.template.system_prompt_preset,
            copied_recently: ui_status.copied_recently,
            labels,
            theme,
        };

        let active_filters_count = snapshot.active_custom_filter_rules_count();
        let active_prompts_count = active_preset.template.active_prompt_blocks_count();
        let filter_modal_props_opt =
            ui_status.regex_filter_open.then(|| snapshot.build_filter_modal_props(labels, theme));
        let prompt_modal_props_opt = snapshot
            .is_prompt_modal_open()
            .then(|| snapshot.build_system_prompt_modal_props(labels, theme));

        let banner = self.build_warning_banner(banner_items);
        let file_tree = self.build_file_tree(
            tree_projection,
            focused_tree_index,
            sort_mode,
            search_query,
            ui_status,
            active_filters_count,
            all_collapsed,
        );
        let rules_editor = self.build_rules_editor(
            active_preset,
            ui_status.filters_popover_open,
            active_prompts_count,
        );
        let live_preview = self.build_live_preview(preview_props);

        let left_column = div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(div().flex_1().overflow_hidden().child(file_tree))
            .child(rules_editor);

        let mut root = div()
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
            .child(banner);

        if let Some(modal_props) = filter_modal_props_opt {
            root = root.child(self.build_filter_modal(modal_props));
        }

        if let Some(prompt_props) = prompt_modal_props_opt {
            root = root.child(self.build_system_prompt_modal(prompt_props));
        }

        root
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

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn build_file_tree(
        &self,
        projection: crate::features::context_packer::components::file_tree::FlatTreeProjection,
        focused_tree_index: Option<usize>,
        sort_mode: crate::features::context_packer::components::file_tree::TreeSortMode,
        search_query: String,
        ui_status: crate::features::context_packer::state::PackerUiStatus,
        active_filters_count: usize,
        all_collapsed: bool,
    ) -> FileTreeComponent {
        let labels = self.locale.labels();
        let theme = self.theme_variant.tokens();
        let state_check = self.state.clone();
        let state_expand = self.state.clone();
        let state_select_file = self.state.clone();
        let state_secret = self.state.clone();
        let state_master = self.state.clone();
        let state_fold = self.state.clone();
        let state_sort = self.state.clone();
        let state_regex_toggle = self.state.clone();
        let state_focus_search = self.state.clone();
        let state_clear_search = self.state.clone();
        let state_open_proj = self.state.clone();
        let prompt_label = labels.btn_open_folder_prompt;

        FileTreeComponent::new(projection, labels)
            .with_theme(theme)
            .with_focused_index(focused_tree_index)
            .with_sort_mode(sort_mode)
            .with_search_state(search_query, ui_status.search_focused, all_collapsed)
            .with_filter_modal_indicator(ui_status.regex_filter_open, active_filters_count)
            .track_scroll(self.tree_scroll_handle.clone())
            .on_toggle_regex_mode(move |_ev, _win, cx| {
                state_regex_toggle.update(cx, |s, cx| {
                    s.toggle_regex_filter_panel();
                    cx.notify();
                });
            })
            .on_toggle_sort_mode(move |_ev, _win, cx| {
                state_sort.update(cx, |s, cx| {
                    s.toggle_tree_sort_mode();
                    cx.notify();
                });
            })
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
            .on_select_file(move |row_idx, file_path, _win, cx| {
                let target_file = file_path.to_owned();
                state_select_file.update(cx, |s, cx| {
                    s.select_file_in_tree_and_jump(row_idx, &target_file);
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
        active_prompts_count: usize,
    ) -> RulesEditorComponent {
        let labels = self.locale.labels();
        let theme = self.theme_variant.tokens();
        let state_rule = self.state.clone();
        let state_chunk = self.state.clone();
        let state_popover = self.state.clone();
        let state_open_prompts = self.state.clone();

        RulesEditorComponent::new(preset, labels)
            .with_theme(theme)
            .with_popover_open(popover_open)
            .with_prompts_count(active_prompts_count)
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
            .on_open_prompts_modal(move |_ev, _win, cx| {
                state_open_prompts.update(cx, |s, cx| {
                    s.set_prompt_modal_open(true);
                    cx.notify();
                });
            })
    }

    #[allow(clippy::too_many_lines)]
    fn build_filter_modal(
        &self,
        props: crate::features::context_packer::components::filter_modal::FilterModalProps,
    ) -> FilterModalComponent {
        let state_close = self.state.clone();
        let state_reset = self.state.clone();
        let state_ext_mode = self.state.clone();
        let state_toggle_ext = self.state.clone();
        let state_focus_ext = self.state.clone();
        let state_submit_ext = self.state.clone();
        let state_toggle_folder = self.state.clone();
        let state_focus_folder = self.state.clone();
        let state_submit_folder = self.state.clone();
        let state_re_scope = self.state.clone();
        let state_re_focus = self.state.clone();
        let state_re_name_focus = self.state.clone();
        let state_re_submit_tpl = self.state.clone();
        let state_re_clear = self.state.clone();
        let state_re_case = self.state.clone();
        let state_re_preset = self.state.clone();
        let state_re_tpl_idx = self.state.clone();
        let state_re_del_idx = self.state.clone();

        FilterModalComponent::new(props)
            .on_close(move |_ev, _win, cx| {
                state_close.update(cx, |s, cx| {
                    s.set_regex_filter_open(false);
                    cx.notify();
                });
            })
            .on_reset_all(move |_ev, _win, cx| {
                state_reset.update(cx, |s, cx| {
                    s.reset_all_filters();
                    cx.notify();
                });
            })
            .on_select_ext_mode(move |mode, _win, cx| {
                state_ext_mode.update(cx, |s, cx| {
                    s.set_extension_filter_mode(mode);
                    cx.notify();
                });
            })
            .on_toggle_extension(move |ext, _win, cx| {
                let ext_owned = ext.to_owned();
                state_toggle_ext.update(cx, |s, cx| {
                    s.toggle_extension_in_modal(&ext_owned);
                    cx.notify();
                });
            })
            .on_focus_extension_input(move |_ev, _win, cx| {
                state_focus_ext.update(cx, |s, cx| {
                    s.set_extension_input_focused(true);
                    cx.notify();
                });
            })
            .on_submit_extension_input(move |_ev, _win, cx| {
                state_submit_ext.update(cx, |s, cx| {
                    s.submit_manual_extension();
                    cx.notify();
                });
            })
            .on_toggle_folder(move |folder, _win, cx| {
                let folder_owned = folder.to_owned();
                state_toggle_folder.update(cx, |s, cx| {
                    s.toggle_quick_filter_chip(
                        &crate::features::context_packer::components::file_tree::QuickFilterChipKind::ExcludedFolder(
                            folder_owned,
                        ),
                    );
                    cx.notify();
                });
            })
            .on_focus_folder_input(move |_ev, _win, cx| {
                state_focus_folder.update(cx, |s, cx| {
                    s.set_folder_input_focused(true);
                    cx.notify();
                });
            })
            .on_submit_folder_input(move |_ev, _win, cx| {
                state_submit_folder.update(cx, |s, cx| {
                    s.submit_manual_folder_exclusion();
                    cx.notify();
                });
            })
            .on_select_regex_scope(move |scope, _win, cx| {
                state_re_scope.update(cx, |s, cx| {
                    s.select_regex_scope(scope);
                    cx.notify();
                });
            })
            .on_focus_regex_input(move |_ev, _win, cx| {
                state_re_focus.update(cx, |s, cx| {
                    s.set_regex_input_focused(true);
                    cx.notify();
                });
            })
            .on_focus_regex_name_input(move |_ev, _win, cx| {
                state_re_name_focus.update(cx, |s, cx| {
                    s.set_regex_name_input_focused(true);
                    cx.notify();
                });
            })
            .on_submit_regex_template(move |_ev, _win, cx| {
                state_re_submit_tpl.update(cx, |s, cx| {
                    if s.submit_manual_regex_template() {
                        cx.notify();
                    }
                });
            })
            .on_clear_regex_input(move |_ev, _win, cx| {
                state_re_clear.update(cx, |s, cx| {
                    s.clear_regex_draft();
                    cx.notify();
                });
            })
            .on_toggle_regex_case(move |_ev, _win, cx| {
                state_re_case.update(cx, |s, cx| {
                    s.toggle_regex_case_sensitive();
                    cx.notify();
                });
            })
            .on_apply_regex_preset(move |pattern, _win, cx| {
                let pat_owned = pattern.to_owned();
                state_re_preset.update(cx, |s, cx| {
                    s.apply_regex_preset_pattern(&pat_owned);
                    cx.notify();
                });
            })
            .on_apply_regex_template_idx(move |idx, _win, cx| {
                state_re_tpl_idx.update(cx, |s, cx| {
                    s.apply_regex_template_by_index(idx);
                    cx.notify();
                });
            })
            .on_delete_regex_template_idx(move |idx, _win, cx| {
                state_re_del_idx.update(cx, |s, cx| {
                    if s.delete_regex_template(idx) {
                        cx.notify();
                    }
                });
            })
    }

    fn build_system_prompt_modal(
        &self,
        props: crate::features::context_packer::components::filter_modal::SystemPromptModalProps,
    ) -> SystemPromptModalComponent {
        let state_cat = self.state.clone();
        let state_toggle = self.state.clone();
        let state_up = self.state.clone();
        let state_down = self.state.clone();
        let state_del = self.state.clone();
        let state_focus_p = self.state.clone();
        let state_submit_p = self.state.clone();
        let state_paste = self.state.clone();
        let state_copy = self.state.clone();
        let state_reset = self.state.clone();
        let state_close = self.state.clone();

        SystemPromptModalComponent::new(props)
            .on_select_category(move |cat_opt, _win, cx| {
                state_cat.update(cx, |s, cx| {
                    s.set_prompt_category_filter(cat_opt);
                    cx.notify();
                });
            })
            .on_toggle_block(move |idx, _win, cx| {
                state_toggle.update(cx, |s, cx| {
                    s.toggle_prompt_block(idx);
                    cx.notify();
                });
            })
            .on_move_up(move |idx, _win, cx| {
                state_up.update(cx, |s, cx| {
                    s.move_prompt_block_up(idx);
                    cx.notify();
                });
            })
            .on_move_down(move |idx, _win, cx| {
                state_down.update(cx, |s, cx| {
                    s.move_prompt_block_down(idx);
                    cx.notify();
                });
            })
            .on_delete_block(move |idx, _win, cx| {
                state_del.update(cx, |s, cx| {
                    s.delete_prompt_block(idx);
                    cx.notify();
                });
            })
            .on_focus_prompt_input(move |_ev, _win, cx| {
                state_focus_p.update(cx, |s, cx| {
                    s.set_prompt_input_focused(true);
                    cx.notify();
                });
            })
            .on_submit_prompt_input(move |_ev, _win, cx| {
                state_submit_p.update(cx, |s, cx| {
                    s.submit_manual_prompt_draft();
                    cx.notify();
                });
            })
            .with_actions(
                move |_ev, _win, cx| {
                    let clip_opt = cx.read_from_clipboard().and_then(|i| i.text());
                    if let Some(text) = clip_opt {
                        state_paste.update(cx, |s, cx| {
                            let cat = s
                                .prompt_category_filter()
                                .unwrap_or(core::preset::models::template::PromptCategory::Custom);
                            s.add_custom_prompt_block(cat, text.as_str());
                            cx.notify();
                        });
                    }
                },
                move |_ev, _win, cx| {
                    let text = state_copy
                        .read(cx)
                        .effective_preset()
                        .template
                        .effective_preamble()
                        .to_owned();
                    if !text.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                    }
                },
                move |_ev, _win, cx| {
                    state_reset.update(cx, |s, cx| {
                        s.reset_system_prompt_to_builtin();
                        cx.notify();
                    });
                },
                move |_ev, _win, cx| {
                    state_close.update(cx, |s, cx| {
                        s.set_prompt_modal_open(false);
                        cx.notify();
                    });
                },
            )
    }

    fn build_live_preview(&self, props: LivePreviewProps) -> LivePreviewComponent {
        let state_chunk = self.state.clone();
        let state_prompt = self.state.clone();
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
            .on_open_prompt_modal(move |_ev, _win, cx| {
                state_prompt.update(cx, |s, cx| {
                    s.set_prompt_modal_open(true);
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
