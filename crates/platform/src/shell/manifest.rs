use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use core::system::ports::shell::{ShellMenuConfig, ShellRegistrationStatus};

pub const WINDOWS_DEFAULT_CLASSES_PREFIX: &str = "Software\\Classes";
pub(crate) const MACOS_WORKFLOW_BUNDLE_NAME: &str = "CodeContext Copy.workflow";
pub(crate) const MACOS_PRIMARY_WFLOW_REL_PATH: &str =
    "CodeContext Copy.workflow/Contents/document.wflow";
pub(crate) const LINUX_PRIMARY_DESKTOP_REL_PATH: &str = "kio/servicemenus/codecontext.desktop";

const PROJECT_ROOT_MARKERS: &[&str] =
    &[".ctxpack.json", ".git", "Cargo.toml", "package.json", "pyproject.toml", "go.mod"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryKeySpec {
    pub relative_key_path: String,
    pub default_value: String,
    pub named_values: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsRegistryManifest {
    pub classes_prefix: String,
    pub expected_pack_dir_command: String,
    pub expected_apply_dir_command: String,
    pub keys: Vec<RegistryKeySpec>,
    pub root_keys_to_delete: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellFileArtifact {
    pub rel_path: String,
    pub content: String,
    pub executable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileManifest {
    pub expected_marker: String,
    pub primary_rel_path: String,
    pub artifacts: Vec<ShellFileArtifact>,
    pub root_entries_to_delete: Vec<String>,
}

#[inline]
pub fn sanitize_executable_path(exe_path: &Path) -> String {
    let raw = exe_path.to_string_lossy();
    let stripped = if let Some(unc_rest) = raw.strip_prefix("\\\\?\\UNC\\") {
        format!("\\\\{unc_rest}")
    } else if let Some(dos_rest) = raw.strip_prefix("\\\\?\\") {
        dos_rest.to_owned()
    } else {
        raw.into_owned()
    };
    if stripped.contains('\x22') { stripped.replace('\x22', "") } else { stripped }
}

#[inline]
pub(crate) fn expected_windows_pack_dir_command(config: &ShellMenuConfig) -> String {
    let exe = sanitize_executable_path(&config.executable_path);
    format!("\"{exe}\" --pack-headless \"%V\\.\"")
}

#[inline]
pub(crate) fn expected_windows_apply_dir_command(config: &ShellMenuConfig) -> String {
    let exe = sanitize_executable_path(&config.executable_path);
    format!("\"{exe}\" --apply-headless \"%V\\.\"")
}

#[inline]
pub(crate) fn expected_unix_pack_marker(config: &ShellMenuConfig) -> String {
    let exe = sanitize_executable_path(&config.executable_path);
    format!("\"{exe}\" --pack-headless")
}

pub fn sanitize_explorer_path_arg(raw_arg: &str) -> PathBuf {
    let mut s = raw_arg.trim();
    while (s.starts_with('\x22') && s.ends_with('\x22') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s = s.get(1..s.len() - 1).unwrap_or(s).trim();
    }

    s = s.trim_matches('\x22').trim();

    if let Some(without_unc) = s.strip_prefix("\\\\?\\") {
        s = without_unc;
    }

    if let Some(without_dot) = s.strip_suffix("\\.") {
        s = if without_dot.ends_with(':') {
            return PathBuf::from(format!("{without_dot}\\"));
        } else {
            without_dot
        };
    } else if let Some(without_dot) = s.strip_suffix("/.") {
        s = if without_dot.is_empty() { "/" } else { without_dot };
    }

    if s.len() == 2 && s.as_bytes().get(1).copied() == Some(b':') {
        return PathBuf::from(format!("{s}\\"));
    }

    if s.is_empty() { PathBuf::from(".") } else { PathBuf::from(s) }
}

pub fn resolve_headless_target_path(raw_arg: &str) -> PathBuf {
    let cleaned = sanitize_explorer_path_arg(raw_arg);
    if cleaned.is_file() {
        return find_project_root_from_file(&cleaned);
    }
    cleaned
}

fn find_project_root_from_file(file_path: &Path) -> PathBuf {
    let Some(immediate_parent) = file_path.parent() else {
        return PathBuf::from(".");
    };
    if immediate_parent.as_os_str().is_empty() {
        return PathBuf::from(".");
    }

    let mut cursor = Some(immediate_parent);
    while let Some(dir) = cursor {
        for &marker in PROJECT_ROOT_MARKERS {
            if dir.join(marker).exists() {
                return dir.to_path_buf();
            }
        }
        cursor = dir.parent().filter(|p| !p.as_os_str().is_empty());
    }

    immediate_parent.to_path_buf()
}

pub fn build_windows_registry_manifest(
    classes_prefix: &str,
    config: &ShellMenuConfig,
) -> WindowsRegistryManifest {
    let prefix = classes_prefix.trim_end_matches('\\');
    let exe = sanitize_executable_path(&config.executable_path);
    let icon_val = format!("\"{exe}\",0");

    let pack_dir_cmd = format!("\"{exe}\" --pack-headless \"%V\\.\"");
    let pack_file_cmd = format!("\"{exe}\" --pack-headless \"%1\"");
    let apply_dir_cmd = format!("\"{exe}\" --apply-headless \"%V\\.\"");
    let apply_file_cmd = format!("\"{exe}\" --apply-headless \"%1\"");
    let open_dir_cmd = format!("\"{exe}\" --project \"%V\\.\"");
    let open_file_cmd = format!("\"{exe}\" --project \"%1\"");

    let mut keys = Vec::with_capacity(21);

    for &(scope, pack_cmd, apply_cmd, open_cmd) in &[
        (
            "Directory\\shell\\CodeContext",
            pack_dir_cmd.as_str(),
            apply_dir_cmd.as_str(),
            open_dir_cmd.as_str(),
        ),
        (
            "Directory\\Background\\shell\\CodeContext",
            pack_dir_cmd.as_str(),
            apply_dir_cmd.as_str(),
            open_dir_cmd.as_str(),
        ),
        (
            "*\\shell\\CodeContext",
            pack_file_cmd.as_str(),
            apply_file_cmd.as_str(),
            open_file_cmd.as_str(),
        ),
    ] {
        let base_key = format!("{prefix}\\{scope}");
        keys.push(RegistryKeySpec {
            relative_key_path: base_key.clone(),
            default_value: String::new(),
            named_values: vec![
                ("MUIVerb".to_owned(), config.root_menu_label.clone()),
                ("Icon".to_owned(), icon_val.clone()),
                ("SubCommands".to_owned(), String::new()),
            ],
        });

        for &(sub_id, label, cmd) in &[
            ("01_pack", &config.pack_menu_label, pack_cmd),
            ("02_apply", &config.apply_menu_label, apply_cmd),
            ("03_open_gui", &config.open_gui_label, open_cmd),
        ] {
            let item_key = format!("{base_key}\\shell\\{sub_id}");
            let cmd_key = format!("{item_key}\\command");
            keys.push(RegistryKeySpec {
                relative_key_path: item_key,
                default_value: label.clone(),
                named_values: vec![
                    ("MUIVerb".to_owned(), label.clone()),
                    ("Icon".to_owned(), icon_val.clone()),
                ],
            });
            keys.push(RegistryKeySpec {
                relative_key_path: cmd_key,
                default_value: cmd.to_owned(),
                named_values: Vec::new(),
            });
        }
    }

    let root_keys_to_delete = vec![
        format!("{prefix}\\Directory\\shell\\CodeContext"),
        format!("{prefix}\\Directory\\Background\\shell\\CodeContext"),
        format!("{prefix}\\*\\shell\\CodeContext"),
        format!("{prefix}\\Directory\\shell\\CodeContext.Pack"),
        format!("{prefix}\\Directory\\Background\\shell\\CodeContext.Pack"),
        format!("{prefix}\\*\\shell\\CodeContext.Pack"),
        format!("{prefix}\\Directory\\shell\\CodeContext.OpenGui"),
        format!("{prefix}\\Directory\\Background\\shell\\CodeContext.OpenGui"),
    ];

    WindowsRegistryManifest {
        classes_prefix: prefix.to_owned(),
        expected_pack_dir_command: pack_dir_cmd,
        expected_apply_dir_command: apply_dir_cmd,
        keys,
        root_keys_to_delete,
    }
}

pub fn build_macos_workflow_manifest(config: &ShellMenuConfig) -> FileManifest {
    let exe = sanitize_executable_path(&config.executable_path);
    let expected_marker = format!("\"{exe}\" --pack-headless");

    let info_plist = format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
            "<plist version=\"1.0\">\n",
            "<dict>\n",
            "    <key>NSServices</key>\n",
            "    <array>\n",
            "        <dict>\n",
            "            <key>NSMenuItem</key>\n",
            "            <dict>\n",
            "                <key>default</key>\n",
            "                <string>{label}</string>\n",
            "            </dict>\n",
            "            <key>NSMessage</key>\n",
            "            <string>runWorkflowAsService</string>\n",
            "            <key>NSSendFileTypes</key>\n",
            "            <array>\n",
            "                <string>public.folder</string>\n",
            "                <string>public.item</string>\n",
            "            </array>\n",
            "        </dict>\n",
            "    </array>\n",
            "</dict>\n",
            "</plist>\n"
        ),
        label = xml_escape(&config.pack_menu_label)
    );

    let document_wflow = format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
            "<plist version=\"1.0\">\n",
            "<dict>\n",
            "    <key>COMMAND_STRING</key>\n",
            "    <string>\"{exe}\" --pack-headless \"$1\"</string>\n",
            "</dict>\n",
            "</plist>\n"
        ),
        exe = xml_escape(&exe)
    );

    let wflow_rel = MACOS_PRIMARY_WFLOW_REL_PATH.to_owned();
    let plist_rel = format!("{MACOS_WORKFLOW_BUNDLE_NAME}/Contents/Info.plist");

    FileManifest {
        expected_marker,
        primary_rel_path: wflow_rel.clone(),
        artifacts: vec![
            ShellFileArtifact { rel_path: plist_rel, content: info_plist, executable: false },
            ShellFileArtifact { rel_path: wflow_rel, content: document_wflow, executable: false },
        ],
        root_entries_to_delete: vec![MACOS_WORKFLOW_BUNDLE_NAME.to_owned()],
    }
}

