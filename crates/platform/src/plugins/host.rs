use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use core::system::plugins::manifest::{
    ActivationEvent, ContributionPoints, PluginCodeTransformSpec, PluginCommandSpec,
    PluginContextProviderSpec, PluginManifest, PluginPermission, PluginViewContribution,
    PluginViewLocation,
};
use core::system::plugins::protocol::{
    PluginDescriptor, PluginHostPort, PluginPortError, PluginUiEvent, PluginViewSpec,
    PluginWidgetNode,
};

#[derive(Debug, Default)]
struct PluginRegistryState {
    plugins: Vec<PluginDescriptor>,
    plugins_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SystemPluginHost {
    state: Arc<Mutex<PluginRegistryState>>,
}

impl Default for SystemPluginHost {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPluginHost {
    #[inline]
    pub fn new() -> Self {
        let base_dir = dirs::config_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("CodeContext")
            .join("plugins");
        let host = Self::with_plugins_dir(base_dir);
        host.discover_external_plugins();
        host
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        Self::with_plugins_dir(PathBuf::from("/virtual/in_memory/plugins"))
    }

    pub fn with_plugins_dir(plugins_dir: PathBuf) -> Self {
        let built_in = built_in_default_plugins();
        Self { state: Arc::new(Mutex::new(PluginRegistryState { plugins: built_in, plugins_dir })) }
    }

    #[inline]
    pub fn plugins_dir(&self) -> PathBuf {
        self.state.lock().map_or_else(|_| PathBuf::from("."), |g| g.plugins_dir.clone())
    }

    pub fn register_custom_plugin(&self, manifest: PluginManifest, enabled: bool) {
        if let Ok(mut guard) = self.state.lock() {
            if let Some(existing) = guard.plugins.iter_mut().find(|p| p.manifest.id == manifest.id)
            {
                existing.manifest = manifest;
                existing.enabled = enabled;
            } else {
                guard.plugins.push(PluginDescriptor { manifest, enabled, is_built_in: false });
            }
        }
    }

    pub fn discover_external_plugins(&self) {
        let dir = self.plugins_dir();
        let Ok(entries) = fs::read_dir(&dir) else {
            return;
        };
        for entry in entries.flatten() {
            let manifest_path = entry.path().join("plugin.json");
            if manifest_path.is_file()
                && let Ok(raw) = fs::read_to_string(&manifest_path)
                && let Ok(manifest) = parse_simple_plugin_manifest(&raw)
            {
                self.register_custom_plugin(manifest, true);
            }
        }
    }
}

impl PluginHostPort for SystemPluginHost {
    fn list_plugins(&self) -> Vec<PluginDescriptor> {
        self.state.lock().map(|g| g.plugins.clone()).unwrap_or_default()
    }

    fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> Result<(), PluginPortError> {
        let mut guard = self.state.lock().map_err(|e| PluginPortError::ExecutionFailed {
            plugin_id: plugin_id.to_owned(),
            reason: e.to_string(),
        })?;
        let Some(desc) = guard.plugins.iter_mut().find(|p| p.manifest.id == plugin_id) else {
            return Err(PluginPortError::NotFoundOrDisabled(plugin_id.to_owned()));
        };
        desc.enabled = enabled;
        Ok(())
    }

    fn collect_dynamic_context_sections(
        &self,
        project_root: Option<&Path>,
    ) -> Vec<(String, String)> {
        let plugins = self.list_plugins();
        let mut sections = Vec::new();
        let Some(root) = project_root else {
            return sections;
        };

        for desc in plugins.into_iter().filter(|p| p.enabled) {
            if desc.manifest.id == "builtin.git_context"
                && let Some(git_summary) = collect_git_branch_context(root)
            {
                sections.push(("GIT BRANCH CONTEXT".to_owned(), git_summary));
            } else if let Some(ref entrypoint) = desc.manifest.entrypoint
                && desc.manifest.has_permission(PluginPermission::FsReadProject)
                && let Ok(out) = invoke_external_json_rpc_hook(
                    entrypoint,
                    "onPackContext",
                    &root.to_string_lossy(),
                )
                && !out.trim().is_empty()
            {
                sections.push((desc.manifest.name, out));
            }
        }
        sections
    }

