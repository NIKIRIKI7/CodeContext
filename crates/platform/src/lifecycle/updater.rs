use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use core::system::ports::updater::{
    EMBEDDED_RELEASE_PUBLIC_KEY_HEX, ReleaseArtifact, ReleaseManifest, ReleaseProviderSpec,
    SelfInstallStatus, SemanticVersion, UpdateChannel, UpdateStage, UpdaterPort, UpdaterPortError,
    select_best_release,
};

const SHA256_K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

pub fn compute_sha256_hex(data: &[u8]) -> String {
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity((data.len() + 72) & !63);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.as_chunks::<64>().0 {
        compress_sha256_block(&mut state, chunk);
    }

    let mut out = String::with_capacity(64);
    for word in state {
        let _ = write!(out, "{word:08x}");
    }
    out
}

fn compress_sha256_block(state: &mut [u32; 8], chunk: &[u8; 64]) {
    let mut schedule = [0_u32; 64];
    for (idx, &word_bytes) in chunk.as_chunks::<4>().0.iter().enumerate() {
        if let Some(slot) = schedule.get_mut(idx) {
            *slot = u32::from_be_bytes(word_bytes);
        }
    }

    for idx in 16..64 {
        let w15 = schedule.get(idx - 15).copied().unwrap_or(0);
        let w2 = schedule.get(idx - 2).copied().unwrap_or(0);
        let w16 = schedule.get(idx - 16).copied().unwrap_or(0);
        let w7 = schedule.get(idx - 7).copied().unwrap_or(0);
        let s0 = w15.rotate_right(7) ^ w15.rotate_right(18) ^ (w15 >> 3);
        let s1 = w2.rotate_right(17) ^ w2.rotate_right(19) ^ (w2 >> 10);
        if let Some(slot) = schedule.get_mut(idx) {
            *slot = w16.wrapping_add(s0).wrapping_add(w7).wrapping_add(s1);
        }
    }

    let [mut v0, mut v1, mut v2, mut v3, mut v4, mut v5, mut v6, mut v7] = *state;
    for (k_val, w_val) in SHA256_K.iter().copied().zip(schedule.iter().copied()) {
        let s1 = v4.rotate_right(6) ^ v4.rotate_right(11) ^ v4.rotate_right(25);
        let ch = (v4 & v5) ^ ((!v4) & v6);
        let temp1 = v7.wrapping_add(s1).wrapping_add(ch).wrapping_add(k_val).wrapping_add(w_val);
        let s0 = v0.rotate_right(2) ^ v0.rotate_right(13) ^ v0.rotate_right(22);
        let maj = (v0 & v1) ^ (v0 & v2) ^ (v1 & v2);
        let temp2 = s0.wrapping_add(maj);

        v7 = v6;
        v6 = v5;
        v5 = v4;
        v4 = v3.wrapping_add(temp1);
        v3 = v2;
        v2 = v1;
        v1 = v0;
        v0 = temp1.wrapping_add(temp2);
    }

    let [s0, s1, s2, s3, s4, s5, s6, s7] = *state;
    *state = [
        s0.wrapping_add(v0),
        s1.wrapping_add(v1),
        s2.wrapping_add(v2),
        s3.wrapping_add(v3),
        s4.wrapping_add(v4),
        s5.wrapping_add(v5),
        s6.wrapping_add(v6),
        s7.wrapping_add(v7),
    ];
}

#[inline]
pub fn sign_release_payload(
    version: &SemanticVersion,
    target: &str,
    sha256_hex: &str,
    public_key_hex: &str,
) -> String {
    let canonical = format!(
        "codecontext-sig-v1|{version}|{target}|{}|{}",
        sha256_hex.trim().to_ascii_lowercase(),
        public_key_hex.trim().to_ascii_lowercase()
    );
    compute_sha256_hex(canonical.as_bytes())
}

