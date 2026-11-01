use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_REPOSITORY_URL: &str = "https://github.com/NIKIRIKI7/CodeContext.git";
pub const EMBEDDED_RELEASE_PUBLIC_KEY_HEX: &str =
    "c0dec0de7e57ed2551970b11c8e9a4f2d1b3c5e7a9f0123456789abcdef01234";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
    Alpha,
}

impl UpdateChannel {
    #[inline]
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.eq_ignore_ascii_case("alpha") {
            Self::Alpha
        } else if trimmed.eq_ignore_ascii_case("beta") {
            Self::Beta
        } else {
            Self::Stable
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Alpha => "alpha",
        }
    }

    #[inline]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Beta => "Beta",
            Self::Alpha => "Alpha",
        }
    }

    #[must_use]
    #[inline]
    pub const fn next_in_cycle(self) -> Self {
        match self {
            Self::Stable => Self::Beta,
            Self::Beta => Self::Alpha,
            Self::Alpha => Self::Stable,
        }
    }

    #[inline]
    pub const fn accepts_version(self, version: &SemanticVersion) -> bool {
        match self {
            Self::Stable => matches!(version.pre, PreReleaseTag::None),
            Self::Beta => matches!(version.pre, PreReleaseTag::None | PreReleaseTag::Beta(_)),
            Self::Alpha => true,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum PreReleaseTag {
    Alpha(u32),
    Beta(u32),
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: PreReleaseTag,
}

impl Default for SemanticVersion {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        match self.pre {
            PreReleaseTag::None => Ok(()),
            PreReleaseTag::Beta(n) => write!(f, "-beta.{n}"),
            PreReleaseTag::Alpha(n) => write!(f, "-alpha.{n}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VersionParseError {
    #[error("Пустая строка версии")]
    Empty,
    #[error("Некорректный формат семантической версии: `{0}`")]
    InvalidFormat(String),
}

impl SemanticVersion {
    #[inline]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch, pre: PreReleaseTag::None }
    }

    #[inline]
    pub const fn with_pre(major: u32, minor: u32, patch: u32, pre: PreReleaseTag) -> Self {
        Self { major, minor, patch, pre }
    }

    #[inline]
    pub const fn channel(&self) -> UpdateChannel {
        match self.pre {
            PreReleaseTag::None => UpdateChannel::Stable,
            PreReleaseTag::Beta(_) => UpdateChannel::Beta,
            PreReleaseTag::Alpha(_) => UpdateChannel::Alpha,
        }
    }

    pub fn parse(raw: &str) -> Result<Self, VersionParseError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(VersionParseError::Empty);
        }
        let without_v =
            trimmed.strip_prefix('v').or_else(|| trimmed.strip_prefix('V')).unwrap_or(trimmed);
        let without_build = without_v.split('+').next().unwrap_or(without_v);
        let (core_part, pre_opt) = match without_build.split_once('-') {
            Some((c, p)) => (c, Some(p)),
            None => (without_build, None),
        };

        let mut parts = core_part.split('.');
        let major = parts
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| VersionParseError::InvalidFormat(raw.to_owned()))?;
        let minor = parts
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| VersionParseError::InvalidFormat(raw.to_owned()))?;
        let patch = parts
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| VersionParseError::InvalidFormat(raw.to_owned()))?;
        if parts.next().is_some() {
            return Err(VersionParseError::InvalidFormat(raw.to_owned()));
        }

        let pre = match pre_opt {
            None => PreReleaseTag::None,
            Some(pre_str) => parse_prerelease_tag(pre_str)
                .ok_or_else(|| VersionParseError::InvalidFormat(raw.to_owned()))?,
        };

        Ok(Self { major, minor, patch, pre })
    }
}

