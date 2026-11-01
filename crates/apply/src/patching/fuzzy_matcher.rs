use core::shared_kernel::fs::line_ending::LineEnding;

const MAX_STACK_BIGRAMS: usize = 256;

#[derive(Debug, Clone)]
pub(crate) struct NonEmptyLine<'a> {
    pub raw_idx: usize,
    pub trimmed: &'a str,
    pub syntax_norm: String,
    pub indent_cols: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct IndexedText<'a> {
    pub raw_text: &'a str,
    pub raw_lines: Vec<&'a str>,
    pub non_empty: Vec<NonEmptyLine<'a>>,
}

impl<'a> IndexedText<'a> {
    pub(crate) fn new(raw_text: &'a str) -> Self {
        let raw_lines: Vec<&'a str> = raw_text.lines().collect();
        let mut non_empty = Vec::with_capacity(raw_lines.len());
        for (raw_idx, &line) in raw_lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let syntax_norm = normalize_loose_code_line(trimmed);
                let indent_cols = count_leading_indent_cols(line);
                non_empty.push(NonEmptyLine { raw_idx, trimmed, syntax_norm, indent_cols });
            }
        }
        Self { raw_text, raw_lines, non_empty }
    }

    pub(crate) fn replace_raw_line_range(
        &self,
        start_raw_idx: usize,
        end_raw_idx: usize,
        replace: &str,
        indent_delta: isize,
        target_ending: LineEnding,
    ) -> String {
        let eol = target_ending.as_str();
        let mut out = String::with_capacity(self.raw_text.len() + replace.len() + 32);
        let mut first = true;

        for &line in self.raw_lines.get(..start_raw_idx).unwrap_or(&[]) {
            if !first {
                out.push_str(eol);
            }
            out.push_str(line);
            first = false;
        }

        for rep_line in replace.lines() {
            if !first {
                out.push_str(eol);
            }
            append_reindented_line(&mut out, rep_line, indent_delta);
            first = false;
        }

        for &line in self.raw_lines.get(end_raw_idx..).unwrap_or(&[]) {
            if !first {
                out.push_str(eol);
            }
            out.push_str(line);
            first = false;
        }

        if self.raw_text.ends_with('\n') && !out.is_empty() {
            out.push_str(eol);
        }
        out
    }
}

#[inline]
fn count_leading_indent_cols(line: &str) -> usize {
    let mut cols = 0_usize;
    for b in line.bytes() {
        match b {
            b' ' => cols += 1,
            b'\t' => cols += 4,
            _ => break,
        }
    }
    cols
}

fn append_reindented_line(out: &mut String, rep_line: &str, indent_delta: isize) {
    if indent_delta <= 0 || rep_line.trim().is_empty() {
        out.push_str(rep_line);
        return;
    }
    let extra_spaces = usize::try_from(indent_delta).unwrap_or(0).min(32);
    for _ in 0..extra_spaces {
        out.push(' ');
    }
    out.push_str(rep_line);
}

#[inline]
fn compute_auto_indent_delta(
    matched_work_first_indent: usize,
    search_first_indent: usize,
    replace: &str,
) -> isize {
    let rep_first_indent =
        replace.lines().find(|l| !l.trim().is_empty()).map_or(0, count_leading_indent_cols);
    if search_first_indent == 0 && rep_first_indent == 0 && matched_work_first_indent > 0 {
        isize::try_from(matched_work_first_indent).unwrap_or(0)
    } else {
        0
    }
}

