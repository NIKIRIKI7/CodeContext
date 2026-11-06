# Справочник манифеста `plugin.json` и Точек расширения (`ContributionPoints`)

Каждый плагин размещается в собственной папке внутри `~/.config/CodeContext/plugins/<plugin_id>/` (или локально в `.ctxpack/plugins/<plugin_id>/`) и содержит файл `plugin.json`.

---

## 1. Полная структура `plugin.json`

```json
{
  "id": "company.git-branch-context",
  "name": "Git Branch & Diff Context",
  "version": "1.0.0",
  "description": "Прикрепляет статус ветки Git и последние коммиты к контексту",
  "author": "CodeContext Contributors",
  "entrypoint": "python3 main.py",
  "activation_events": [
    "on_pack_context",
    { "on_command": "git.inject_branch_summary" }
  ],
  "permissions": [
    "workspace_selection",
    "fs_read_project"
  ],
  "contributes": {
    "commands": [
      {
        "id": "git.inject_branch_summary",
        "title": "Git: Скопировать сводку текущей ветки",
        "default_shortcut": "Ctrl+Shift+G",
        "category": "Git"
      }
    ],
    "views": [
      {
        "id": "git_context_panel",
        "title": "Git Контекст",
        "icon": "git",
        "location": "bottom_dock"
      }
    ],
    "context_providers": [
      {
        "id": "git_branch_status",
        "title": "Статус ветки и последние 5 коммитов",
        "header_tag": "GIT_CONTEXT",
        "enabled_by_default": true
      }
    ],
    "code_transforms": [
      {
        "id": "rust_skeleton",
        "title": "Только публичные сигнатуры (.rs)",
        "target_extensions": ["rs"]
      }
    ]
  }
}
```