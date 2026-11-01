//! Архитектурные фитнес-функции проекта (DDD + Ports & Adapters + GPUI State-Entity-View).
//! Построены по принципам книги Влада Хононова «Learning Domain-Driven Design».
//! Запуск: `cargo test --package app --test architecture`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package, PackageId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Максимальное число `.rs` файлов в одной директории (без учёта `mod.rs` и `lib.rs`).
const MAX_IMPL_FILES_PER_DIR: usize = 4;

static METADATA_CACHE: OnceLock<Metadata> = OnceLock::new();

/// Мгновенно возвращает корень воркспейса без запуска внешнего процесса `cargo metadata`.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Не удалось определить корень воркспейса от CARGO_MANIFEST_DIR")
        .to_path_buf()
}

fn load_metadata() -> &'static Metadata {
    METADATA_CACHE.get_or_init(|| {
        MetadataCommand::new()
            .current_dir(workspace_root())
            .exec()
            .expect("Не удалось выполнить `cargo metadata`")
    })
}

/// Возвращает ПРЯМЫЕ боевые (Normal) зависимости пакета.
fn direct_normal_deps<'a>(metadata: &'a Metadata, pkg_id: &PackageId) -> HashSet<&'a str> {
    let resolve = metadata.resolve.as_ref().expect("Граф должен быть разрешён");
    let pkgs_by_id: HashMap<&PackageId, &Package> =
        metadata.packages.iter().map(|p| (&p.id, p)).collect();

    let mut result = HashSet::new();
    if let Some(node) = resolve.nodes.iter().find(|n| &n.id == pkg_id) {
        for dep in &node.deps {
            let is_normal = dep.dep_kinds.iter().any(|k| k.kind == DependencyKind::Normal);
            if is_normal && let Some(pkg) = pkgs_by_id.get(&dep.pkg) {
                result.insert(pkg.name.as_str());
            }
        }
    }
    result
}

/// Вычисляет все ТРАНЗИТИВНЫЕ боевые (Normal) зависимости пакета (не раскрывая внутренности `di`).
fn transitive_normal_deps<'a>(metadata: &'a Metadata, root_pkg_id: &PackageId) -> HashSet<&'a str> {
    let resolve = metadata.resolve.as_ref().expect("Граф должен быть разрешён");
    let nodes_by_id: HashMap<&PackageId, _> = resolve.nodes.iter().map(|n| (&n.id, n)).collect();
    let pkgs_by_id: HashMap<&PackageId, &Package> =
        metadata.packages.iter().map(|p| (&p.id, p)).collect();

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root_pkg_id);

    let mut result_names = HashSet::new();

    while let Some(current_id) = queue.pop_front() {
        if !visited.insert(current_id) {
            continue;
        }
        if let Some(node) = nodes_by_id.get(current_id) {
            for dep in &node.deps {
                let is_normal = dep.dep_kinds.iter().any(|k| k.kind == DependencyKind::Normal);

                if is_normal {
                    if let Some(pkg) = pkgs_by_id.get(&dep.pkg) {
                        result_names.insert(pkg.name.as_str());
                        if pkg.name == "di" {
                            continue;
                        }
                    }
                    queue.push_back(&dep.pkg);
                }
            }
        }
    }
    result_names
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

fn collect_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if dir.is_dir() {
        dirs.push(dir.to_path_buf());
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.extend(collect_dirs(&path));
                }
            }
        }
    }
    dirs
}

fn prod_code_slice(content: &str) -> &str {
    content.split("#[cfg(test)]").next().unwrap_or(content)
}

#[test]
fn test_1_ports_and_adapters_workspace_boundaries() {
    let metadata = load_metadata();

    let allowed_workspace_deps: HashMap<&str, &[&str]> = HashMap::from([
        ("core", &[][..]),
        ("config", &["core"][..]),
        ("scanner", &["core"][..]),
        ("apply", &["core"][..]),
        ("platform", &["core"][..]),
        ("di", &["core", "config", "scanner", "apply", "platform"][..]),
        ("ui", &["core", "di"][..]),
        ("app", &["core", "di", "ui", "platform", "config"][..]),
    ]);

    let workspace_pkgs = metadata.workspace_packages();
    let workspace_names: HashSet<&str> = workspace_pkgs.iter().map(|p| p.name.as_str()).collect();

    for pkg in workspace_pkgs {
        let pkg_name = pkg.name.as_str();
        let allowed = allowed_workspace_deps.get(pkg_name).unwrap_or_else(|| {
            panic!(
                "\n[DDD ОШИБКА ГРАНИЦ] Незарегистрированный крейт `{pkg_name}`!\n\
                Определите его роль в `tests/architecture.rs`.\n"
            )
        });

        let all_deps = transitive_normal_deps(metadata, &pkg.id);
        for dep in all_deps {
            assert!(
                !workspace_names.contains(dep) || allowed.contains(&dep),
                "\n[НАРУШЕНИЕ PORTS & ADAPTERS]\n\
                Крейт `{pkg_name}` зависит от `{dep}`.\n\
                Разрешённые зависимости для `{pkg_name}`: {allowed:?}\n"
            );
        }
    }
}