pub(crate) fn normalize_loose_code_line(line: &str) -> String {
    let trimmed = line.trim().trim_end_matches([',', ';']);
    let replaced = trimmed.replace("Option::None", "None");
    let mut out = String::with_capacity(replaced.len());
    let mut prev_space = false;
    for ch in replaced.chars() {
        if ch == '"' || ch == '\'' || ch == '`' {
            continue;
        }
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

#[inline]
pub(crate) fn is_ellipsis_placeholder_line(trimmed: &str) -> bool {
    if trimmed == "..." || trimmed == "…" {
        return true;
    }
    if let Some(comment) = trimmed
        .strip_prefix("//")
        .or_else(|| trimmed.strip_prefix('#'))
        .or_else(|| trimmed.strip_prefix("/*").and_then(|s| s.strip_suffix("*/")))
    {
        let c = comment.trim();
        return c == "..."
            || c == "…"
            || c.starts_with("... ")
            || c.ends_with(" ...")
            || c.contains("existing code")
            || c.contains("unchanged")
            || c.contains("остальной код")
            || c.contains("без изменений");
    }
    false
}

#[inline]
fn is_strong_boundary_anchor(syntax_norm: &str) -> bool {
    if syntax_norm.len() < 8 || syntax_norm.starts_with("#[") || syntax_norm.starts_with("#![") {
        return false;
    }
    !matches!(
        syntax_norm,
        "}" | "{"
            | "};"
            | ")"
            | ");"
            | "]"
            | "];"
            | "else {"
            | "else"
            | "Ok(())"
            | "return"
            | "None"
            | "true"
            | "false"
    )
}

/// Этап 3: Сопоставление по `trimmed`-строкам через индекс непустых строк.
pub(crate) fn try_trimmed_subsequence_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_idx = IndexedText::new(search);
    let s_len = search_idx.non_empty.len();
    if s_len == 0 {
        return None;
    }
    let work_idx = IndexedText::new(working);
    if work_idx.non_empty.len() < s_len {
        return None;
    }

    let max_raw_span = search_idx.raw_lines.len().saturating_add(12);
    let mut matched_span: Option<(usize, usize)> = None;

    for start_ne in 0..=(work_idx.non_empty.len() - s_len) {
        let end_ne = start_ne + s_len - 1;
        let window = work_idx.non_empty.get(start_ne..=end_ne)?;
        let raw_span =
            window.last()?.raw_idx.saturating_sub(window.first()?.raw_idx).saturating_add(1);
        if raw_span > max_raw_span {
            continue;
        }
        let all_eq =
            window.iter().zip(search_idx.non_empty.iter()).all(|(w, s)| w.trimmed == s.trimmed);
        if all_eq {
            if matched_span.is_some() {
                return None;
            }
            matched_span = Some((start_ne, end_ne));
        }
    }

    let (first_ne, last_ne) = matched_span?;
    let first_work = work_idx.non_empty.get(first_ne)?;
    let last_work = work_idx.non_empty.get(last_ne)?;
    let first_search = search_idx.non_empty.first()?;
    let indent_delta =
        compute_auto_indent_delta(first_work.indent_cols, first_search.indent_cols, replace);

    Some(work_idx.replace_raw_line_range(
        first_work.raw_idx,
        last_work.raw_idx + 1,
        replace,
        indent_delta,
        target_ending,
    ))
}

/// Этап 4: Сопоставление по синтаксически нормализованным строкам.
pub(crate) fn try_syntax_normalized_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_idx = IndexedText::new(search);
    let s_len = search_idx.non_empty.len();
    if s_len == 0 {
        return None;
    }
    let work_idx = IndexedText::new(working);
    if work_idx.non_empty.len() < s_len {
        return None;
    }

    let max_raw_span = search_idx.raw_lines.len().saturating_add(12);
    let mut matched_span: Option<(usize, usize)> = None;

    for start_ne in 0..=(work_idx.non_empty.len() - s_len) {
        let end_ne = start_ne + s_len - 1;
        let window = work_idx.non_empty.get(start_ne..=end_ne)?;
        let raw_span =
            window.last()?.raw_idx.saturating_sub(window.first()?.raw_idx).saturating_add(1);
        if raw_span > max_raw_span {
            continue;
        }
        let all_eq = window
            .iter()
            .zip(search_idx.non_empty.iter())
            .all(|(w, s)| !s.syntax_norm.is_empty() && w.syntax_norm == s.syntax_norm);
        if all_eq {
            if matched_span.is_some() {
                return None;
            }
            matched_span = Some((start_ne, end_ne));
        }
    }

    let (first_ne, last_ne) = matched_span?;
    let first_work = work_idx.non_empty.get(first_ne)?;
    let last_work = work_idx.non_empty.get(last_ne)?;
    let first_search = search_idx.non_empty.first()?;
    let indent_delta =
        compute_auto_indent_delta(first_work.indent_cols, first_search.indent_cols, replace);

    Some(work_idx.replace_raw_line_range(
        first_work.raw_idx,
        last_work.raw_idx + 1,
        replace,
        indent_delta,
        target_ending,
    ))
}

