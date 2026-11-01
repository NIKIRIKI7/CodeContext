use crate::preset::models::preset::Preset;
use crate::shared_kernel::execution::anomaly::Anomaly;
use crate::shared_kernel::fs::file_kind::{
    FileKind, contains_ignore_ascii_case, ends_with_ignore_ascii_case, split_file_name_and_ext,
    starts_with_ignore_ascii_case,
};

const SENSITIVE_DIRS: &[&str] = &[".ssh", ".aws", ".gnupg", ".kube", ".docker"];
const SAFE_CODE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "c", "cpp", "cc", "h", "hpp", "java", "kt",
    "swift", "cs", "rb", "php", "html", "css", "scss", "sql", "md", "mdx", "lock",
];
const EXACT_SECRET_FILENAMES: &[&str] = &[
    ".env",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    "id_xmss",
    ".npmrc",
    ".pypirc",
    ".netrc",
    ".htpasswd",
    "credentials",
    "credentials.json",
    "client_secret.json",
    "secrets.json",
    "secrets.yaml",
    "secrets.yml",
    "secrets.toml",
    "secrets.ini",
    "master.key",
];
const SAFE_DOTENV_TEMPLATES: &[&str] =
    &[".env.example", ".env.sample", ".env.template", ".env.dist", ".env.schema", ".env.defaults"];
const SECRET_EXTENSIONS: &[&str] =
    &["pem", "key", "pfx", "p12", "kdbx", "jks", "keystore", "ovpn", "asc", "der", "csr"];
