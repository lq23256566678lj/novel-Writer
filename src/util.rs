use crate::model::INDENT;
use std::fs;
use std::path::{Path, PathBuf};

pub fn handle_newline_indent(old: &str, new: &str) -> String {
    let common = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count();

    let new_chars: Vec<char> = new.chars().collect();
    if common >= new_chars.len() {
        return new.to_string();
    }

    if new_chars[common] == '\n' {
        let mut result = String::with_capacity(new.len() + INDENT.len());
        for (i, ch) in new_chars.iter().enumerate() {
            result.push(*ch);
            if i == common {
                let next = new_chars.get(i + 1);
                match next {
                    Some('\n') | None => {}
                    Some('\u{3000}') => {}
                    _ => {
                        result.push_str(INDENT);
                    }
                }
            }
        }
        return result;
    }

    new.to_string()
}

pub fn add_indent_to_all_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for line in s.split_inclusive('\n') {
        let is_empty_line = line == "\n" || line.is_empty();
        if !is_empty_line && !line.starts_with('\u{3000}') {
            out.push_str(INDENT);
        }
        out.push_str(line);
    }
    out
}

pub fn collect_text_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                collect_text_files(&path, out);
            } else if path.extension().map_or(false, |e| e == "txt" || e == "md") {
                out.push(path);
            }
        }
    }
}