/// Этап 5: Поддержка `...` / `// ...` эллипсисов и игнорирования комментариев внутри SEARCH-блока.
pub(crate) fn try_ellipsis_and_comment_tolerant_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let has_ellipsis_or_comment = search.lines().any(|l| {
        let t = l.trim();
        is_ellipsis_placeholder_line(t) || t.starts_with("//") || t.starts_with('#')
    });
    if !has_ellipsis_or_comment {
        return None;
    }

    let work_idx = IndexedText::new(working);
    if work_idx.non_empty.is_empty() {
        return None;
    }

    let chunks = collect_non_comment_chunks(search);
    if chunks.is_empty() {
        return None;
    }

    let first_chunk = chunks.first()?;
    let mut candidate_ranges: Vec<(usize, usize)> = Vec::new();

    for start_ne in 0..work_idx.non_empty.len() {
        let Some(first_end_ne) = match_chunk_from(&work_idx.non_empty, start_ne, first_chunk)
        else {
            continue;
        };
        let mut cursor_ne = first_end_ne + 1;
        let mut last_matched_ne = first_end_ne;
        let mut all_chunks_ok = true;

        for next_chunk in chunks.get(1..).unwrap_or(&[]) {
            let max_search_ne = (cursor_ne + 48).min(work_idx.non_empty.len());
            let mut found_next = None;
            for probe_ne in cursor_ne..max_search_ne {
                if let Some(end_ne) = match_chunk_from(&work_idx.non_empty, probe_ne, next_chunk) {
                    found_next = Some(end_ne);
                    break;
                }
            }
            if let Some(end_ne) = found_next {
                last_matched_ne = end_ne;
                cursor_ne = end_ne + 1;
            } else {
                all_chunks_ok = false;
                break;
            }
        }

        if all_chunks_ok {
            candidate_ranges.push((start_ne, last_matched_ne));
        }
    }

    if candidate_ranges.len() != 1 {
        return None;
    }
    let &(first_ne, last_ne) = candidate_ranges.first()?;
    let first_work = work_idx.non_empty.get(first_ne)?;
    let last_work = work_idx.non_empty.get(last_ne)?;
    Some(work_idx.replace_raw_line_range(
        first_work.raw_idx,
        last_work.raw_idx + 1,
        replace,
        0,
        target_ending,
    ))
}

