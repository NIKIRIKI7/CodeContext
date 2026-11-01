use core::packing::services::template_engine::{PlainTextSink, TemplateEngine, TextSink};
use memchr::memchr_iter;

use super::minifier::append_minified;

struct SimdLfNormalizingSink<'a> {
    out: &'a mut String,
}

impl<'a> SimdLfNormalizingSink<'a> {
    #[inline]
    const fn new(out: &'a mut String) -> Self {
        Self { out }
    }
}

impl TextSink for SimdLfNormalizingSink<'_> {
    #[inline]
    fn push_str(&mut self, raw: &str) {
        let mut last = 0;
        for pos in memchr_iter(b'\r', raw.as_bytes()) {
            if let Some(chunk) = raw.get(last..pos) {
                self.out.push_str(chunk);
            }
            last = pos + 1;
        }
        if let Some(tail) = raw.get(last..) {
            self.out.push_str(tail);
        }
    }

    #[inline]
    fn push_char(&mut self, ch: char) {
        if ch != '\r' {
            self.out.push(ch);
        }
    }
}

#[inline]
pub(crate) fn append_transformed_content(
    rel_path: &str,
    raw: &str,
    out: &mut String,
    minify: bool,
    normalize_lf: bool,
) {
    if normalize_lf {
        let mut simd_sink = SimdLfNormalizingSink::new(out);
        if minify {
            TemplateEngine::append_minified_to_sink(rel_path, raw, &mut simd_sink);
        } else {
            simd_sink.push_str(raw);
        }
    } else if minify {
        append_minified(rel_path, raw, out);
    } else {
        let mut plain_sink = PlainTextSink::new(out);
        plain_sink.push_str(raw);
    }
}