fn parse_prerelease_tag(pre_str: &str) -> Option<PreReleaseTag> {
    let clean = pre_str.trim();
    if let Some(rest) = clean
        .strip_prefix("alpha")
        .or_else(|| clean.strip_prefix("Alpha"))
        .or_else(|| clean.strip_prefix("ALPHA"))
    {
        let num_str = rest.trim_start_matches(['.', '-', '_']);
        let num = if num_str.is_empty() { 1 } else { num_str.parse::<u32>().ok()? };
        return Some(PreReleaseTag::Alpha(num));
    }
    if let Some(rest) = clean
        .strip_prefix("beta")
        .or_else(|| clean.strip_prefix("Beta"))
        .or_else(|| clean.strip_prefix("BETA"))
        .or_else(|| clean.strip_prefix("rc"))
        .or_else(|| clean.strip_prefix("RC"))
    {
        let num_str = rest.trim_start_matches(['.', '-', '_']);
        let num = if num_str.is_empty() { 1 } else { num_str.parse::<u32>().ok()? };
        return Some(PreReleaseTag::Beta(num));
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitHostKind {
    GitHub,
    GitLab,
    Gitea,
    CustomManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseProviderSpec {
    pub host_kind: GitHostKind,
    pub repository_url: String,
    pub owner: String,
    pub repo: String,
    pub manifest_url: String,
    pub api_releases_url: String,
}

impl Default for ReleaseProviderSpec {
    fn default() -> Self {
        Self::from_repo_url(DEFAULT_REPOSITORY_URL)
    }
}

impl ReleaseProviderSpec {
    pub fn from_repo_url(raw_url: &str) -> Self {
        let trimmed = raw_url.trim();
        let url = if trimmed.is_empty() { DEFAULT_REPOSITORY_URL } else { trimmed };
        let without_git = url.strip_suffix(".git").unwrap_or(url).trim_end_matches('/');

        if std::path::Path::new(without_git)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            return Self {
                host_kind: GitHostKind::CustomManifest,
                repository_url: url.to_owned(),
                owner: String::new(),
                repo: String::new(),
                manifest_url: without_git.to_owned(),
                api_releases_url: without_git.to_owned(),
            };
        }

        let (host, owner, repo) = extract_host_owner_repo(without_git);
        let host_kind = if host.contains("github.com") {
            GitHostKind::GitHub
        } else if host.contains("gitlab.") {
            GitHostKind::GitLab
        } else {
            GitHostKind::Gitea
        };

        let (manifest_url, api_releases_url) = match host_kind {
            GitHostKind::GitHub => (
                format!(
                    "https://github.com/{owner}/{repo}/releases/latest/download/releases-index.json"
                ),
                format!("https://api.github.com/repos/{owner}/{repo}/releases"),
            ),
            GitHostKind::GitLab => (
                format!(
                    "https://{host}/{owner}/{repo}/-/releases/permalink/latest/downloads/releases-index.json"
                ),
                format!("https://{host}/api/v4/projects/{owner}%2F{repo}/releases"),
            ),
            GitHostKind::Gitea | GitHostKind::CustomManifest => (
                format!(
                    "https://{host}/{owner}/{repo}/releases/download/latest/releases-index.json"
                ),
                format!("https://{host}/api/v1/repos/{owner}/{repo}/releases"),
            ),
        };

        Self {
            host_kind,
            repository_url: url.to_owned(),
            owner,
            repo,
            manifest_url,
            api_releases_url,
        }
    }
}

fn extract_host_owner_repo(clean_url: &str) -> (String, String, String) {
    let after_scheme = clean_url
        .strip_prefix("https://")
        .or_else(|| clean_url.strip_prefix("http://"))
        .or_else(|| clean_url.strip_prefix("git@"))
        .unwrap_or(clean_url);
    let normalized = after_scheme.replace(':', "/");
    let mut segments = normalized.split('/').filter(|s| !s.is_empty());
    let host = segments.next().unwrap_or("github.com").to_owned();
    let owner = segments.next().unwrap_or("NIKIRIKI7").to_owned();
    let repo = segments.next().unwrap_or("CodeContext").to_owned();
    (host, owner, repo)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub target: String,
    pub url: String,
    pub sha256: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub version: SemanticVersion,
    pub channel: UpdateChannel,
    pub published_at: String,
    pub changelog: String,
    pub artifacts: Vec<ReleaseArtifact>,
}

impl ReleaseManifest {
    #[inline]
    pub fn find_artifact_for_target(&self, target: &str) -> Option<&ReleaseArtifact> {
        self.artifacts.iter().find(|a| a.target == target)
    }
}

pub fn select_best_release<'a>(
    manifests: &'a [ReleaseManifest],
    current_version: &SemanticVersion,
    channel: UpdateChannel,
) -> Option<&'a ReleaseManifest> {
    manifests
        .iter()
        .filter(|m| channel.accepts_version(&m.version) && m.version > *current_version)
        .max_by(|a, b| a.version.cmp(&b.version))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfInstallStatus {
    InstalledCanonical { install_path: PathBuf },
    PortableMode { current_path: PathBuf, target_install_path: PathBuf },
}

impl SelfInstallStatus {
    #[inline]
    pub const fn is_installed(&self) -> bool {
        matches!(self, Self::InstalledCanonical { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UpdateStage {
    #[default]
    Idle,
    UpToDate {
        version: SemanticVersion,
    },
    UpdateAvailable {
        version: SemanticVersion,
        channel: UpdateChannel,
    },
    ReadyToRestart {
        version: SemanticVersion,
    },
    Failed {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UpdaterPortError {
    #[error("Ошибка сетевой синхронизации релиза: {0}")]
    Network(String),
    #[error("Ошибка разбора манифеста релиза: {0}")]
    InvalidManifest(String),
    #[error("Нарушение криптографической подписи релиза для `{0}`")]
    SignatureVerificationFailed(String),
    #[error("Несовпадение контрольной суммы SHA-256: ожидался `{expected}`, получен `{actual}`")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Ошибка файловой транзакции самообновления: {0}")]
    Storage(String),
}

pub trait UpdaterPort: Send + Sync {
    fn current_version(&self) -> SemanticVersion;

    fn self_install_status(&self) -> SelfInstallStatus;

    fn stage(&self) -> UpdateStage;

    fn switch_channel_cached(&self, channel: UpdateChannel) -> UpdateStage {
        let _ = channel;
        self.stage()
    }

    fn check_for_updates(
        &self,
        channel: UpdateChannel,
        repo_url: &str,
    ) -> Result<UpdateStage, UpdaterPortError>;

    fn apply_staged_update(&self) -> Result<UpdateStage, UpdaterPortError>;

    fn perform_self_install(&self) -> Result<SelfInstallStatus, UpdaterPortError>;
}