fn collect_non_comment_chunks(search: &str) -> Vec<Vec<String>> {
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut current_chunk: Vec<String> = Vec::new();

    for raw_line in search.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_ellipsis_placeholder_line(trimmed) {
            if !current_chunk.is_empty() {
                chunks.push(std::mem::take(&mut current_chunk));
            }
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        let norm = normalize_loose_code_line(trimmed);
        if !norm.is_empty() {
            current_chunk.push(norm);
        }
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    chunks
}

fn match_chunk_from(
    work_ne: &[NonEmptyLine<'_>],
    start_ne: usize,
    chunk_norms: &[String],
) -> Option<usize> {
    let mut w_cursor = start_ne;
    let mut last_matched = start_ne;
    for (idx, expected_norm) in chunk_norms.iter().enumerate() {
        while w_cursor < work_ne.len() {
            let w_line = work_ne.get(w_cursor)?;
            if (w_line.trimmed.starts_with("//") || w_line.trimmed.starts_with('#')) && idx > 0 {
                w_cursor += 1;
                continue;
            }
            break;
        }
        let actual = work_ne.get(w_cursor)?;
        if &actual.syntax_norm != expected_norm {
            return None;
        }
        last_matched = w_cursor;
        w_cursor += 1;
    }
    Some(last_matched)
}

/// Этап 6 (Эталон Zed): Нечёткое выравнивание скользящим окном с оценкой близости строк
/// по коэффициенту биграмм Сёренсена — Дайса и проверкой однозначности лучшего окна.
pub(crate) fn try_zed_fuzzy_similarity_replace(
    working: &str,
    search: &str,
    replace: &str,
    target_ending: LineEnding,
) -> Option<String> {
    let search_idx = IndexedText::new(search);
    let s_len = search_idx.non_empty.len();
    if s_len < 2 {
        return None;
    }
    let work_idx = IndexedText::new(working);
    let w_len = work_idx.non_empty.len();
    if w_len == 0 {
        return None;
    }

    if w_len >= s_len
        && let Some((best_start, best_end)) = find_equal_length_fuzzy_window(&work_idx, &search_idx)
    {
        let first_work = work_idx.non_empty.get(best_start)?;
        let last_work = work_idx.non_empty.get(best_end)?;
        return Some(work_idx.replace_raw_line_range(
            first_work.raw_idx,
            last_work.raw_idx + 1,
            replace,
            0,
            target_ending,
        ));
    }

    if s_len >= 3
        && let Some((first_ne, last_ne)) = find_anchor_bracketed_fuzzy_span(&work_idx, &search_idx)
    {
        let first_work = work_idx.non_empty.get(first_ne)?;
        let last_work = work_idx.non_empty.get(last_ne)?;
        return Some(work_idx.replace_raw_line_range(
            first_work.raw_idx,
            last_work.raw_idx + 1,
            replace,
            0,
            target_ending,
        ));
    }

    None
}

fn find_equal_length_fuzzy_window(
    work_idx: &IndexedText<'_>,
    search_idx: &IndexedText<'_>,
) -> Option<(usize, usize)> {
    let s_len = search_idx.non_empty.len();
    let w_len = work_idx.non_empty.len();
    let max_raw_span = search_idx.raw_lines.len().saturating_add(8);

    let mut best: Option<(usize, usize, u32)> = None;
    let mut second_best_score: u32 = 0;

    for start_ne in 0..=(w_len - s_len) {
        let end_ne = start_ne + s_len - 1;
        let window = work_idx.non_empty.get(start_ne..=end_ne)?;
        let raw_span =
            window.last()?.raw_idx.saturating_sub(window.first()?.raw_idx).saturating_add(1);
        if raw_span > max_raw_span {
            continue;
        }

        let first_match = window.first().map(|l| &l.syntax_norm)
            == search_idx.non_empty.first().map(|l| &l.syntax_norm);
        let last_match = window.last().map(|l| &l.syntax_norm)
            == search_idx.non_empty.last().map(|l| &l.syntax_norm);
        if !first_match && !last_match {
            continue;
        }

        let score = score_equal_length_window(window, &search_idx.non_empty);
        if let Some((best_start, _, best_score)) = best {
            if score > best_score {
                if start_ne.abs_diff(best_start) >= s_len {
                    second_best_score = best_score;
                }
                best = Some((start_ne, end_ne, score));
            } else if start_ne.abs_diff(best_start) >= s_len && score > second_best_score {
                second_best_score = score;
            }
        } else {
            best = Some((start_ne, end_ne, score));
        }
    }

    let (best_start, best_end, best_score) = best?;
    if best_score < 820 || (second_best_score > 0 && best_score < second_best_score + 80) {
        return None;
    }
    Some((best_start, best_end))
}

fn find_anchor_bracketed_fuzzy_span(
    work_idx: &IndexedText<'_>,
    search_idx: &IndexedText<'_>,
) -> Option<(usize, usize)> {
    let s_ne = &search_idx.non_empty;
    let w_ne = &work_idx.non_empty;
    let first_s = s_ne.first()?;
    let last_s = s_ne.last()?;

    if first_s.syntax_norm == last_s.syntax_norm
        || !is_strong_boundary_anchor(&first_s.syntax_norm)
        || !is_strong_boundary_anchor(&last_s.syntax_norm)
    {
        return None;
    }

    let min_span = s_ne.len().saturating_sub(2).max(2);
    let max_span = s_ne.len().saturating_add(4);
    let mut candidates: Vec<(usize, usize, u32)> = Vec::new();

    for start_ne in 0..w_ne.len() {
        let w_start = w_ne.get(start_ne)?;
        if w_start.syntax_norm != first_s.syntax_norm {
            continue;
        }
        let min_end = start_ne.saturating_add(min_span.saturating_sub(1));
        let max_end = start_ne.saturating_add(max_span).min(w_ne.len());
        if min_end >= max_end {
            continue;
        }

        for end_ne in min_end..max_end {
            let w_end = w_ne.get(end_ne)?;
            if w_end.syntax_norm != last_s.syntax_norm {
                continue;
            }
            let slice = w_ne.get(start_ne..=end_ne)?;
            let recall_permille = compute_ordered_line_recall_permille(slice, s_ne);
            let span_len_u32 = u32::try_from(slice.len()).unwrap_or(1);
            let search_len_u32 = u32::try_from(s_ne.len()).unwrap_or(1);
            let precision_permille =
                (recall_permille.saturating_mul(search_len_u32) / span_len_u32).min(1000);
            let combined_score = u32::midpoint(recall_permille, precision_permille);

            if recall_permille >= 650 && combined_score >= 600 {
                candidates.push((start_ne, end_ne, combined_score));
            }
        }
    }

    if candidates.len() == 1 {
        let &(s, e, _) = candidates.first()?;
        return Some((s, e));
    }

    candidates.sort_by_key(|a| std::cmp::Reverse(a.2));
    if let (Some(&top), Some(&second)) = (candidates.first(), candidates.get(1))
        && top.2 >= second.2 + 150
    {
        return Some((top.0, top.1));
    }
    None
}

fn compute_ordered_line_recall_permille(
    work_slice: &[NonEmptyLine<'_>],
    search_ne: &[NonEmptyLine<'_>],
) -> u32 {
    if search_ne.is_empty() {
        return 0;
    }
    let mut w_cursor = 0_usize;
    let mut matched_score_sum = 0_u32;

    for s_line in search_ne {
        let mut best_for_line = 0_u32;
        let mut best_pos = w_cursor;
        for (offset, w_line) in work_slice.get(w_cursor..).unwrap_or(&[]).iter().enumerate() {
            let sim = dice_similarity_permille(&w_line.syntax_norm, &s_line.syntax_norm);
            if sim > best_for_line {
                best_for_line = sim;
                best_pos = w_cursor + offset;
                if sim == 1000 {
                    break;
                }
            }
        }
        if best_for_line >= 750 {
            matched_score_sum += best_for_line;
            w_cursor = best_pos + 1;
        }
    }

    let denom = u32::try_from(search_ne.len()).unwrap_or(1);
    matched_score_sum / denom
}

fn score_equal_length_window(window: &[NonEmptyLine<'_>], search_ne: &[NonEmptyLine<'_>]) -> u32 {
    if window.is_empty() || window.len() != search_ne.len() {
        return 0;
    }
    let mut sum = 0_u32;
    for (w, s) in window.iter().zip(search_ne.iter()) {
        let line_sim = dice_similarity_permille(&w.syntax_norm, &s.syntax_norm);
        if window.len() <= 3 && line_sim < 600 {
            return 0;
        }
        sum += line_sim;
    }
    let len_u32 = u32::try_from(window.len()).unwrap_or(1);
    sum / len_u32
}

/// Вычисляет коэффициент сходства биграмм Сёренсена — Дайса в промилле (`0..=1000`) без аллокаций в куче.
pub(crate) fn dice_similarity_permille(a: &str, b: &str) -> u32 {
    if a == b {
        return 1000;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let a_len = a_bytes.len().min(MAX_STACK_BIGRAMS + 1);
    let b_len = b_bytes.len().min(MAX_STACK_BIGRAMS + 1);
    if a_len < 2 || b_len < 2 {
        return 0;
    }

    let a_bigrams_count = a_len - 1;
    let b_bigrams_count = b_len - 1;
    let mut b_used = [false; MAX_STACK_BIGRAMS];
    let mut matches = 0_usize;

    for i in 0..a_bigrams_count {
        let a0 = a_bytes.get(i).copied().unwrap_or(0);
        let a1 = a_bytes.get(i + 1).copied().unwrap_or(0);
        for j in 0..b_bigrams_count {
            if b_used.get(j).copied().unwrap_or(true) {
                continue;
            }
            let b0 = b_bytes.get(j).copied().unwrap_or(0);
            let b1 = b_bytes.get(j + 1).copied().unwrap_or(0);
            if a0 == b0 && a1 == b1 {
                if let Some(slot) = b_used.get_mut(j) {
                    *slot = true;
                }
                matches += 1;
                break;
            }
        }
    }

    let numer = u32::try_from(matches.saturating_mul(2000)).unwrap_or(0);
    let denom = u32::try_from(a_bigrams_count + b_bigrams_count).unwrap_or(1);
    (numer / denom).min(1000)
}