#[test]
fn test_2_generic_subdomain_acl_isolation() {
    let metadata = load_metadata();

    let forbidden_external: HashMap<&str, &[&str]> = HashMap::from([
        (
            "core",
            &[
                "gpui",
                "gpui-component",
                "arboard",
                "interprocess",
                "global-hotkey",
                "ignore",
                "similar",
                "serde_json",
            ][..],
        ),
        ("config", &["gpui", "gpui-component", "arboard", "ignore", "similar"][..]),
        ("scanner", &["gpui", "gpui-component", "arboard", "similar", "serde_json"][..]),
        ("apply", &["gpui", "gpui-component", "arboard", "ignore"][..]),
        ("platform", &["gpui", "gpui-component", "ignore", "similar"][..]),
        ("ui", &["ignore", "arboard", "interprocess", "similar"][..]),
    ]);

    for pkg in metadata.workspace_packages() {
        let pkg_name = pkg.name.as_str();
        if let Some(forbidden_list) = forbidden_external.get(pkg_name) {
            // Для ядра `core` проверяем весь транзитивный граф, а для адаптеров и `ui` — прямые зависимости
            let deps = if pkg_name == "core" {
                transitive_normal_deps(metadata, &pkg.id)
            } else {
                direct_normal_deps(metadata, &pkg.id)
            };

            for &forbidden in *forbidden_list {
                assert!(
                    !deps.contains(forbidden),
                    "\n[ПРОБОЙ ANTICORRUPTION LAYER (ACL)]\n\
                    Крейт `{pkg_name}` напрямую тянет библиотеку Generic-поддомена `{forbidden}`!\n\
                    Работайте только через доменные порты `core`.\n"
                );
            }
        }
    }
}

#[test]
fn test_3_workspace_lints_enforced_everywhere() {
    let crates_dir = workspace_root().join("crates");
    let crate_names = ["core", "config", "scanner", "apply", "platform", "di", "ui", "app"];

    for name in crate_names {
        let manifest_path = crates_dir.join(name).join("Cargo.toml");
        let manifest_content =
            fs::read_to_string(&manifest_path).expect("Не удалось прочитать Cargo.toml");
        let normalized = manifest_content.replace(' ', "");
        let has_workspace_lints = normalized.contains("[lints]\nworkspace=true")
            || normalized.contains("[lints]\r\nworkspace=true")
            || normalized.contains("lints.workspace=true");

        assert!(
            has_workspace_lints,
            "\n[НАРУШЕНИЕ КАЧЕСТВА КОДА]\n\
            В `{}` отсутствует `[lints] workspace = true`.\n",
            manifest_path.display()
        );
    }
}

#[test]
fn test_4_ddd_core_subdomains_and_context_map() {
    let core_src = workspace_root().join("crates").join("core").join("src");
    if !core_src.exists() {
        return;
    }

    let allowed_subdomains: HashSet<&str> =
        HashSet::from(["shared_kernel", "packing", "applying", "preset", "system"]);

    if let Ok(entries) = fs::read_dir(&core_src) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap().to_string_lossy();
                assert!(
                    allowed_subdomains.contains(dir_name.as_ref()),
                    "\n[АНТИПАТТЕРН ТЕХНИЧЕСКОЙ НАРЕЗКИ В КОРНЕ CORE (Рис. 13-1 DDD)]\n\
                    В `crates/core/src/` обнаружена папка `{dir_name}`.\n\
                    Корень `core/src` должен делиться строго по бизнес-поддоменам: {allowed_subdomains:?}.\n"
                );
            }
        }
    }

    let context_map_forbidden: &[(&str, &[&str])] = &[
        ("shared_kernel", &["crate::packing", "crate::applying", "crate::preset", "crate::system"]),
        ("preset", &["crate::packing", "crate::applying", "crate::system"]),
        ("system", &["crate::packing", "crate::applying", "crate::preset"]),
        ("packing", &["crate::applying", "crate::system"]),
        ("applying", &["crate::packing", "crate::system"]),
    ];

    for (subdomain, forbidden_list) in context_map_forbidden {
        let sub_dir = core_src.join(subdomain);
        if !sub_dir.exists() {
            continue;
        }
        for file_path in collect_rs_files(&sub_dir) {
            let content = fs::read_to_string(&file_path).expect("Не удалось прочитать файл");
            let prod_code = prod_code_slice(&content);
            for &forbidden in *forbidden_list {
                assert!(
                    !prod_code.contains(forbidden),
                    "\n[НАРУШЕНИЕ КАРТЫ КОНТЕКСТОВ (CONTEXT MAP) В CORE]\n\
                    Файл `{}` в поддомене `{subdomain}` импортирует `{forbidden}`.\n",
                    file_path.display()
                );
            }
        }
    }
}