const DATA_EXTENSIONS_FOR_SECRET_SUBSTRING: &[&str] =
    &["json", "yaml", "yml", "toml", "ini", "cfg", "conf", "env", "txt", "properties", "xml"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretMatchReason {
    DotEnvFile,
    SshOrPrivateKeyFile,
    CertificateOrKeyExtension,
    CredentialsConfigFile,
    SecretKeywordInConfigName,
    SensitiveDirectory,
    PrivateKeyHeaderInContent,
}

impl SecretMatchReason {
    #[inline]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DotEnvFile => "DotEnvFile",
            Self::SshOrPrivateKeyFile => "SshOrPrivateKeyFile",
            Self::CertificateOrKeyExtension => "CertificateOrKeyExtension",
            Self::CredentialsConfigFile => "CredentialsConfigFile",
            Self::SecretKeywordInConfigName => "SecretKeywordInConfigName",
            Self::SensitiveDirectory => "SensitiveDirectory",
            Self::PrivateKeyHeaderInContent => "PrivateKeyHeaderInContent",
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DotEnvFile => "Файл окружения (.env*)",
            Self::SshOrPrivateKeyFile => "Приватный SSH/криптоключ (id_rsa / id_ed25519)",
            Self::CertificateOrKeyExtension => {
                "Сертификат или ключевое хранилище (*.pem, *.key, *.p12)"
            }
            Self::CredentialsConfigFile => "Файл учётных данных (credentials / .npmrc / .netrc)",
            Self::SecretKeywordInConfigName => "Конфигурационный файл с маркером *secret*",
            Self::SensitiveDirectory => "Защищённая директория (.ssh / .aws / .gnupg)",
            Self::PrivateKeyHeaderInContent => {
                "Обнаружен приватный ключ (PRIVATE KEY) в содержимом файла"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretRuleVerdict {
    PassToNext,
    AllowSafe,
    Block(SecretMatchReason),
}

#[derive(Debug, Clone, Copy)]
pub struct PathCandidate<'a> {
    pub rel_path: &'a str,
    pub file_name: &'a str,
    pub ext_opt: Option<&'a str>,
    pub starts_with_dot: bool,
    pub starts_with_id: bool,
}

impl<'a> PathCandidate<'a> {
    #[inline]
    pub fn parse(rel_path: &'a str) -> Option<Self> {
        let (file_name, ext_opt) = split_file_name_and_ext(rel_path);
        if file_name.is_empty() {
            return None;
        }
        let starts_with_dot = file_name.as_bytes().first().copied() == Some(b'.');
        let starts_with_id = starts_with_ignore_ascii_case(file_name, "id_");
        Some(Self { rel_path, file_name, ext_opt, starts_with_dot, starts_with_id })
    }
}

pub type PathSecretRuleFn = fn(&PathCandidate<'_>) -> SecretRuleVerdict;
pub type ContentSecretRuleFn = fn(&str) -> Option<SecretMatchReason>;

const DEFAULT_PATH_RULE_CHAIN: &[PathSecretRuleFn] = &[
    rule_safe_code_extension_fast_pass,
    rule_public_key_extension_allow,
    rule_dotenv_templates_and_files,
    rule_exact_credential_filenames,
    rule_ssh_private_key_prefixes,
    rule_certificate_or_key_extension,
    rule_secret_keyword_in_data_config,
];

const DEFAULT_CONTENT_RULE_CHAIN: &[ContentSecretRuleFn] = &[rule_private_key_header_in_content];

#[derive(Debug, Default, Clone, Copy)]
pub struct SecretGuard;

impl SecretGuard {
    #[inline]
    pub fn inspect_path(rel_path: &str) -> Option<SecretMatchReason> {
        Self::inspect_path_with_chain(rel_path, DEFAULT_PATH_RULE_CHAIN)
    }

    #[inline]
    pub fn inspect_path_with_chain(
        rel_path: &str,
        chain: &[PathSecretRuleFn],
    ) -> Option<SecretMatchReason> {
        if rule_sensitive_directory_raw(rel_path) {
            return Some(SecretMatchReason::SensitiveDirectory);
        }
        let candidate = PathCandidate::parse(rel_path)?;
        for &rule in chain {
            match rule(&candidate) {
                SecretRuleVerdict::PassToNext => {}
                SecretRuleVerdict::AllowSafe => return None,
                SecretRuleVerdict::Block(reason) => return Some(reason),
            }
        }
        None
    }

    #[inline]
    pub fn inspect_content_prefix(content: &str) -> Option<SecretMatchReason> {
        Self::inspect_content_with_chain(content, DEFAULT_CONTENT_RULE_CHAIN)
    }

    #[inline]
    pub fn inspect_content_with_chain(
        content: &str,
        chain: &[ContentSecretRuleFn],
    ) -> Option<SecretMatchReason> {
        let prefix = content.get(..4096).unwrap_or(content);
        for &rule in chain {
            if let Some(reason) = rule(prefix) {
                return Some(reason);
            }
        }
        None
    }

    pub fn classify_entry(
        rel_path: &str,
        preset: &Preset,
    ) -> (FileKind, Option<SecretMatchReason>, Option<Anomaly>) {
        if let Some(reason) = Self::inspect_path(rel_path) {
            let anomaly = if preset.is_secret_explicitly_allowed(rel_path) {
                None
            } else {
                Some(Anomaly::SecretBlocked {
                    rel_path: rel_path.to_owned(),
                    reason: reason.as_str().to_owned(),
                })
            };
            return (FileKind::Secret, Some(reason), anomaly);
        }
        let kind = FileKind::classify_by_path(rel_path);
        (kind, None, None)
    }
}

#[inline]
fn rule_sensitive_directory_raw(rel_path: &str) -> bool {
    let bytes = rel_path.as_bytes();
    let has_dot_dir =
        bytes.first().copied() == Some(b'.') || bytes.windows(2).any(|w| w == b"/." || w == b"\\.");
    if !has_dot_dir {
        return false;
    }
    for segment in rel_path.split(['/', '\\']) {
        if segment.as_bytes().first().copied() == Some(b'.')
            && SENSITIVE_DIRS.iter().any(|dir| segment.eq_ignore_ascii_case(dir))
        {
            return true;
        }
    }
    false
}

#[inline]
fn rule_safe_code_extension_fast_pass(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if !candidate.starts_with_dot
        && !candidate.starts_with_id
        && let Some(ext) = candidate.ext_opt
        && SAFE_CODE_EXTENSIONS.iter().any(|c| ext.eq_ignore_ascii_case(c))
    {
        return SecretRuleVerdict::AllowSafe;
    }
    SecretRuleVerdict::PassToNext
}

#[inline]
fn rule_public_key_extension_allow(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if candidate.ext_opt.is_some_and(|ext| ext.eq_ignore_ascii_case("pub")) {
        SecretRuleVerdict::AllowSafe
    } else {
        SecretRuleVerdict::PassToNext
    }
}

#[inline]
fn rule_dotenv_templates_and_files(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if candidate.starts_with_dot {
        if SAFE_DOTENV_TEMPLATES.iter().any(|safe| candidate.file_name.eq_ignore_ascii_case(safe)) {
            return SecretRuleVerdict::AllowSafe;
        }
        if candidate.file_name.eq_ignore_ascii_case(".env")
            || starts_with_ignore_ascii_case(candidate.file_name, ".env.")
        {
            return SecretRuleVerdict::Block(SecretMatchReason::DotEnvFile);
        }
    } else if ends_with_ignore_ascii_case(candidate.file_name, ".env") {
        return SecretRuleVerdict::Block(SecretMatchReason::DotEnvFile);
    }
    SecretRuleVerdict::PassToNext
}

#[inline]
fn rule_exact_credential_filenames(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if EXACT_SECRET_FILENAMES.iter().any(|exact| candidate.file_name.eq_ignore_ascii_case(exact)) {
        SecretRuleVerdict::Block(SecretMatchReason::CredentialsConfigFile)
    } else {
        SecretRuleVerdict::PassToNext
    }
}

#[inline]
fn rule_ssh_private_key_prefixes(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if candidate.starts_with_id
        && (starts_with_ignore_ascii_case(candidate.file_name, "id_rsa")
            || starts_with_ignore_ascii_case(candidate.file_name, "id_ed25519")
            || starts_with_ignore_ascii_case(candidate.file_name, "id_ecdsa")
            || starts_with_ignore_ascii_case(candidate.file_name, "id_dsa"))
    {
        SecretRuleVerdict::Block(SecretMatchReason::SshOrPrivateKeyFile)
    } else {
        SecretRuleVerdict::PassToNext
    }
}

#[inline]
fn rule_certificate_or_key_extension(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if let Some(ext) = candidate.ext_opt
        && SECRET_EXTENSIONS.iter().any(|sec_ext| ext.eq_ignore_ascii_case(sec_ext))
    {
        return SecretRuleVerdict::Block(SecretMatchReason::CertificateOrKeyExtension);
    }
    SecretRuleVerdict::PassToNext
}

#[inline]
fn rule_secret_keyword_in_data_config(candidate: &PathCandidate<'_>) -> SecretRuleVerdict {
    if let Some(ext) = candidate.ext_opt {
        let is_data_ext = DATA_EXTENSIONS_FOR_SECRET_SUBSTRING
            .iter()
            .any(|d_ext| ext.eq_ignore_ascii_case(d_ext));
        if is_data_ext
            && (contains_ignore_ascii_case(candidate.file_name, "secret")
                || starts_with_ignore_ascii_case(candidate.file_name, "service-account")
                || starts_with_ignore_ascii_case(candidate.file_name, "service_account"))
        {
            return SecretRuleVerdict::Block(SecretMatchReason::SecretKeywordInConfigName);
        }
    }
    SecretRuleVerdict::PassToNext
}

#[inline]
fn rule_private_key_header_in_content(prefix: &str) -> Option<SecretMatchReason> {
    for line in prefix.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-----BEGIN ") && trimmed.ends_with("PRIVATE KEY-----") {
            return Some(SecretMatchReason::PrivateKeyHeaderInContent);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        PathCandidate, PathSecretRuleFn, SecretGuard, SecretMatchReason, SecretRuleVerdict,
    };
    use crate::preset::models::preset::Preset;
    use crate::shared_kernel::fs::file_kind::FileKind;

    #[test]
    fn blocks_all_secret_patterns_and_allows_safe_templates_and_source_code() {
        let blocked = [
            ".env",
            ".env.local",
            ".env.production",
            "config/.env.staging",
            "production.env",
            "id_rsa",
            "id_ed25519",
            "keys/id_rsa_deploy",
            "certs/server.pem",
            "certs/private.key",
            "auth/keystore.p12",
            "config/my_secret_tokens.json",
            "gcp/service-account-prod.json",
            ".aws/credentials",
            "home/.ssh/config",
            ".npmrc",
        ];
        for path in blocked {
            assert!(
                SecretGuard::inspect_path(path).is_some(),
                "SecretGuard обязан заблокировать путь `{path}`"
            );
        }

        let allowed = [
            ".env.example",
            ".env.sample",
            ".env.template",
            "keys/id_rsa.pub",
            "keys/id_ed25519.pub",
            "crates/core/src/packing/services/secret_guard.rs",
            "src/main.rs",
            "README.md",
            "Cargo.toml",
        ];
        for path in allowed {
            assert!(
                SecretGuard::inspect_path(path).is_none(),
                "SecretGuard не должен давать ложное срабатывание на `{path}`"
            );
        }
    }

    #[test]
    fn scenario_a3_untrusted_project_config_cannot_bypass_secret_guard() {
        let mut untrusted_preset = Preset::default();
        untrusted_preset.include_globs.push(".env".to_owned());
        untrusted_preset.explicitly_allowed_secrets.push(".env".to_owned());

        let safe_preset = untrusted_preset.sanitized_from_untrusted_project_config();
        let (kind, reason, anomaly) = SecretGuard::classify_entry(".env", &safe_preset);
        assert_eq!(kind, FileKind::Secret);
        assert_eq!(reason, Some(SecretMatchReason::DotEnvFile));
        assert!(anomaly.is_some());
    }

    #[test]
    fn detects_private_key_headers_in_file_content() {
        let content =
            "some config\n-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmU=\n";
        assert_eq!(
            SecretGuard::inspect_content_prefix(content),
            Some(SecretMatchReason::PrivateKeyHeaderInContent)
        );
        assert_eq!(SecretGuard::inspect_content_prefix("pub fn normal_rust_code() {}\n"), None);
    }

    #[test]
    fn custom_chain_of_responsibility_rule_extends_secret_guard_without_modifying_core() {
        fn custom_vault_rule(c: &PathCandidate<'_>) -> SecretRuleVerdict {
            if c.file_name.eq_ignore_ascii_case("vault.token") {
                SecretRuleVerdict::Block(SecretMatchReason::CredentialsConfigFile)
            } else {
                SecretRuleVerdict::PassToNext
            }
        }
        let chain: [PathSecretRuleFn; 1] = [custom_vault_rule];
        assert_eq!(
            SecretGuard::inspect_path_with_chain("deploy/vault.token", &chain),
            Some(SecretMatchReason::CredentialsConfigFile)
        );
        assert_eq!(SecretGuard::inspect_path_with_chain("src/lib.rs", &chain), None);
    }
}