pub fn build_linux_xdg_manifest(config: &ShellMenuConfig) -> FileManifest {
    let exe = sanitize_executable_path(&config.executable_path);
    let expected_marker = format!("\"{exe}\" --pack-headless");

    let dolphin_desktop = format!(
        concat!(
            "[Desktop Entry]\n",
            "Type=Service\n",
            "MimeType=inode/directory;\n",
            "X-KDE-Submenu={root_label}\n",
            "Actions=PackHeadless;ApplyHeadless;OpenGui;\n",
            "X-KDE-Priority=TopLevel\n\n",
            "[Desktop Action PackHeadless]\n",
            "Name={pack_label}\n",
            "Icon=utilities-terminal\n",
            "Exec=\"{exe}\" --pack-headless %f\n\n",
            "[Desktop Action ApplyHeadless]\n",
            "Name={apply_label}\n",
            "Icon=utilities-terminal\n",
            "Exec=\"{exe}\" --apply-headless %f\n\n",
            "[Desktop Action OpenGui]\n",
            "Name={open_label}\n",
            "Icon=utilities-terminal\n",
            "Exec=\"{exe}\" --project %f\n"
        ),
        root_label = config.root_menu_label,
        pack_label = config.pack_menu_label,
        apply_label = config.apply_menu_label,
        open_label = config.open_gui_label,
        exe = exe
    );

    let nemo_action = format!(
        concat!(
            "[Nemo Action]\n",
            "Active=true\n",
            "Name={pack_label}\n",
            "Exec=\"{exe}\" --pack-headless %F\n",
            "Selection=s\n",
            "Extensions=dir;any;\n",
            "Quote=double\n"
        ),
        pack_label = config.pack_menu_label,
        exe = exe
    );

    let nautilus_script = format!("#!/usr/bin/env sh\n\"{exe}\" --pack-headless \"${{1:-.}}\"\n");

    let kio_rel = LINUX_PRIMARY_DESKTOP_REL_PATH.to_owned();
    let nemo_rel = "nemo/actions/codecontext.nemo_action".to_owned();
    let nautilus_rel = "nautilus/scripts/CodeContext Copy".to_owned();

    FileManifest {
        expected_marker,
        primary_rel_path: kio_rel.clone(),
        artifacts: vec![
            ShellFileArtifact {
                rel_path: kio_rel.clone(),
                content: dolphin_desktop,
                executable: false,
            },
            ShellFileArtifact {
                rel_path: nemo_rel.clone(),
                content: nemo_action,
                executable: false,
            },
            ShellFileArtifact {
                rel_path: nautilus_rel.clone(),
                content: nautilus_script,
                executable: true,
            },
        ],
        root_entries_to_delete: vec![kio_rel, nemo_rel, nautilus_rel],
    }
}

