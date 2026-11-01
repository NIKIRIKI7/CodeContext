use core::packing::services::template_engine::TemplateEngine;

#[inline]
pub(crate) fn append_minified(rel_path: &str, input: &str, out: &mut String) {
    TemplateEngine::append_minified_to_buffer(rel_path, input, out);
}
