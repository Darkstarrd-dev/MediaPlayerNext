use super::ZipDirectoryEntry;
use crate::CandidateMediaKind;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
enum SortToken {
    Text(String),
    Number(u64),
}

pub(crate) fn sort_archive_pages(entries: &[ZipDirectoryEntry]) -> Vec<ZipDirectoryEntry> {
    let mut pages: Vec<_> = entries
        .iter()
        .filter(|item| item.kind == CandidateMediaKind::Image)
        .cloned()
        .collect();
    pages.sort_by(|left, right| natural_cmp(&left.entry_path, &right.entry_path));
    pages
}

fn natural_cmp(left: &str, right: &str) -> Ordering {
    let left_tokens = tokenize(left);
    let right_tokens = tokenize(right);

    for (left_token, right_token) in left_tokens.iter().zip(right_tokens.iter()) {
        let ordering = match (left_token, right_token) {
            (SortToken::Number(left_value), SortToken::Number(right_value)) => {
                left_value.cmp(right_value)
            }
            (SortToken::Text(left_value), SortToken::Text(right_value)) => {
                left_value.cmp(right_value)
            }
            (SortToken::Number(_), SortToken::Text(_)) => Ordering::Less,
            (SortToken::Text(_), SortToken::Number(_)) => Ordering::Greater,
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left_tokens.len().cmp(&right_tokens.len())
}

fn tokenize(value: &str) -> Vec<SortToken> {
    let mut tokens = Vec::new();
    let mut text = String::new();
    let mut number = String::new();

    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_digit() {
            if !text.is_empty() {
                tokens.push(SortToken::Text(std::mem::take(&mut text)));
            }
            number.push(ch);
        } else {
            if !number.is_empty() {
                tokens.push(SortToken::Number(number.parse::<u64>().unwrap_or_default()));
                number.clear();
            }
            text.push(ch);
        }
    }

    if !number.is_empty() {
        tokens.push(SortToken::Number(number.parse::<u64>().unwrap_or_default()));
    }
    if !text.is_empty() {
        tokens.push(SortToken::Text(text));
    }

    tokens
}
