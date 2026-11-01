//! Доменный сервис разбиения крупного контекста на части (`Chunker`).

use crate::packing::services::template_engine::TemplateEngine;
use crate::packing::values::pack_chunk::PackChunk;
use crate::preset::models::template::OutputTemplate;

/// Сервис сегментированной упаковки контекста без разрыва файлов посередине.
#[derive(Debug, Default, Clone, Copy)]
pub struct Chunker;

impl Chunker {
    /// Упаковывает список файлов в один или несколько `PackChunk` согласно `template.max_chunk_bytes`.
    /// Каждый `PackChunk` аллоцируется в куче ровно 1 раз по заранее рассчитанным границам.
    pub fn pack_into_chunks<P, C>(
        files: &[(P, C)],
        template: &OutputTemplate,
        minify: bool,
        normalize_lf: bool,
    ) -> Vec<PackChunk>
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        if files.is_empty() {
            return vec![PackChunk::new(0, 1, String::new(), 0)];
        }

        let Some(max_bytes) = template.max_chunk_bytes.filter(|&limit| limit > 0) else {
            let content = TemplateEngine::format_files_preallocated(
                files,
                template,
                true,
                minify,
                normalize_lf,
            );
            return vec![PackChunk::new(0, 1, content, files.len())];
        };

        let ranges = Self::compute_chunk_ranges(files, template, max_bytes);
        let total_chunks = ranges.len();
        let mut chunks = Vec::with_capacity(total_chunks);

        for (chunk_idx, (start, end)) in ranges.into_iter().enumerate() {
            let slice = files.get(start..end).unwrap_or(&[]);
            let content = TemplateEngine::format_files_preallocated(
                slice,
                template,
                chunk_idx == 0,
                minify,
                normalize_lf,
            );
            chunks.push(PackChunk::new(chunk_idx, total_chunks, content, slice.len()));
        }

        chunks
    }

    fn compute_chunk_ranges<P, C>(
        files: &[(P, C)],
        template: &OutputTemplate,
        max_bytes: usize,
    ) -> Vec<(usize, usize)>
    where
        P: AsRef<str>,
        C: AsRef<str>,
    {
        let fixed_marker = template.fixed_marker_bytes_per_file();
        let base_overhead = template.preamble.len() + template.postamble.len();
        let first_chunk_tree_overhead = if template.include_file_tree {
            TemplateEngine::calculate_tree_header_len(files)
        } else {
            0
        };

        let mut ranges = Vec::with_capacity(16);
        let mut range_start = 0;
        let mut current_bytes = base_overhead + first_chunk_tree_overhead;

        for (idx, (path, content)) in files.iter().enumerate() {
            let file_cost = path.as_ref().len() + content.as_ref().len() + fixed_marker;
            if idx > range_start && current_bytes.saturating_add(file_cost) > max_bytes {
                ranges.push((range_start, idx));
                range_start = idx;
                current_bytes = base_overhead + file_cost;
            } else {
                current_bytes = current_bytes.saturating_add(file_cost);
            }
        }

        if range_start < files.len() {
            ranges.push((range_start, files.len()));
        }

        ranges
    }
}

#[cfg(test)]
mod tests {
    use super::Chunker;
    use crate::preset::models::template::OutputTemplate;

    #[test]
    fn splits_files_across_chunks_without_breaking_individual_files() {
        let files = vec![
            ("src/a.rs", "fn a() {}\n"),
            ("src/b.rs", "fn b() {}\n"),
            ("src/c.rs", "fn c() {}\n"),
        ];

        let tpl = OutputTemplate::canonical().with_max_chunk_bytes(Some(110));
        let chunks = Chunker::pack_into_chunks(&files, &tpl, false, false);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[0].total, 2);
        assert_eq!(chunks[0].file_count, 2);
        assert!(chunks[0].content.contains("=== FILE: src/a.rs ==="));
        assert!(chunks[0].content.contains("=== FILE: src/b.rs ==="));

        assert_eq!(chunks[1].index, 1);
        assert_eq!(chunks[1].total, 2);
        assert_eq!(chunks[1].file_count, 1);
        assert!(chunks[1].content.contains("=== FILE: src/c.rs ==="));
    }
}
