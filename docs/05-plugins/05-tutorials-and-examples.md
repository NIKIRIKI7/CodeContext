# Практические рецепты создания плагинов

## Рецепт 1: Плагин на Python за 2 минуты (`@git-summary`)

1. Создайте папку `~/.config/CodeContext/plugins/git_summary/`.
2. Создайте файл `plugin.json`:
   ```json
   {
     "id": "custom.git_summary",
     "name": "Git Summary Provider",
     "version": "0.1.0",
     "description": "Добавляет последние 3 коммита в контекст",
     "author": "Dev",
     "entrypoint": "python3 plugin.py"
   }
   ```
3. Создайте `plugin.py`:
   ```python
   import json, subprocess, sys

   req = json.loads(sys.stdin.readline())
   root = req.get("params", {}).get("project_root", ".")
   out = subprocess.check_output(["git", "-C", root, "log", "-n", "3", "--oneline"], text=True)
   print(out.strip())
   ```
4. Включите плагин в `Настройки -> Модули и панели`.