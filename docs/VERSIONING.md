# Versioning & Updater Guide

## Semantic Versioning (SemVer)
This project follows **Semantic Versioning** — format `MAJOR.MINOR.PATCH` (e.g., `1.2.4`).

### What each number means
| Part | When to bump | Example |
|------|-------------|---------|
| **MAJOR** | Breaking changes, incompatible API, architecture rewrite | `1.0.0` → `2.0.0` |
| **MINOR** | New feature (backward compatible) | `1.0.0` → `1.1.0` |
| **PATCH** | Bugfix, performance, minor improvements | `1.0.0` → `1.0.1` |

---

## 🔄 In-App Updater (Автообновления)
Проект содержит встроенный механизм обновлений, построенный по принципам Clean Architecture:
1. **GitHub API (`UpdaterService`)**: Приложение обращается к `https://api.github.com/repos/NIKIRIKI7/CodeContext/releases`.
2. **Парсинг версий**: Сравниваются теги релизов. Поддерживаются Pre-release версии (`-pre.N`), если в настройках стоит соответствующая галочка.
3. **Паттерн State (`UpdaterUseCase` + `Store`)**: Процесс загрузки прокидывается в глобальный State (`state.show_update`, `state.update_info`).
4. **Theme-Aware UI (`UpdateDialog`)**: Модальное окно реактивно подписывается на State и меняет свой UI (загрузка -> результат), используя цвета текущей темы.
5. **Безопасная замена файлов**: В Windows запущенный `exe` нельзя перезаписать. Приложение переименовывает себя в `.old`, скачивает новый файл на своё место и перезапускается. При следующем запуске `main.py` удаляет `.old`.

---

## Workflow
### Branches
- **`main`** — stable releases. Push → GitHub Actions creates a **Release**.
- **`dev`** — development branch. Push → GitHub Actions creates a **Pre-release** (`-pre.N`).

### Development cycle
#### 1. Start a new cycle
```bash
git checkout dev
bump-my-version bump patch      # 1.0.0 → 1.0.1 (Bugfix)
bump-my-version bump minor      # 1.0.0 → 1.1.0 (New feature)
bump-my-version bump major      # 1.0.0 → 2.0.0 (Breaking change)
```
This updates `VERSION.txt`, commits automatically, and you're ready to work.

#### 2. Develop & push
```bash
git add .
git commit -m "feat: some feature"
git push origin dev
```
GitHub Actions builds a pre-release (e.g., `v1.1.0-pre.42`). In-app updater can download it if "Prereleases" is checked.

#### 3. Release
Once tested, merge to `main`:
```bash
git checkout main
git merge dev
git push origin main
```
GitHub Actions creates a stable Release (e.g., `v1.1.0`) with all platform builds. Users will receive it automatically.

### After release
Go back to `dev`, bump the version for the next cycle:
```bash
git checkout dev
bump-my-version bump minor  # 1.1.0 → 1.2.0
```

## Automation

### bump-my-version
Installed tool that updates `VERSION.txt` and creates a commit automatically.

**Setup file:** `.bumpversion.toml` (in project root)

**Commands:**
```bash
bump-my-version bump patch   # bugfix
bump-my-version bump minor   # new feature
bump-my-version bump major   # breaking change
```

### How CI reads the version
The `.github/workflows/build.yml` reads `VERSION.txt`:
- `main` branch → stable release with tag `v{version}`
- `dev` branch → pre-release with tag `v{version}-pre.{run_number}`

## Quick reference
```
Current: 1.2.0
├── bump patch  → 1.2.1  (bugfix, no new features)
├── bump minor  → 1.3.0  (new feature, backward compatible)
└── bump major  → 2.0.0  (breaking changes)
```