#[inline]
pub fn verify_release_signature(
    artifact: &ReleaseArtifact,
    version: &SemanticVersion,
    public_key_hex: &str,
) -> bool {
    if artifact.signature.trim().is_empty() {
        return false;
    }
    let expected =
        sign_release_payload(version, &artifact.target, &artifact.sha256, public_key_hex);
    artifact.signature.trim().eq_ignore_ascii_case(&expected)
}

pub fn verify_payload_integrity(
    payload_bytes: &[u8],
    artifact: &ReleaseArtifact,
    version: &SemanticVersion,
    public_key_hex: &str,
) -> Result<(), UpdaterPortError> {
    if !verify_release_signature(artifact, version, public_key_hex) {
        return Err(UpdaterPortError::SignatureVerificationFailed(version.to_string()));
    }
    let actual_sha = compute_sha256_hex(payload_bytes);
    if !actual_sha.eq_ignore_ascii_case(artifact.sha256.trim()) {
        return Err(UpdaterPortError::ChecksumMismatch {
            expected: artifact.sha256.clone(),
            actual: actual_sha,
        });
    }
    Ok(())
}

pub fn current_target_triple() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-pc-windows-msvc"
        } else {
            "x86_64-pc-windows-msvc"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "x86_64") { "x86_64-apple-darwin" } else { "aarch64-apple-darwin" }
    } else if cfg!(target_os = "freebsd") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-unknown-freebsd"
        } else {
            "x86_64-unknown-freebsd"
        }
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        "x86_64-unknown-linux-gnu"
    }
}

pub fn canonical_install_binary_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
        base.join("Programs").join("CodeContext").join("codecontext.exe")
    } else if cfg!(target_os = "macos") {
        let base = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
        base.join("Applications").join("CodeContext.app").join("Contents/MacOS/codecontext")
    } else {
        let base = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
        base.join(".local").join("bin").join("codecontext")
    }
}

#[inline]
pub fn is_valid_native_executable_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 65_536 {
        return false;
    }
    if cfg!(target_os = "windows") {
        bytes.starts_with(b"MZ")
    } else if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        bytes.starts_with(b"\x7fELF")
    } else if cfg!(target_os = "macos") {
        matches!(
            bytes.get(..4),
            Some(
                [0xcf | 0xce, 0xfa, 0xed, 0xfe]
                    | [0xfe, 0xed, 0xfa, 0xcf | 0xce]
                    | [0xca, 0xfe, 0xba, 0xbe]
            )
        )
    } else {
        true
    }
}

pub fn repair_corrupted_canonical_binary_if_needed() {
    let canonical = canonical_install_binary_path();
    let Ok(meta) = fs::metadata(&canonical) else {
        return;
    };
    if !meta.is_file() {
        return;
    }
    let is_corrupted = if meta.len() < 65_536 {
        true
    } else if let Ok(existing_bytes) = fs::read(&canonical) {
        !is_valid_native_executable_bytes(&existing_bytes)
    } else {
        false
    };
    if !is_corrupted {
        return;
    }
    if let Some(valid_bytes) = find_recovery_binary_bytes(&canonical) {
        let _ = atomic_swap_executable(&canonical, &valid_bytes);
        let backup_old = canonical.with_extension("old");
        if backup_old.exists() {
            let _ = fs::remove_file(backup_old);
        }
    }
}

fn find_recovery_binary_bytes(canonical: &Path) -> Option<Vec<u8>> {
    let backup_old = canonical.with_extension("old");
    if let Ok(bytes) = fs::read(&backup_old)
        && is_valid_native_executable_bytes(&bytes)
    {
        return Some(bytes);
    }
    if let Ok(curr_exe) = std::env::current_exe()
        && curr_exe != canonical
        && curr_exe.file_stem().and_then(|s| s.to_str()).is_some_and(|stem| {
            stem.eq_ignore_ascii_case("app") || stem.eq_ignore_ascii_case("codecontext")
        })
        && let Ok(bytes) = fs::read(&curr_exe)
        && is_valid_native_executable_bytes(&bytes)
    {
        return Some(bytes);
    }
    let bin_name = if cfg!(windows) { "app.exe" } else { "app" };
    let mut roots = Vec::with_capacity(3);
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    if let Some(manifest_root) =
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent)
    {
        roots.push(manifest_root.to_path_buf());
    }
    for root in roots {
        for rel in [
            format!("target/release/{bin_name}"),
            format!("target/{}/release/{bin_name}", current_target_triple()),
            format!("target/debug/{bin_name}"),
        ] {
            let candidate = root.join(rel);
            if let Ok(bytes) = fs::read(&candidate)
                && is_valid_native_executable_bytes(&bytes)
            {
                return Some(bytes);
            }
        }
    }
    None
}

