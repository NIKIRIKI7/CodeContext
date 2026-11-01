//! Представление переводов строк (LF / CRLF) — Value Object.

use serde::{Deserialize, Serialize};

/// Тип переводов строк в текстовом файле.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LineEnding {
    /// Стандартный перевод строк Unix (`\n`).
    #[default]
    Lf,
    /// Перевод строк Windows (`\r\n`).
    Crlf,
    /// Смешанные переводы строк в одном файле.
    Mixed,
}

impl LineEnding {
    /// Определяет стиль переводов строк в переданном тексте за один проход (0 аллокаций).
    #[inline]
    pub fn detect(content: &str) -> Self {
        let bytes = content.as_bytes();
        let mut has_crlf = false;
        let mut has_lone_lf = false;

        for (idx, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                let prev_is_cr =
                    idx.checked_sub(1).and_then(|prev| bytes.get(prev)).copied() == Some(b'\r');
                if prev_is_cr {
                    has_crlf = true;
                } else {
                    has_lone_lf = true;
                }
                if has_crlf && has_lone_lf {
                    return Self::Mixed;
                }
            }
        }

        if has_crlf { Self::Crlf } else { Self::Lf }
    }

    /// Возвращает строковое представление последовательности перевода строки.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf | Self::Mixed => "\n",
            Self::Crlf => "\r\n",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineEnding;

    #[test]
    fn detects_lf_crlf_and_mixed_accurately() {
        assert_eq!(LineEnding::detect("fn main() {}\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect("fn main() {}\r\n"), LineEnding::Crlf);
        assert_eq!(LineEnding::detect("line1\r\nline2\n"), LineEnding::Mixed);
        assert_eq!(LineEnding::detect("single_line_no_newline"), LineEnding::Lf);
    }
}
