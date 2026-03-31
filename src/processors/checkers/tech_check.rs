use anyhow::{Result, bail};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::config::TechCheckConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{discover_checker_products, execute_checker_batch};

pub struct TechCheckProcessor {
    config: TechCheckConfig,
}

impl TechCheckProcessor {
    pub fn new(config: TechCheckConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    fn check_files(&self, files: &[&Path]) -> Result<()> {
        let terms = load_terms(&self.config.tech_files_dir)?;
        if terms.is_empty() {
            return Ok(());
        }
        let sorted_terms = sorted_terms(&terms);
        let mut errors = Vec::new();
        let mut all_backticked = HashSet::new();

        for file in files {
            let content = fs::read_to_string(file)?;
            let unquoted = find_unquoted_terms(&content, &sorted_terms);
            for (line_num, term) in &unquoted {
                errors.push(format!(
                    "{}:{}: tech term `{}` is not backtick-quoted",
                    file.display(), line_num, term,
                ));
            }
            let backticked = find_backticked_terms(&content);
            for t in &backticked {
                let is_known = terms.iter().any(|term| term.eq_ignore_ascii_case(t));
                if !is_known {
                    errors.push(format!(
                        "{}: `{}` is backtick-quoted but not in tech term list",
                        file.display(), t,
                    ));
                }
            }
            all_backticked.extend(backticked);
        }

        // Check for unused terms (terms in the list but never backticked in any file)
        for term in &terms {
            let found = all_backticked.iter().any(|b| b.eq_ignore_ascii_case(term));
            if !found {
                errors.push(format!(
                    "tech term `{}` is in {} but never backtick-quoted in any file",
                    term, self.config.tech_files_dir,
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            bail!("Tech check failures:\n{}", errors.join("\n"))
        }
    }
}

impl crate::processors::ProductDiscovery for TechCheckProcessor {
    fn description(&self) -> &str {
        "Check that technical terms are backtick-quoted in markdown files"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        Path::new(&self.config.tech_files_dir).is_dir()
            && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn discover(
        &self,
        graph: &mut BuildGraph,
        file_index: &FileIndex,
    ) -> Result<()> {
        if !Path::new(&self.config.tech_files_dir).is_dir() {
            return Ok(());
        }
        // Collect all .txt files from tech_files_dir as extra inputs
        let mut extra_inputs = self.config.extra_inputs.clone();
        for entry in fs::read_dir(&self.config.tech_files_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "txt") {
                extra_inputs.push(path.to_string_lossy().into_owned());
            }
        }
        for ai in &self.config.auto_inputs {
            extra_inputs.extend(crate::processors::config_file_inputs(ai));
        }
        discover_checker_products(
            graph, &self.config.scan, file_index, &extra_inputs, &self.config,
            crate::processors::names::TECH_CHECK,
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        self.execute_product(product)
    }

    fn supports_batch(&self) -> bool {
        self.config.batch
    }

    fn execute_batch(&self, products: &[&Product]) -> Vec<Result<()>> {
        execute_checker_batch(products, |files| self.check_files(files))
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}

// --- Shared logic used by both the processor and the `tech fix` command ---

/// Load all technical terms from .txt files in the given directory.
/// Each file has one term per line.
pub fn load_terms(tech_files_dir: &str) -> Result<HashSet<String>> {
    let dir = Path::new(tech_files_dir);
    if !dir.is_dir() {
        bail!("tech_files_dir `{}` does not exist or is not a directory", tech_files_dir);
    }
    let mut terms = HashSet::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "txt") {
            let content = fs::read_to_string(&path)?;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    terms.insert(line.to_string());
                }
            }
        }
    }
    Ok(terms)
}

/// Sort terms longest-first for greedy matching (so "Android Studio" matches before "Android").
fn sorted_terms(terms: &HashSet<String>) -> Vec<&str> {
    let mut sorted: Vec<&str> = terms.iter().map(|s| s.as_str()).collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.len()));
    sorted
}

// --- Text analysis helpers ---

/// Find ranges in the text that are inside fenced code blocks (``` ... ```)
fn fenced_code_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut pos = 0;
    let bytes = text.as_bytes();
    while pos < bytes.len() {
        if bytes[pos] == b'`' && pos + 2 < bytes.len() && bytes[pos + 1] == b'`' && bytes[pos + 2] == b'`' {
            let start = pos;
            pos += 3;
            while pos < bytes.len() && bytes[pos] == b'`' {
                pos += 1;
            }
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            loop {
                if pos >= bytes.len() {
                    ranges.push((start, bytes.len()));
                    break;
                }
                if bytes[pos] == b'\n' || pos == 0 {
                    let line_start = if bytes[pos] == b'\n' { pos + 1 } else { pos };
                    if line_start + 2 < bytes.len()
                        && bytes[line_start] == b'`'
                        && bytes[line_start + 1] == b'`'
                        && bytes[line_start + 2] == b'`'
                    {
                        let mut end = line_start + 3;
                        while end < bytes.len() && bytes[end] != b'\n' {
                            end += 1;
                        }
                        if end < bytes.len() {
                            end += 1;
                        }
                        ranges.push((start, end));
                        pos = end;
                        break;
                    }
                }
                pos += 1;
            }
        } else {
            pos += 1;
        }
    }
    ranges
}

