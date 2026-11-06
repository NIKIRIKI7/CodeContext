# Справочник Host API и JSON-RPC 2.0 протокола

Внешние плагины взаимодействуют с `SystemPluginHost` (`crates/platform/src/plugins/host.rs`) через стандартные потоки `stdin` / `stdout` в формате **JSON-RPC 2.0**.

---

## 1. Формат вызова хука от Хоста к Плагину

При наступлении события активации Хост отправляет однострочный JSON-пакет в `stdin` плагина:

```json
{"jsonrpc":"2.0","id":1,"method":"onPackContext","params":{"project_root":"/workspace/my-project"}}
```

## 2. Встроенные плагины ядра (`Built-in Plugins`)

По умолчанию в `SystemPluginHost` поставляются два встроенных расширения (доступны в `Настройки -> Модули и панели`):
1. **`builtin.git_context`** — автоматический сбор статуса ветки Git (`git status -sb`) и инжекция в контекст.
2. **`builtin.ast_signatures`** — компрессор Rust-файлов (`extract_rust_public_signatures`), оставляющий только публичные структуры, трейты и сигнатуры функций с экономией токенов в **5–8 раз**.