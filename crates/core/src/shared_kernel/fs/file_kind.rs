use serde::{Deserialize, Serialize};

const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "tiff", "avif", "pdf", "zip", "tar", "gz",
    "tgz", "bz2", "xz", "7z", "rar", "zst", "exe", "dll", "so", "dylib", "lib", "a", "o", "obj",
    "rlib", "rmeta", "pdb", "wasm", "class", "jar", "pyc", "pyd", "ttf", "otf", "woff", "woff2",
    "eot", "mp3", "wav", "ogg", "flac", "mp4", "avi", "mov", "mkv", "webm", "sqlite", "sqlite3",
    "db", "mdb", "lockb", "bin", "dat", "iso", "dmg",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommentStyle {
    CStyle,
    Hash,
    Xml,
    SqlDash,
    Semicolon,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinifyStyle {
    SingleLineCStyle,
    IndentationPreserving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageSyntaxProfile {
    pub file_kind: FileKind,
    pub comment_style: CommentStyle,
    pub minify_style: MinifyStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct LanguageSyntaxRule {
    pub group_name: &'static str,
    pub extensions: &'static [&'static str],
    pub exact_filenames: &'static [&'static str],
    pub profile: LanguageSyntaxProfile,
}

pub const DEFAULT_LANGUAGE_SYNTAX_RULES: &[LanguageSyntaxRule] = &[
    LanguageSyntaxRule {
        group_name: "CStyleSource",
        extensions: &[
            "rs", "ts", "tsx", "js", "jsx", "go", "c", "cpp", "cc", "h", "hpp", "java", "kt",
            "swift", "cs", "php", "css", "scss", "zig", "dart", "scala", "wgsl", "glsl", "proto",
            "groovy",
        ],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::SourceCode,
            comment_style: CommentStyle::CStyle,
            minify_style: MinifyStyle::SingleLineCStyle,
        },
    },
    LanguageSyntaxRule {
        group_name: "IndentationHashSource",
        extensions: &[
            "py", "pyi", "pyw", "pyx", "pxd", "sh", "bash", "zsh", "ps1", "rb", "pl", "r", "ex",
            "exs", "nix", "tf", "hcl", "graphql", "gql",
        ],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::SourceCode,
            comment_style: CommentStyle::Hash,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
    LanguageSyntaxRule {
        group_name: "MarkupXmlSource",
        extensions: &["html", "htm", "xml", "svg", "vue", "svelte"],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::SourceCode,
            comment_style: CommentStyle::Xml,
            minify_style: MinifyStyle::SingleLineCStyle,
        },
    },
    LanguageSyntaxRule {
        group_name: "DashCommentSource",
        extensions: &["sql", "lua", "hs", "erl", "ml", "elm"],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::SourceCode,
            comment_style: CommentStyle::SqlDash,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
    LanguageSyntaxRule {
        group_name: "SemicolonCommentSource",
        extensions: &["clj", "cljs", "edn", "lisp", "scm", "asm", "s"],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::SourceCode,
            comment_style: CommentStyle::Semicolon,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
    LanguageSyntaxRule {
        group_name: "BatchScriptSource",
        extensions: &["bat", "cmd"],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::SourceCode,
            comment_style: CommentStyle::None,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
    LanguageSyntaxRule {
        group_name: "HashCommentConfiguration",
        extensions: &["toml", "yaml", "yml", "ini", "cfg", "conf", "properties"],
        exact_filenames: &["justfile", "makefile", "dockerfile", "cmakelists.txt"],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::Configuration,
            comment_style: CommentStyle::Hash,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
    LanguageSyntaxRule {
        group_name: "JsonCStyleConfiguration",
        extensions: &["jsonc"],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::Configuration,
            comment_style: CommentStyle::CStyle,
            minify_style: MinifyStyle::SingleLineCStyle,
        },
    },
    LanguageSyntaxRule {
        group_name: "StrictConfiguration",
        extensions: &["json", "lock"],
        exact_filenames: &[],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::Configuration,
            comment_style: CommentStyle::None,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
    LanguageSyntaxRule {
        group_name: "Documentation",
        extensions: &["md", "mdx", "txt", "rst", "adoc"],
        exact_filenames: &["readme", "license", "changelog", "notice", "contributing"],
        profile: LanguageSyntaxProfile {
            file_kind: FileKind::Documentation,
            comment_style: CommentStyle::None,
            minify_style: MinifyStyle::IndentationPreserving,
        },
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    SourceCode,
    Configuration,
    Documentation,
    Secret,
    Binary,
    SymlinkSkipped,
}

impl FileKind {
    #[inline]
    pub const fn is_packable_by_default(self) -> bool {
        matches!(self, Self::SourceCode | Self::Configuration | Self::Documentation)
    }

    #[inline]
    pub const fn is_secret(self) -> bool {
        matches!(self, Self::Secret)
    }

    #[inline]
    pub const fn is_binary(self) -> bool {
        matches!(self, Self::Binary)
    }

    #[inline]
    pub fn is_known_binary_extension(rel_path: &str) -> bool {
        let (_, Some(ext)) = split_file_name_and_ext(rel_path) else {
            return false;
        };
        if matches_any_rule_extension(ext, DEFAULT_LANGUAGE_SYNTAX_RULES).is_some() {
            return false;
        }
        BINARY_EXTENSIONS.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate))
    }

    #[inline]
    pub fn classify_by_path(rel_path: &str) -> Self {
        Self::syntax_profile_for_path(rel_path).file_kind
    }

    #[inline]
    pub fn syntax_profile_for_path(rel_path: &str) -> LanguageSyntaxProfile {
        Self::syntax_profile_with_rules(rel_path, DEFAULT_LANGUAGE_SYNTAX_RULES)
    }

    #[inline]
    pub fn syntax_profile_with_rules(
        rel_path: &str,
        rules: &[LanguageSyntaxRule],
    ) -> LanguageSyntaxProfile {
        let (file_name, ext_opt) = split_file_name_and_ext(rel_path);

        if let Some(ext) = ext_opt {
            if let Some(profile) = matches_any_rule_extension(ext, rules) {
                return profile;
            }
            if BINARY_EXTENSIONS.iter().any(|c| ext.eq_ignore_ascii_case(c)) {
                return LanguageSyntaxProfile {
                    file_kind: Self::Binary,
                    comment_style: CommentStyle::None,
                    minify_style: MinifyStyle::IndentationPreserving,
                };
            }
        }

        if let Some(profile) = matches_any_rule_filename(file_name, rules) {
            return profile;
        }

        LanguageSyntaxProfile {
            file_kind: Self::SourceCode,
            comment_style: CommentStyle::CStyle,
            minify_style: MinifyStyle::SingleLineCStyle,
        }
    }
}

#[inline]
fn matches_any_rule_extension(
    ext: &str,
    rules: &[LanguageSyntaxRule],
) -> Option<LanguageSyntaxProfile> {
    for rule in rules {
        if rule.extensions.iter().any(|candidate| ext.eq_ignore_ascii_case(candidate)) {
            return Some(rule.profile);
        }
    }
    None
}

#[inline]
fn matches_any_rule_filename(
    file_name: &str,
    rules: &[LanguageSyntaxRule],
) -> Option<LanguageSyntaxProfile> {
    for rule in rules {
        if rule.exact_filenames.iter().any(|candidate| file_name.eq_ignore_ascii_case(candidate)) {
            return Some(rule.profile);
        }
    }
    None
}

#[inline]
pub fn split_file_name_and_ext(rel_path: &str) -> (&str, Option<&str>) {
    let file_name = match rel_path.rfind(['/', '\\']) {
        Some(pos) => rel_path.get(pos + 1..).unwrap_or(rel_path),
        None => rel_path,
    };
    let ext = match file_name.rfind('.') {
        Some(dot_pos) if dot_pos > 0 && dot_pos + 1 < file_name.len() => {
            file_name.get(dot_pos + 1..)
        }
        _ => None,
    };
    (file_name, ext)
}

#[inline]
pub fn starts_with_ignore_ascii_case(haystack: &str, prefix: &str) -> bool {
    haystack
        .as_bytes()
        .get(..prefix.len())
        .is_some_and(|slice| slice.eq_ignore_ascii_case(prefix.as_bytes()))
}

#[inline]
pub fn ends_with_ignore_ascii_case(haystack: &str, suffix: &str) -> bool {
    let len = haystack.len();
    let suf_len = suffix.len();
    len >= suf_len
        && haystack
            .as_bytes()
            .get(len - suf_len..)
            .is_some_and(|slice| slice.eq_ignore_ascii_case(suffix.as_bytes()))
}

#[inline]
pub fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    let n_bytes = needle.as_bytes();
    if n_bytes.is_empty() {
        return true;
    }
    haystack.as_bytes().windows(n_bytes.len()).any(|window| window.eq_ignore_ascii_case(n_bytes))
}

#[cfg(test)]
mod tests {
    use super::{CommentStyle, FileKind, LanguageSyntaxProfile, LanguageSyntaxRule, MinifyStyle};

    #[test]
    fn classifies_binary_docs_config_and_source_without_allocations() {
        assert_eq!(FileKind::classify_by_path("assets/logo.PNG"), FileKind::Binary);
        assert_eq!(FileKind::classify_by_path("target/app.exe"), FileKind::Binary);
        assert_eq!(FileKind::classify_by_path("docs/spec.md"), FileKind::Documentation);
        assert_eq!(FileKind::classify_by_path("LICENSE"), FileKind::Documentation);
        assert_eq!(FileKind::classify_by_path("Cargo.toml"), FileKind::Configuration);
        assert_eq!(FileKind::classify_by_path("Justfile"), FileKind::Configuration);
        assert_eq!(FileKind::classify_by_path("src/main.rs"), FileKind::SourceCode);
        assert!(FileKind::SourceCode.is_packable_by_default());
        assert!(!FileKind::Binary.is_packable_by_default());
        assert!(!FileKind::Secret.is_packable_by_default());

        let py_prof = FileKind::syntax_profile_for_path("scripts/run.py");
        assert_eq!(py_prof.comment_style, CommentStyle::Hash);
        assert_eq!(py_prof.minify_style, MinifyStyle::IndentationPreserving);

        let rs_prof = FileKind::syntax_profile_for_path("src/lib.rs");
        assert_eq!(rs_prof.comment_style, CommentStyle::CStyle);
        assert_eq!(rs_prof.minify_style, MinifyStyle::SingleLineCStyle);
    }

    #[test]
    fn custom_language_syntax_rule_extends_codestyle_registry_declaratively() {
        const CUSTOM_RULES: &[LanguageSyntaxRule] = &[LanguageSyntaxRule {
            group_name: "CustomDSL",
            extensions: &["dsl"],
            exact_filenames: &["Dslfile"],
            profile: LanguageSyntaxProfile {
                file_kind: FileKind::Configuration,
                comment_style: CommentStyle::Semicolon,
                minify_style: MinifyStyle::IndentationPreserving,
            },
        }];

        let prof = FileKind::syntax_profile_with_rules("rules/pipeline.dsl", CUSTOM_RULES);
        assert_eq!(prof.file_kind, FileKind::Configuration);
        assert_eq!(prof.comment_style, CommentStyle::Semicolon);
        assert_eq!(prof.minify_style, MinifyStyle::IndentationPreserving);
    }
}