/// Find all backtick span ranges (start, end) in text, excluding fenced code blocks.
/// Returns positions of the opening and closing backtick (inclusive of backticks).
fn backtick_span_ranges(text: &str, fenced: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Skip fenced code blocks
        let mut in_fenced = false;
        for &(fs, fe) in fenced {
            if i >= fs && i < fe {
                i = fe;
                in_fenced = true;
                break;
            }
        }
        if in_fenced {
            continue;
        }
        if bytes[i] == b'`' {
            // Find matching closing backtick
            let open = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'`' && bytes[i] != b'\n' {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'`' && i > open + 1 {
                spans.push((open, i + 1)); // include both backticks
            }
            if i < bytes.len() {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    spans
}

/// Check if a byte position is inside any of the given ranges.
fn inside_ranges(pos: usize, end: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|&(s, e)| pos >= s && end <= e)
}

/// Check if the character at a byte position is a word-boundary character.
/// A term match is valid if the characters immediately before and after it
/// are not alphanumeric (or the match is at the start/end of text).
fn is_word_boundary(text: &[u8], pos: usize) -> bool {
    if pos >= text.len() {
        return true;
    }
    let ch = text[pos];
    // Not a word character: anything that's not alphanumeric or underscore
    !ch.is_ascii_alphanumeric() && ch != b'_'
}

/// Case-insensitive substring search. Returns byte offset of the match, or None.
fn find_case_insensitive(haystack: &str, needle: &str, start: usize) -> Option<usize> {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || start + n.len() > h.len() {
        return None;
    }
    for i in start..=(h.len() - n.len()) {
        if h[i..i + n.len()].eq_ignore_ascii_case(n) {
            return Some(i);
        }
    }
    None
}

/// Find all occurrences of a term in text (case-insensitive, word-boundary).
/// Returns (start, end) byte positions for each match.
fn find_term_occurrences(text: &str, term: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut results = Vec::new();
    let mut pos = 0;
    while let Some(start) = find_case_insensitive(text, term, pos) {
        let end = start + term.len();
        // Check word boundaries
        let before_ok = start == 0 || is_word_boundary(bytes, start - 1);
        let after_ok = end >= bytes.len() || is_word_boundary(bytes, end);
        if before_ok && after_ok {
            results.push((start, end));
        }
        pos = start + 1;
    }
    results
}

// --- Core check/fix logic ---

/// Find unquoted technical terms in a markdown file's content.
/// Returns (line_number, term) pairs.
pub fn find_unquoted_terms(content: &str, sorted_terms: &[&str]) -> Vec<(usize, String)> {
    let fenced = fenced_code_ranges(content);
    let backtick_spans = backtick_span_ranges(content, &fenced);
    // Track which byte positions are already claimed by a longer term match
    let mut claimed = Vec::new();
    let mut results = Vec::new();

    for &term in sorted_terms {
        for (start, end) in find_term_occurrences(content, term) {
            // Skip if inside fenced code block
            if inside_ranges(start, end, &fenced) {
                continue;
            }
            // Skip if inside a backtick span
            if inside_ranges(start, end, &backtick_spans) {
                continue;
            }
            // Skip if overlapping with an already-claimed longer term
            if claimed.iter().any(|&(cs, ce): &(usize, usize)| {
                start < ce && end > cs
            }) {
                continue;
            }
            claimed.push((start, end));
            let line_num = content[..start].matches('\n').count() + 1;
            results.push((line_num, term.to_string()));
        }
    }
    results
}

/// Split a backticked string into individual terms.
/// Handles comma-separated lists like "sed, awk" and word separators "and"/"or".
fn split_backticked(inner: &str) -> Vec<String> {
    let mut results = Vec::new();
    for part in inner.split(',') {
        for tok in part.split(" and ").flat_map(|s| s.split(" or ")) {
            let trimmed = tok.trim();
            if !trimmed.is_empty() {
                results.push(trimmed.to_string());
            }
        }
    }
    results
}

/// Check if a backticked string looks like a term reference (not arbitrary inline code).
fn looks_like_term_reference(inner: &str) -> bool {
    let parts = split_backticked(inner);
    if parts.is_empty() {
        return false;
    }
    let code_chars = ['(', ')', '{', '}', '[', ']', ';', '=', '>', '<', '|', '\\', '"', '\''];
    for part in &parts {
        if part.contains(' ') || part.chars().any(|c| code_chars.contains(&c)) {
            return false;
        }
    }
    true
}

/// Extract all backtick-quoted terms from a markdown file's content,
/// excluding content inside fenced code blocks.
/// Handles grouped terms like `sed, awk` by splitting them.
pub fn find_backticked_terms(content: &str) -> HashSet<String> {
    let fenced = fenced_code_ranges(content);
    let spans = backtick_span_ranges(content, &fenced);
    let mut terms = HashSet::new();
    for (start, end) in &spans {
        let inner = &content[start + 1..end - 1];
        if !looks_like_term_reference(inner) {
            continue;
        }
        for term in split_backticked(inner) {
            terms.insert(term);
        }
    }
    terms
}

/// Find unquoted term positions (byte offsets) for the fix command.
/// Returns (start, end, term_text) sorted longest-first, non-overlapping.
fn find_unquoted_positions(content: &str, sorted_terms: &[&str]) -> Vec<(usize, usize, String)> {
    let fenced = fenced_code_ranges(content);
    let backtick_spans = backtick_span_ranges(content, &fenced);
    let mut claimed: Vec<(usize, usize)> = Vec::new();
    let mut results = Vec::new();

    for &term in sorted_terms {
        for (start, end) in find_term_occurrences(content, term) {
            if inside_ranges(start, end, &fenced) {
                continue;
            }
            if inside_ranges(start, end, &backtick_spans) {
                continue;
            }
            if claimed.iter().any(|&(cs, ce)| start < ce && end > cs) {
                continue;
            }
            claimed.push((start, end));
            results.push((start, end, content[start..end].to_string()));
        }
    }
    results
}

/// Find backtick-quoted terms that are NOT in the tech term list.
/// Only considers spans that look like term references, not arbitrary inline code.
fn find_non_tech_backticked_positions(content: &str, terms: &HashSet<String>) -> Vec<(usize, usize)> {
    let fenced = fenced_code_ranges(content);
    let spans = backtick_span_ranges(content, &fenced);
    let mut results = Vec::new();
    for &(start, end) in &spans {
        let inner = &content[start + 1..end - 1];
        if !looks_like_term_reference(inner) {
            continue;
        }
        let parts = split_backticked(inner);
        let all_non_tech = parts.iter().all(|p| {
            !terms.iter().any(|t| t.eq_ignore_ascii_case(p))
        });
        if all_non_tech {
            results.push((start, end));
        }
    }
    results
}

/// Apply edits to text, right-to-left. Edits must not overlap.
fn apply_edits(content: &str, edits: &mut Vec<(usize, usize, String)>) -> String {
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    edits.dedup_by(|a, b| a.1 > b.0);

    let mut result = content.to_string();
    for (start, end, replacement) in edits.iter() {
        result = format!("{}{}{}", &result[..*start], replacement, &result[*end..]);
    }
    result
}

/// Auto-fix a single markdown file: add backticks to unquoted tech terms,
/// remove backticks from non-tech backticked terms.
/// Returns true if the file was modified.
pub fn fix_file(path: &Path, terms: &HashSet<String>, sorted_terms: &[&str]) -> Result<bool> {
    let original = fs::read_to_string(path)?;

    // Step 1: remove backticks from non-tech terms (e.g. `CI`/`CD` → CI/CD)
    let mut removals: Vec<(usize, usize, String)> = find_non_tech_backticked_positions(&original, terms)
        .into_iter()
        .map(|(s, e)| (s, e, original[s + 1..e - 1].to_string()))
        .collect();
    let cleaned = if removals.is_empty() {
        original.clone()
    } else {
        apply_edits(&original, &mut removals)
    };

    // Step 2: add backticks to unquoted terms (on the cleaned text, so CI/CD is now found)
    let mut additions: Vec<(usize, usize, String)> = find_unquoted_positions(&cleaned, sorted_terms)
        .into_iter()
        .map(|(s, e, m)| (s, e, format!("`{}`", m)))
        .collect();
    let result = if additions.is_empty() {
        cleaned
    } else {
        apply_edits(&cleaned, &mut additions)
    };

    if result != original {
        fs::write(path, &result)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Fix all markdown files: called by `rsconstruct tech fix`.
/// Uses the same scan config as the tech_check processor to find files.
pub fn fix_all(config: &TechCheckConfig) -> Result<()> {
    let terms = load_terms(&config.tech_files_dir)?;
    if terms.is_empty() {
        println!("No technical terms found in {}", config.tech_files_dir);
        return Ok(());
    }
    let sorted = sorted_terms(&terms);

    let file_index = FileIndex::build()?;
    let md_files = file_index.scan(&config.scan, true);

    if md_files.is_empty() {
        println!("No markdown files found");
        return Ok(());
    }

    println!("Checking {} markdown files against {} tech terms...", md_files.len(), terms.len());

    let mut modified_count = 0;
    for file in &md_files {
        if fix_file(file, &terms, &sorted)? {
            modified_count += 1;
            println!("  Fixed: {}", file.display());
        }
    }

    println!("Done. Modified {} of {} files.", modified_count, md_files.len());
    Ok(())
}