    fn transform_file_content(&self, rel_path: &str, original_content: &str) -> Option<String> {
        let plugins = self.list_plugins();
        let is_rust_ext =
            Path::new(rel_path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("rs"));
        for desc in plugins.into_iter().filter(|p| p.enabled) {
            if desc.manifest.id == "builtin.ast_signatures" && is_rust_ext {
                return Some(extract_rust_public_signatures(original_content));
            }
        }
        None
    }

    fn execute_command(
        &self,
        command_id: &str,
        project_root: Option<&Path>,
    ) -> Result<String, PluginPortError> {
        let plugins = self.list_plugins();
        for desc in plugins.into_iter().filter(|p| p.enabled) {
            let has_cmd = desc.manifest.contributes.commands.iter().any(|c| c.id == command_id);
            if !has_cmd {
                continue;
            }
            if desc.manifest.id == "builtin.git_context" {
                let summary = project_root
                    .and_then(collect_git_branch_context)
                    .unwrap_or_else(|| "Git repository not detected".to_owned());
                return Ok(summary);
            }
            if desc.manifest.id == "builtin.dep_explorer" {
                return Ok("Dependency Explorer ready".to_owned());
            }
            if let Some(ref entry) = desc.manifest.entrypoint {
                let root_str = project_root.map_or(".", |p| p.to_str().unwrap_or("."));
                return invoke_external_json_rpc_hook(entry, command_id, root_str).map_err(
                    |reason| PluginPortError::ExecutionFailed {
                        plugin_id: desc.manifest.id,
                        reason,
                    },
                );
            }
        }
        Err(PluginPortError::NotFoundOrDisabled(command_id.to_owned()))
    }

    fn render_plugin_view(
        &self,
        plugin_id: &str,
        view_id: &str,
        project_root: Option<&Path>,
    ) -> Result<PluginViewSpec, PluginPortError> {
        let plugins = self.list_plugins();
        let Some(desc) = plugins.into_iter().find(|p| p.manifest.id == plugin_id && p.enabled)
        else {
            return Err(PluginPortError::NotFoundOrDisabled(plugin_id.to_owned()));
        };

        let summary = project_root
            .and_then(collect_git_branch_context)
            .unwrap_or_else(|| "Open a Git repository to inspect branch context.".to_owned());

        Ok(PluginViewSpec {
            plugin_id: desc.manifest.id,
            view_id: view_id.to_owned(),
            title: desc.manifest.name,
            root: PluginWidgetNode::Section {
                title: "Plugin View".to_owned(),
                children: vec![
                    PluginWidgetNode::Label {
                        text: desc.manifest.description,
                        monospace: false,
                        muted: true,
                    },
                    PluginWidgetNode::CodePreview {
                        id: "plugin-preview-box".to_owned(),
                        content: summary,
                    },
                ],
            },
        })
    }