pub fn atomic_swap_executable(
    target_exe: &Path,
    new_binary_bytes: &[u8],
) -> Result<(), UpdaterPortError> {
    if let Some(parent) = target_exe.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
    }

    let pid = std::process::id();
    let staged_path = target_exe.with_extension(format!("{pid}.staged.tmp"));
    let backup_old_path = target_exe.with_extension("old");

    fs::write(&staged_path, new_binary_bytes)
        .map_err(|e| UpdaterPortError::Storage(e.to_string()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = fs::set_permissions(&staged_path, fs::Permissions::from_mode(0o700));
    }

    if backup_old_path.exists() {
        let _ = fs::remove_file(&backup_old_path);
    }

    let had_existing = target_exe.exists();
    if had_existing {
        fs::rename(target_exe, &backup_old_path)
            .map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
    }

    if let Err(err) = fs::rename(&staged_path, target_exe) {
        if had_existing {
            let _ = fs::rename(&backup_old_path, target_exe);
        }
        let _ = fs::remove_file(&staged_path);
        return Err(UpdaterPortError::Storage(err.to_string()));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdaterBackendMode {
    InMemory,
    NativeOs,
}

#[derive(Debug)]
struct UpdaterInnerState {
    current_version: SemanticVersion,
    current_exe: PathBuf,
    install_target_exe: PathBuf,
    stage: UpdateStage,
    simulated_releases: Vec<ReleaseManifest>,
    cached_remote_releases: Option<Vec<ReleaseManifest>>,
    staged_update: Option<(ReleaseManifest, Vec<u8>)>,
}

#[derive(Debug, Clone)]
pub struct SystemUpdater {
    mode: UpdaterBackendMode,
    state: Arc<Mutex<UpdaterInnerState>>,
}

impl Default for SystemUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemUpdater {
    #[inline]
    pub fn new() -> Self {
        repair_corrupted_canonical_binary_if_needed();
        let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("codecontext"));
        let old_artifact = current_exe.with_extension("old");
        if old_artifact.exists()
            && fs::read(&current_exe).is_ok_and(|b| is_valid_native_executable_bytes(&b))
        {
            let _ = fs::remove_file(old_artifact);
        }
        Self::with_paths(
            UpdaterBackendMode::NativeOs,
            SemanticVersion::parse(env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            current_exe,
            canonical_install_binary_path(),
        )
    }

    #[inline]
    pub fn new_in_memory() -> Self {
        repair_corrupted_canonical_binary_if_needed();
        let target = PathBuf::from("/virtual/in_memory/CodeContext/codecontext");
        Self::with_paths(
            UpdaterBackendMode::InMemory,
            SemanticVersion::new(0, 1, 0),
            target.clone(),
            target,
        )
    }

    #[inline]
    fn with_paths(
        is_in_memory: UpdaterBackendMode,
        current_version: SemanticVersion,
        current_exe: PathBuf,
        install_target_exe: PathBuf,
    ) -> Self {
        Self {
            mode: is_in_memory,
            state: Arc::new(Mutex::new(UpdaterInnerState {
                current_version,
                current_exe,
                install_target_exe,
                stage: UpdateStage::Idle,
                simulated_releases: Vec::new(),
                cached_remote_releases: None,
                staged_update: None,
            })),
        }
    }

    pub fn set_simulated_remote_releases(&self, releases: Vec<ReleaseManifest>) {
        if let Ok(mut guard) = self.state.lock() {
            guard.simulated_releases = releases;
        }
    }

    pub fn set_simulated_staged_payload(&self, manifest: ReleaseManifest, payload: Vec<u8>) {
        if let Ok(mut guard) = self.state.lock() {
            guard.stage = UpdateStage::UpdateAvailable {
                version: manifest.version,
                channel: manifest.channel,
            };
            guard.staged_update = Some((manifest, payload));
        }
    }

    pub fn set_simulated_portable_paths(&self, current_exe: PathBuf, target_exe: PathBuf) {
        if let Ok(mut guard) = self.state.lock() {
            guard.current_exe = current_exe;
            guard.install_target_exe = target_exe;
        }
    }

    pub fn restart_into_updated_binary(&self) -> Result<(), UpdaterPortError> {
        let current_exe = self
            .state
            .lock()
            .map_err(|e| UpdaterPortError::Storage(e.to_string()))?
            .current_exe
            .clone();
        if self.mode == UpdaterBackendMode::NativeOs && current_exe.exists() {
            Command::new(&current_exe)
                .spawn()
                .map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}

impl UpdaterPort for SystemUpdater {
    fn current_version(&self) -> SemanticVersion {
        self.state.lock().map(|g| g.current_version).unwrap_or_default()
    }

    fn self_install_status(&self) -> SelfInstallStatus {
        let Ok(guard) = self.state.lock() else {
            return SelfInstallStatus::InstalledCanonical {
                install_path: canonical_install_binary_path(),
            };
        };
        if guard.current_exe == guard.install_target_exe {
            SelfInstallStatus::InstalledCanonical { install_path: guard.install_target_exe.clone() }
        } else {
            SelfInstallStatus::PortableMode {
                current_path: guard.current_exe.clone(),
                target_install_path: guard.install_target_exe.clone(),
            }
        }
    }

    fn stage(&self) -> UpdateStage {
        self.state.lock().map(|g| g.stage.clone()).unwrap_or_default()
    }

    fn switch_channel_cached(&self, channel: UpdateChannel) -> UpdateStage {
        let Ok(mut guard) = self.state.lock() else {
            return UpdateStage::Idle;
        };
        let cached_opt = if self.mode == UpdaterBackendMode::InMemory {
            (!guard.simulated_releases.is_empty()).then(|| guard.simulated_releases.clone())
        } else {
            guard.cached_remote_releases.clone()
        };
        let Some(releases) = cached_opt else {
            guard.staged_update = None;
            guard.stage = UpdateStage::Idle;
            return UpdateStage::Idle;
        };

        apply_selected_release_to_state(self.mode, &mut guard, &releases, channel)
    }

    fn check_for_updates(
        &self,
        channel: UpdateChannel,
        repo_url: &str,
    ) -> Result<UpdateStage, UpdaterPortError> {
        let provider = ReleaseProviderSpec::from_repo_url(repo_url);
        let releases = if self.mode == UpdaterBackendMode::InMemory {
            self.state
                .lock()
                .map_err(|e| UpdaterPortError::Storage(e.to_string()))?
                .simulated_releases
                .clone()
        } else {
            match fetch_remote_releases(&provider) {
                Ok(list) => list,
                Err(err) => {
                    if let Ok(mut guard) = self.state.lock() {
                        guard.stage = UpdateStage::Failed { reason: err.to_string() };
                    }
                    return Err(err);
                }
            }
        };

        let mut guard = self.state.lock().map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
        if self.mode == UpdaterBackendMode::NativeOs {
            guard.cached_remote_releases = Some(releases.clone());
        }

        Ok(apply_selected_release_to_state(self.mode, &mut guard, &releases, channel))
    }

    fn apply_staged_update(&self) -> Result<UpdateStage, UpdaterPortError> {
        let (manifest, mut payload, current_exe, install_target_exe) = {
            let guard = self.state.lock().map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
            let Some((manifest, payload)) = guard.staged_update.clone() else {
                return Ok(guard.stage.clone());
            };
            (manifest, payload, guard.current_exe.clone(), guard.install_target_exe.clone())
        };

        let target = current_target_triple();
        if self.mode == UpdaterBackendMode::NativeOs && payload.is_empty() {
            match download_native_release_binary(&manifest.version, target) {
                Ok(downloaded) => payload = downloaded,
                Err(err) => {
                    if let Ok(mut guard) = self.state.lock() {
                        guard.stage = UpdateStage::Failed { reason: err.to_string() };
                    }
                    return Err(err);
                }
            }
        } else if let Some(artifact) = manifest.find_artifact_for_target(target)
            && let Err(err) = verify_payload_integrity(
                &payload,
                artifact,
                &manifest.version,
                EMBEDDED_RELEASE_PUBLIC_KEY_HEX,
            )
        {
            if let Ok(mut guard) = self.state.lock() {
                guard.stage = UpdateStage::Failed { reason: err.to_string() };
            }
            return Err(err);
        }

        if self.mode == UpdaterBackendMode::NativeOs {
            if !is_valid_native_executable_bytes(&payload) {
                let err = UpdaterPortError::InvalidManifest(
                    "Скачанный бинарный файл не прошёл проверку заголовка исполняемого файла ОС"
                        .to_owned(),
                );
                if let Ok(mut guard) = self.state.lock() {
                    guard.stage = UpdateStage::Failed { reason: err.to_string() };
                }
                return Err(err);
            }
            if let Err(err) = atomic_swap_executable(&current_exe, &payload) {
                if let Ok(mut guard) = self.state.lock() {
                    guard.stage = UpdateStage::Failed { reason: err.to_string() };
                }
                return Err(err);
            }
            if install_target_exe != current_exe && install_target_exe.exists() {
                let _ = atomic_swap_executable(&install_target_exe, &payload);
            }
        }

        let mut guard = self.state.lock().map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
        let stage = UpdateStage::ReadyToRestart { version: manifest.version };
        guard.current_version = manifest.version;
        guard.stage = stage.clone();
        Ok(stage)
    }

    fn perform_self_install(&self) -> Result<SelfInstallStatus, UpdaterPortError> {
        let mut guard = self.state.lock().map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
        let target_exe = guard.install_target_exe.clone();
        if guard.current_exe != target_exe {
            if guard.current_exe.exists() {
                let bytes = fs::read(&guard.current_exe)
                    .map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
                atomic_swap_executable(&target_exe, &bytes)?;
            }
            guard.current_exe.clone_from(&target_exe);
        }
        Ok(SelfInstallStatus::InstalledCanonical { install_path: target_exe })
    }
}

fn apply_selected_release_to_state(
    mode: UpdaterBackendMode,
    guard: &mut UpdaterInnerState,
    releases: &[ReleaseManifest],
    channel: UpdateChannel,
) -> UpdateStage {
    let current = guard.current_version;
    let next_stage = if let Some(best) = select_best_release(releases, &current, channel) {
        let staged_payload = if mode == UpdaterBackendMode::InMemory {
            format!("codecontext-binary-{}", best.version).into_bytes()
        } else {
            Vec::new()
        };
        guard.staged_update = Some((best.clone(), staged_payload));
        UpdateStage::UpdateAvailable { version: best.version, channel: best.channel }
    } else {
        guard.staged_update = None;
        UpdateStage::UpToDate { version: current }
    };
    guard.stage = next_stage.clone();
    next_stage
}

fn fetch_remote_releases(
    provider: &ReleaseProviderSpec,
) -> Result<Vec<ReleaseManifest>, UpdaterPortError> {
    let api_res = curl_fetch_text(&provider.api_releases_url, 6);
    if let Ok(ref body) = api_res {
        let parsed = parse_forge_releases_tags(body);
        if !parsed.is_empty() {
            return Ok(parsed);
        }
    }

    let manifest_res = curl_fetch_text(&provider.manifest_url, 6);
    if let Ok(ref body) = manifest_res {
        let parsed = parse_forge_releases_tags(body);
        if !parsed.is_empty() {
            return Ok(parsed);
        }
    }

    match (api_res, manifest_res) {
        (Err(e), _) | (_, Err(e)) => Err(e),
        _ => Ok(Vec::new()),
    }
}

fn curl_fetch_text(url: &str, max_time_secs: u32) -> Result<String, UpdaterPortError> {
    let max_time = max_time_secs.to_string();
    let mut cmd = Command::new("curl");
    cmd.args([
        "-fsSL",
        "--connect-timeout",
        "4",
        "--max-time",
        &max_time,
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "User-Agent: CodeContext-Updater",
        url,
    ]);
    configure_hidden_command(&mut cmd);
    let output = cmd.output().map_err(|e| UpdaterPortError::Network(e.to_string()))?;
    if !output.status.success() {
        return Err(UpdaterPortError::Network(format!("HTTP error fetching `{url}`")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn curl_download_file(url: &str, dest: &Path) -> Result<(), UpdaterPortError> {
    let dest_str = dest.to_string_lossy();
    let mut cmd = Command::new("curl");
    cmd.args([
        "-fsSL",
        "--connect-timeout",
        "8",
        "--max-time",
        "120",
        "-H",
        "User-Agent: CodeContext-Updater",
        "-o",
        &dest_str,
        url,
    ]);
    configure_hidden_command(&mut cmd);
    let status = cmd.status().map_err(|e| UpdaterPortError::Network(e.to_string()))?;
    if !status.success() || !dest.exists() {
        return Err(UpdaterPortError::Network(format!("Не удалось скачать `{url}`")));
    }
    Ok(())
}

fn download_native_release_binary(
    version: &SemanticVersion,
    target: &str,
) -> Result<Vec<u8>, UpdaterPortError> {
    let spec = ReleaseProviderSpec::default();
    let base_url =
        format!("https://github.com/{}/{}/releases/download/v{version}", spec.owner, spec.repo);
    let pid = std::process::id();
    let stage_dir = std::env::temp_dir().join(format!("codecontext_upd_{version}_{pid}"));
    let _ = fs::remove_dir_all(&stage_dir);
    fs::create_dir_all(&stage_dir).map_err(|e| UpdaterPortError::Storage(e.to_string()))?;

    let result = download_and_extract_for_target(&base_url, target, &stage_dir);
    let _ = fs::remove_dir_all(&stage_dir);
    let bytes = result?;
    if !is_valid_native_executable_bytes(&bytes) {
        return Err(UpdaterPortError::InvalidManifest(
            "Скачанный бинарный файл повреждён, слишком мал или не имеет заголовка ОС".to_owned(),
        ));
    }
    Ok(bytes)
}

fn download_and_extract_for_target(
    base_url: &str,
    target: &str,
    stage_dir: &Path,
) -> Result<Vec<u8>, UpdaterPortError> {
    let sha_sums_opt = curl_fetch_text(&format!("{base_url}/SHA256SUMS.txt"), 8).ok();

    if cfg!(target_os = "windows") {
        let exe_name = format!("codecontext-{target}.exe");
        let exe_path = stage_dir.join(&exe_name);
        if curl_download_file(&format!("{base_url}/{exe_name}"), &exe_path).is_ok()
            && let Ok(bytes) = fs::read(&exe_path)
        {
            verify_optional_sha256_sums(&bytes, &exe_name, sha_sums_opt.as_deref())?;
            return Ok(bytes);
        }

        let zip_name = format!("codecontext-{target}.zip");
        let zip_path = stage_dir.join(&zip_name);
        curl_download_file(&format!("{base_url}/{zip_name}"), &zip_path)?;
        if let Ok(zip_bytes) = fs::read(&zip_path) {
            verify_optional_sha256_sums(&zip_bytes, &zip_name, sha_sums_opt.as_deref())?;
        }
        extract_archive_with_system_tar(&zip_path, stage_dir)?;
        return fs::read(stage_dir.join("codecontext.exe"))
            .map_err(|e| UpdaterPortError::Storage(e.to_string()));
    }

    let tar_name = format!("codecontext-{target}.tar.gz");
    let tar_path = stage_dir.join(&tar_name);
    if curl_download_file(&format!("{base_url}/{tar_name}"), &tar_path).is_ok() {
        if let Ok(tar_bytes) = fs::read(&tar_path) {
            verify_optional_sha256_sums(&tar_bytes, &tar_name, sha_sums_opt.as_deref())?;
        }
        extract_archive_with_system_tar(&tar_path, stage_dir)?;
        return fs::read(stage_dir.join("codecontext"))
            .map_err(|e| UpdaterPortError::Storage(e.to_string()));
    }

    let appimage_name = format!("codecontext-{target}.AppImage");
    let appimage_path = stage_dir.join(&appimage_name);
    curl_download_file(&format!("{base_url}/{appimage_name}"), &appimage_path)?;
    let bytes = fs::read(&appimage_path).map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
    verify_optional_sha256_sums(&bytes, &appimage_name, sha_sums_opt.as_deref())?;
    Ok(bytes)
}

fn verify_optional_sha256_sums(
    bytes: &[u8],
    asset_name: &str,
    sha_sums_text: Option<&str>,
) -> Result<(), UpdaterPortError> {
    let Some(sums) = sha_sums_text else {
        return Ok(());
    };
    for line in sums.lines() {
        let mut parts = line.split_whitespace();
        let Some(expected_hex) = parts.next() else {
            continue;
        };
        let Some(file_col) = parts.next() else {
            continue;
        };
        let clean_file = file_col.trim_start_matches('*');
        if clean_file == asset_name {
            let actual_hex = compute_sha256_hex(bytes);
            if !actual_hex.eq_ignore_ascii_case(expected_hex) {
                return Err(UpdaterPortError::ChecksumMismatch {
                    expected: expected_hex.to_owned(),
                    actual: actual_hex,
                });
            }
            return Ok(());
        }
    }
    Ok(())
}

fn extract_archive_with_system_tar(
    archive_path: &Path,
    dest_dir: &Path,
) -> Result<(), UpdaterPortError> {
    let mut cmd = Command::new("tar");
    cmd.arg("-xf").arg(archive_path).arg("-C").arg(dest_dir);
    configure_hidden_command(&mut cmd);
    let status = cmd.status().map_err(|e| UpdaterPortError::Storage(e.to_string()))?;
    if !status.success() {
        return Err(UpdaterPortError::Storage("Ошибка распаковки архива обновления".to_owned()));
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_ref_mut)]
fn configure_hidden_command(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

pub fn parse_forge_releases_tags(raw_json: &str) -> Vec<ReleaseManifest> {
    let mut manifests = extract_manifests_by_json_key(raw_json, "\"tag_name\"");
    if manifests.is_empty() {
        manifests = extract_manifests_by_json_key(raw_json, "\"version\"");
    }
    manifests
}

fn extract_manifests_by_json_key(raw_json: &str, marker: &str) -> Vec<ReleaseManifest> {
    let mut manifests: Vec<ReleaseManifest> = Vec::new();
    let mut cursor = 0;
    while let Some(rel) = raw_json.get(cursor..).and_then(|s| s.find(marker)) {
        let after_key = cursor + rel + marker.len();
        let Some(tail) = raw_json.get(after_key..) else {
            break;
        };
        if let Some(colon) = tail.find(':')
            && let Some(after_colon) = tail.get(colon + 1..)
            && let Some(first_q) = after_colon.find('"')
            && let Some(inside) = after_colon.get(first_q + 1..)
            && let Some(end_q) = inside.find('"')
            && let Some(tag_str) = inside.get(..end_q)
            && let Ok(ver) = SemanticVersion::parse(tag_str)
            && !manifests.iter().any(|m| m.version == ver)
        {
            manifests.push(ReleaseManifest {
                version: ver,
                channel: ver.channel(),
                published_at: String::new(),
                changelog: String::new(),
                artifacts: Vec::new(),
            });
        }
        cursor = after_key;
    }
    manifests
}
