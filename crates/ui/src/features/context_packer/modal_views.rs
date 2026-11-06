use gpui::ClipboardItem;

use crate::features::context_packer::components::file_tree::QuickFilterChipKind;
use crate::features::context_packer::components::filter_modal::{
    FilterModalComponent, FilterModalProps, SystemPromptModalComponent, SystemPromptModalProps,
};
use crate::features::context_packer::view::ContextPackerView;

impl ContextPackerView {
    pub(crate) fn build_filter_modal(&self, props: FilterModalProps) -> FilterModalComponent {
        let state_close = self.state().clone();
        let state_reset = self.state().clone();
        let state_ext_mode = self.state().clone();
        let state_toggle_ext = self.state().clone();
        let state_focus_ext = self.state().clone();
        let state_submit_ext = self.state().clone();
        let state_toggle_folder = self.state().clone();
        let state_focus_folder = self.state().clone();
        let state_submit_folder = self.state().clone();

        let base = FilterModalComponent::new(props)
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
                    s.toggle_quick_filter_chip(&QuickFilterChipKind::ExcludedFolder(folder_owned));
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
            });

        self.attach_regex_modal_handlers(base)
    }

    fn attach_regex_modal_handlers(&self, modal: FilterModalComponent) -> FilterModalComponent {
        let state_re_scope = self.state().clone();
        let state_re_focus = self.state().clone();
        let state_re_name_focus = self.state().clone();
        let state_re_submit_tpl = self.state().clone();
        let state_re_clear = self.state().clone();
        let state_re_case = self.state().clone();
        let state_re_preset = self.state().clone();
        let state_re_tpl_idx = self.state().clone();
        let state_re_del_idx = self.state().clone();

        modal
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

    pub(crate) fn build_system_prompt_modal(
        &self,
        props: SystemPromptModalProps,
    ) -> SystemPromptModalComponent {
        let state_cat = self.state().clone();
        let state_toggle = self.state().clone();
        let state_up = self.state().clone();
        let state_down = self.state().clone();
        let state_del = self.state().clone();
        let state_focus_cat = self.state().clone();
        let state_focus_title = self.state().clone();
        let state_focus_p = self.state().clone();
        let state_submit_p = self.state().clone();
        let state_paste = self.state().clone();
        let state_copy = self.state().clone();
        let state_reset = self.state().clone();
        let state_close = self.state().clone();

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
            .on_focus_prompt_category(move |_ev, _win, cx| {
                state_focus_cat.update(cx, |s, cx| {
                    s.set_prompt_category_focused(true);
                    cx.notify();
                });
            })
            .on_focus_prompt_title(move |_ev, _win, cx| {
                state_focus_title.update(cx, |s, cx| {
                    s.set_prompt_title_focused(true);
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
                            s.paste_into_prompt_composer(text.as_str());
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
}