    fn dispatch_ui_event(
        &self,
        event: &PluginUiEvent,
        project_root: Option<&Path>,
    ) -> Result<PluginViewSpec, PluginPortError> {
        self.render_plugin_view(&event.plugin_id, &event.view_id, project_root)
    }
}

fn built_in_default_plugins() -> Vec<PluginDescriptor> {
    vec![
        PluginDescriptor {
            manifest: PluginManifest {
                id: "builtin.git_context".to_owned(),
                name: "Git Branch & Diff Context (@git)".to_owned(),
                version: "1.0.0".to_owned(),
                description: "Appends current Git branch status and recent commits to packed context"
                    .to_owned(),
                author: "CodeContext Core".to_owned(),
                entrypoint: None,
                activation_events: vec![
                    ActivationEvent::OnPackContext,
                    ActivationEvent::OnCommand("git.inject_branch_summary".to_owned()),
                ],
                permissions: vec![
                    PluginPermission::WorkspaceSelection,
                    PluginPermission::FsReadProject,
                ],
                contributes: ContributionPoints {
                    commands: vec![PluginCommandSpec {
                        id: "git.inject_branch_summary".to_owned(),
                        title: "Plugin: Copy Git Branch Summary".to_owned(),
                        default_shortcut: None,
                        category: "Git".to_owned(),
                    }],
                    views: vec![PluginViewContribution {
                        id: "git_context_panel".to_owned(),
                        title: "Git Context".to_owned(),
                        icon: "git".to_owned(),
                        location: PluginViewLocation::BottomDock,
                    }],
                    context_providers: vec![PluginContextProviderSpec {
                        id: "git_branch_status".to_owned(),
                        title: "Git Branch & Last 5 Commits".to_owned(),
                        header_tag: "GIT_CONTEXT".to_owned(),
                        enabled_by_default: false,
                    }],
                    code_transforms: Vec::new(),
                },
            },
            enabled: false,
            is_built_in: true,
        },
        PluginDescriptor {
            manifest: PluginManifest {
                id: "builtin.ast_signatures".to_owned(),
                name: "Rust API Signatures Skeletonizer".to_owned(),
                version: "1.0.0".to_owned(),
                description:
                    "Compresses Rust source files to public struct/enum/fn signatures (5-8x token savings)"
                        .to_owned(),
                author: "CodeContext Core".to_owned(),
                entrypoint: None,
                activation_events: vec![ActivationEvent::OnPackContext],
                permissions: vec![PluginPermission::FsReadProject],
                contributes: ContributionPoints {
                    commands: vec![PluginCommandSpec {
                        id: "ast.toggle_skeletonizer".to_owned(),
                        title: "Plugin: Toggle Rust Signatures Skeletonizer".to_owned(),
                        default_shortcut: None,
                        category: "Packer".to_owned(),
                    }],
                    views: Vec::new(),
                    context_providers: Vec::new(),
                    code_transforms: vec![PluginCodeTransformSpec {
                        id: "rust_skeleton".to_owned(),
                        title: "Public Signatures Only (.rs)".to_owned(),
                        target_extensions: vec!["rs".to_owned()],
                    }],
                },
            },
            enabled: false,
            is_built_in: true,
        },
    ]
}

fn collect_git_branch_context(project_root: &Path) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(["--no-optional-locks", "status", "-sb"]).current_dir(project_root);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let status_txt = String::from_utf8_lossy(&out.stdout);
    Some(status_txt.trim().to_owned())
}

pub fn extract_rust_public_signatures(source: &str) -> String {
    let mut out = String::with_capacity(source.len() / 2);
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub ")
            || trimmed.starts_with("use ")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("impl")
            || trimmed.starts_with("#[derive")
            || trimmed.starts_with("trait ")
        {
            if let Some((sig_head, _)) = line.split_once('{') {
                out.push_str(sig_head.trim_end());
                out.push_str(" { /* ... */ }\n");
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    if out.is_empty() { source.to_owned() } else { out }
}

fn invoke_external_json_rpc_hook(
    entrypoint: &str,
    method: &str,
    project_root: &str,
) -> Result<String, String> {
    let mut child = Command::new(entrypoint)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        let req = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{method}\",\"params\":{{\"project_root\":\"{}\"}}}}\n",
            project_root.replace('\\', "\\\\").replace('"', "\\\"")
        );
        let _ = stdin.write_all(req.as_bytes());
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("Exit status: {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn parse_simple_plugin_manifest(raw: &str) -> Result<PluginManifest, String> {
    let id = extract_json_string_field(raw, "\"id\"").ok_or("Missing id")?;
    let name = extract_json_string_field(raw, "\"name\"").unwrap_or_else(|| id.clone());
    let version =
        extract_json_string_field(raw, "\"version\"").unwrap_or_else(|| "0.1.0".to_owned());
    let description = extract_json_string_field(raw, "\"description\"").unwrap_or_default();
    let author = extract_json_string_field(raw, "\"author\"").unwrap_or_default();
    let entrypoint = extract_json_string_field(raw, "\"entrypoint\"");
    Ok(PluginManifest {
        id,
        name,
        version,
        description,
        author,
        entrypoint,
        activation_events: vec![ActivationEvent::OnStartup],
        permissions: vec![PluginPermission::WorkspaceSelection],
        contributes: ContributionPoints::default(),
    })
}

fn extract_json_string_field(raw: &str, key: &str) -> Option<String> {
    let pos = raw.find(key)?;
    let tail = raw.get(pos + key.len()..)?;
    let colon = tail.find(':')?;
    let after_colon = tail.get(colon + 1..)?.trim_start();
    let inner = after_colon.strip_prefix('"')?;
    let end_q = inner.find('"')?;
    inner.get(..end_q).map(str::to_owned)
}