#[test]
fn test_5_ddd_tactical_building_blocks_invariants() {
    let core_src = workspace_root().join("crates").join("core").join("src");
    if !core_src.exists() {
        return;
    }

    for file_path in collect_rs_files(&core_src) {
        let path_str = file_path.to_string_lossy().replace('\\', "/");
        let content = fs::read_to_string(&file_path).expect("Не удалось прочитать файл");
        let prod_code = prod_code_slice(&content);

        if path_str.contains("/values/") || path_str.contains("/shared_kernel/fs/") {
            for forbidden in ["aggregates::", "models::", "ports::", "services::", "&mut self"] {
                assert!(
                    !prod_code.contains(forbidden),
                    "\n[НАРУШЕНИЕ ИНВАРИАНТА VALUE OBJECT (Гл. 6 DDD)]\n\
                    Файл Value Object `{}` содержит запрещённую конструкцию `{forbidden}`.\n",
                    file_path.display()
                );
            }
        }

        if path_str.contains("/aggregates/") || path_str.contains("/models/") {
            for forbidden in ["ports::", "services::"] {
                assert!(
                    !prod_code.contains(forbidden),
                    "\n[НАРУШЕНИЕ ЧИСТОТЫ АГРЕГАТА (Гл. 6 DDD)]\n\
                    Файл `{}` обращается к `{forbidden}`.\n",
                    file_path.display()
                );
            }
        }

        if path_str.contains("/ports/") {
            assert!(
                !prod_code.contains("services::"),
                "\n[НАРУШЕНИЕ НАПРАВЛЕНИЯ ЗАВИСИМОСТЕЙ ПОРТОВ]\n\
                Порт `{}` импортирует `services::`.\n",
                file_path.display()
            );
        }
    }
}

#[test]
fn test_6_gpui_state_entity_view_conventions() {
    let features_dir = workspace_root().join("crates").join("ui").join("src").join("features");

    if !features_dir.exists() {
        return;
    }

    let entries = fs::read_dir(&features_dir).expect("Не удалось прочитать features");
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let feature_name = path.file_name().unwrap().to_string_lossy();
        let state_file = path.join("state.rs");
        let view_file = path.join("view.rs");
        let mod_file = path.join("mod.rs");

        assert!(
            state_file.exists() && view_file.exists() && mod_file.exists(),
            "\n[НАРУШЕНИЕ СТРУКТУРЫ ФИЧИ UI]\n\
            Фича `ui::features::{feature_name}` обязана содержать `state.rs`, `view.rs` и `mod.rs`.\n"
        );

        let state_code = fs::read_to_string(&state_file).unwrap();
        assert!(
            !state_code.contains("impl Render for") && !state_code.contains("impl RenderOnce for"),
            "\n[НАРУШЕНИЕ STATE-ENTITY-VIEW]\n\
            `{}` реализует `Render`/`RenderOnce`.\n",
            state_file.display()
        );

        let components_dir = path.join("components");
        if components_dir.exists() {
            for comp_file in collect_rs_files(&components_dir) {
                let comp_code = fs::read_to_string(&comp_file).unwrap();
                assert!(
                    !comp_code.contains("Entity<"),
                    "\n[НАРУШЕНИЕ ИНКАПСУЛЯЦИИ КОМПОНЕНТОВ GPUI]\n\
                    Компонент `{}` хранит `Entity<...>`.\n",
                    comp_file.display()
                );
            }
        }
    }
}

#[test]
fn test_8_max_files_per_directory_cohesion() {
    let crates_dir = workspace_root().join("crates");
    let crate_names = ["core", "config", "scanner", "apply", "platform", "di", "ui", "app"];

    for pkg_name in crate_names {
        let src_dir = crates_dir.join(pkg_name).join("src");
        if !src_dir.exists() {
            continue;
        }

        for dir in collect_dirs(&src_dir) {
            let impl_files_count = fs::read_dir(&dir)
                .expect("Не удалось прочитать директорию")
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("rs"))
                .filter(|p| {
                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    name != "mod.rs" && name != "lib.rs" && name != "main.rs"
                })
                .count();

            assert!(
                impl_files_count <= MAX_IMPL_FILES_PER_DIR,
                "\n[ПРЕВЫШЕН ЛИМИТ ФАЙЛОВ В ПАПКЕ (Anti-Big-Ball-of-Mud)]\n\
                Директория `{}` содержит {impl_files_count} файлов реализации (лимит: {MAX_IMPL_FILES_PER_DIR}).\n",
                dir.display()
            );
        }
    }
}