#[inline]
pub(crate) fn evaluate_command_status(
    actual_content: Option<&str>,
    expected_marker: &str,
) -> ShellRegistrationStatus {
    let Some(content) = actual_content else {
        return ShellRegistrationStatus::NotRegistered;
    };
    if content.contains(expected_marker) {
        ShellRegistrationStatus::RegisteredHealthy
    } else {
        ShellRegistrationStatus::StaleExecutablePath {
            registered_command: content.lines().next().unwrap_or(content).trim().to_owned(),
        }
    }
}

fn xml_escape(input: &str) -> Cow<'_, str> {
    if !input.contains(['&', '<', '>', '\x22', '\'']) {
        return Cow::Borrowed(input);
    }
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\x22' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    Cow::Owned(out)
}

#[derive(Debug, Default, Clone)]
pub(crate) struct InMemoryFileStore {
    files: BTreeMap<String, String>,
}

impl InMemoryFileStore {
    #[inline]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn apply_manifest(&mut self, manifest: &FileManifest) {
        for artifact in &manifest.artifacts {
            self.files.insert(artifact.rel_path.clone(), artifact.content.clone());
        }
    }

    pub(crate) fn remove_manifest(&mut self, manifest: &FileManifest) {
        for root_entry in &manifest.root_entries_to_delete {
            let prefix = format!("{root_entry}/");
            self.files.retain(|k, _| k != root_entry && !k.starts_with(&prefix));
        }
    }

    #[inline]
    pub(crate) fn get(&self, rel_path: &str) -> Option<&str> {
        self.files.get(rel_path).map(String::as_str)
    }
}
