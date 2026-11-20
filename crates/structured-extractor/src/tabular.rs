//! # Tabular Document Parser and Normalizer (CSV/TSV)
//!
//! Robust RFC 4180 parsing, automated delimiter sniffing (comma, tab, semicolon, pipe),
//! header sniffing, column type inference, and tabular statistics profiling.

use std::collections::HashSet;

use crate::error::{Result, StructuredError};
use crate::models::{
    ColumnDataType, ColumnStats, StructuredOptions, TabularDelimiter, TabularTable,
};

/// Sniffs the most likely delimiter from raw text by testing candidate delimiters
/// and analyzing row column consistency across sample lines.
pub fn sniff_delimiter(text: &str) -> TabularDelimiter {
    let candidates = [
        TabularDelimiter::Comma,
        TabularDelimiter::Tab,
        TabularDelimiter::Semicolon,
        TabularDelimiter::Pipe,
    ];

    let sample_lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(15)
        .collect();

    if sample_lines.is_empty() {
        return TabularDelimiter::Comma;
    }

    let mut best_delimiter = TabularDelimiter::Comma;
    let mut best_score = -1.0;

    for &delim in &candidates {
        let ch = delim.as_char();
        let counts: Vec<usize> = sample_lines
            .iter()
            .map(|line| count_delimited_fields(line, ch))
            .collect();

        // If delimiter produces only 1 column everywhere, it's not a valid delimiter
        if counts.iter().all(|&c| c <= 1) {
            continue;
        }

        let avg = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
        let variance = counts
            .iter()
            .map(|&c| (c as f64 - avg).powi(2))
            .sum::<f64>()
            / counts.len() as f64;

        // Consistency score: reward higher column counts with zero or near-zero variance
        let consistency = if variance == 0.0 {
            avg * 10.0
        } else {
            avg / (1.0 + variance)
        };

        if consistency > best_score {
            best_score = consistency;
            best_delimiter = delim;
        }
    }

    best_delimiter
}

/// Helper counting fields in a single line while respecting quotes.
fn count_delimited_fields(line: &str, delim: char) -> usize {
    let mut count = 1;
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == delim && !in_quotes {
            count += 1;
        }
        i += 1;
    }

    count
}

/// Parses raw text into tabular rows using an RFC 4180 streaming state machine.
///
/// Returns `(extracted_rows, total_discovered_rows)`.
pub fn parse_records_raw(
    text: &str,
    delimiter: char,
    options: &StructuredOptions,
) -> Result<(Vec<Vec<String>>, usize)> {
    let mut extracted_rows = Vec::new();
    let mut total_rows = 0;
    let raw_row_limit = options.limits.max_tabular_rows.saturating_add(1);

    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    // Escaped double-quote: '""' inside quotes -> single '"'
                    chars.next();
                    if current_cell.len() + 1 > options.limits.max_cell_bytes {
                        return Err(StructuredError::LimitExceeded(format!(
                            "Cell size exceeds maximum allowed limit of {} bytes",
                            options.limits.max_cell_bytes
                        )));
                    }
                    current_cell.push('"');
                } else {
                    // Ending quote
                    in_quotes = false;
                }
            } else {
                if current_cell.len() + 1 > options.limits.max_cell_bytes {
                    return Err(StructuredError::LimitExceeded(format!(
                        "Cell size exceeds maximum allowed limit of {} bytes",
                        options.limits.max_cell_bytes
                    )));
                }
                current_cell.push(c);
            }
        } else if c == '"' {
            // Starting quote
            in_quotes = true;
        } else if c == delimiter {
            // Field separator
            current_row.push(current_cell);
            current_cell = String::new();
        } else if c == '\r' {
            // Check for \r\n
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            current_row.push(current_cell);
            current_cell = String::new();

            if !is_row_empty(&current_row) {
                total_rows += 1;
                if total_rows <= raw_row_limit {
                    extracted_rows.push(current_row);
                } else if !options.limits.truncate_on_row_limit {
                    return Err(StructuredError::LimitExceeded(format!(
                        "Row count exceeds maximum tabular limit of {} rows",
                        options.limits.max_tabular_rows
                    )));
                }
            }
            current_row = Vec::new();
        } else if c == '\n' {
            // LF line break
            current_row.push(current_cell);
            current_cell = String::new();

            if !is_row_empty(&current_row) {
                total_rows += 1;
                if total_rows <= raw_row_limit {
                    extracted_rows.push(current_row);
                } else if !options.limits.truncate_on_row_limit {
                    return Err(StructuredError::LimitExceeded(format!(
                        "Row count exceeds maximum tabular limit of {} rows",
                        options.limits.max_tabular_rows
                    )));
                }
            }
            current_row = Vec::new();
        } else {
            if current_cell.len() + 1 > options.limits.max_cell_bytes {
                return Err(StructuredError::LimitExceeded(format!(
                    "Cell size exceeds maximum allowed limit of {} bytes",
                    options.limits.max_cell_bytes
                )));
            }
            current_cell.push(c);
        }
    }

    // Process final cell and row if non-empty
    current_row.push(current_cell);
    if !is_row_empty(&current_row) {
        total_rows += 1;
        if total_rows <= raw_row_limit {
            extracted_rows.push(current_row);
        } else if !options.limits.truncate_on_row_limit {
            return Err(StructuredError::LimitExceeded(format!(
                "Row count exceeds maximum tabular limit of {} rows",
                options.limits.max_tabular_rows
            )));
        }
    }

    Ok((extracted_rows, total_rows))
}

/// Checks whether an entire row consists exclusively of empty string cells.
fn is_row_empty(row: &[String]) -> bool {
    row.iter().all(|c| c.trim().is_empty())
}

/// Sniffs whether the first row of parsed data represents headers rather than data.
pub fn sniff_has_headers(raw_rows: &[Vec<String>]) -> bool {
    if raw_rows.len() < 2 {
        return false;
    }

    let header_candidate = &raw_rows[0];
    let data_candidates = &raw_rows[1..raw_rows.len().min(10)];

    // If first row has fewer than 1 column, it cannot be headers
    if header_candidate.is_empty() {
        return false;
    }

    // 1. If any cell in row 0 is pure integer or float, it is likely data, not headers
    let row0_has_numbers = header_candidate.iter().any(|c| {
        let trimmed = c.trim();
        !trimmed.is_empty() && (trimmed.parse::<i64>().is_ok() || trimmed.parse::<f64>().is_ok())
    });

    if row0_has_numbers {
        return false;
    }

    // 2. Type disparity: compare types of row 0 vs subsequent rows
    let mut type_disparity_score = 0;
    for col_idx in 0..header_candidate.len() {
        let col_data_has_numbers = data_candidates.iter().any(|row| {
            if let Some(cell) = row.get(col_idx) {
                let trimmed = cell.trim();
                !trimmed.is_empty()
                    && (trimmed.parse::<i64>().is_ok()
                        || trimmed.parse::<f64>().is_ok()
                        || is_likely_date(trimmed))
            } else {
                false
            }
        });

        if col_data_has_numbers {
            type_disparity_score += 1;
        }
    }

    if type_disparity_score > 0 {
        return true;
    }

    // 3. Uniqueness and non-empty check of row 0
    let distinct_headers: HashSet<&str> = header_candidate.iter().map(|s| s.trim()).collect();
    if distinct_headers.len() == header_candidate.len()
        && header_candidate.iter().all(|s| !s.trim().is_empty())
    {
        // Check for common header naming keywords
        let common_keywords = [
            "id",
            "name",
            "title",
            "type",
            "status",
            "date",
            "time",
            "user",
            "email",
            "description",
            "amount",
            "price",
            "total",
            "count",
            "code",
            "category",
            "value",
            "val",
            "col",
            "column",
            "field",
            "item",
            "key",
            "label",
            "data",
            "sku",
            "created",
            "updated",
        ];

        let keyword_match = header_candidate.iter().any(|s| {
            let lower = s.to_lowercase();
            common_keywords.iter().any(|&kw| lower.contains(kw))
        });

        if keyword_match {
            return true;
        }

        // If all row 0 tokens are clean identifier-like words without whitespace
        let all_identifiers = header_candidate.iter().all(|s| {
            let trimmed = s.trim();
            !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        });

        if all_identifiers {
            return true;
        }
    }

    false
}

/// Checks whether a string value matches a calendar date or ISO 8601 timestamp.
fn is_likely_date(val: &str) -> bool {
    let bytes = val.as_bytes();
    // YYYY-MM-DD
    if bytes.len() >= 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
    {
        return true;
    }

    // MM/DD/YYYY or DD/MM/YYYY
    if bytes.len() >= 10
        && (bytes[2] == b'/' || bytes[2] == b'-')
        && (bytes[5] == b'/' || bytes[5] == b'-')
        && bytes[0..2].iter().all(u8::is_ascii_digit)
        && bytes[3..5].iter().all(u8::is_ascii_digit)
        && bytes[6..10].iter().all(u8::is_ascii_digit)
    {
        return true;
    }

    false
}

/// Infers the column data type from non-empty string values.
pub fn infer_column_type(values: &[&str]) -> ColumnDataType {
    if values.is_empty() {
        return ColumnDataType::String;
    }

    let mut all_int = true;
    let mut all_float = true;
    let mut all_bool = true;
    let mut all_date = true;

    for &v in values {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Integer
        if all_int && trimmed.parse::<i64>().is_err() {
            all_int = false;
        }

        // Float (must parse as f64 and have decimal point or scientific notation)
        if all_float
            && (trimmed.parse::<f64>().is_err()
                || (!trimmed.contains('.') && !trimmed.contains('e') && !trimmed.contains('E')))
        {
            all_float = false;
        }

        // Boolean
        if all_bool {
            let lower = trimmed.to_lowercase();
            if lower != "true"
                && lower != "false"
                && lower != "yes"
                && lower != "no"
                && lower != "t"
                && lower != "f"
                && lower != "1"
                && lower != "0"
            {
                all_bool = false;
            }
        }

        // Date
        if all_date && !is_likely_date(trimmed) {
            all_date = false;
        }
    }

    if all_int {
        ColumnDataType::Integer
    } else if all_float {
        ColumnDataType::Float
    } else if all_bool {
        ColumnDataType::Boolean
    } else if all_date {
        ColumnDataType::DateTime
    } else {
        ColumnDataType::String
    }
}

/// Normalizes an individual cell according to whitespace and null representation rules.
pub fn normalize_cell(cell: &str, options: &StructuredOptions) -> String {
    let mut val = if options.tabular_trim_whitespace {
        cell.trim()
    } else {
        cell
    };

    if options.tabular_normalize_nulls {
        let lower = val.to_lowercase();
        if matches!(
            lower.as_str(),
            "" | "na"
                | "n/a"
                | "null"
                | "none"
                | "nan"
                | "-"
                | "--"
                | "nil"
                | "<null>"
                | "#n/a"
                | "undefined"
        ) {
            val = "";
        }
    }

    val.to_string()
}

/// Parses and normalizes a delimited tabular text stream into a structured `TabularTable`.
pub fn parse_tabular(
    text: &str,
    explicit_delimiter: Option<TabularDelimiter>,
    name: Option<String>,
    options: &StructuredOptions,
) -> Result<TabularTable> {
    if text.trim().is_empty() {
        return Err(StructuredError::UnrecognizedFormat(
            "Input tabular content is empty".to_string(),
        ));
    }

    let delimiter = explicit_delimiter.unwrap_or_else(|| sniff_delimiter(text));
    let (raw_rows, total_rows) = parse_records_raw(text, delimiter.as_char(), options)?;

    if raw_rows.is_empty() {
        return Err(StructuredError::UnrecognizedFormat(
            "No tabular records could be extracted from input".to_string(),
        ));
    }

    let has_headers = sniff_has_headers(&raw_rows);

    let (raw_headers, raw_data_rows, total_data_rows) = if has_headers {
        let headers = raw_rows[0].clone();
        let mut data = raw_rows[1..].to_vec();
        if data.len() > options.limits.max_tabular_rows {
            data.truncate(options.limits.max_tabular_rows);
        }
        (headers, data, total_rows.saturating_sub(1))
    } else {
        // Synthesize column headers: column_1, column_2, ...
        let col_count = raw_rows.iter().map(Vec::len).max().unwrap_or(0);
        let headers = (1..=col_count).map(|i| format!("column_{i}")).collect();
        let mut data = raw_rows;
        if data.len() > options.limits.max_tabular_rows {
            data.truncate(options.limits.max_tabular_rows);
        }
        (headers, data, total_rows)
    };

    if !options.limits.truncate_on_row_limit && total_data_rows > options.limits.max_tabular_rows {
        return Err(StructuredError::LimitExceeded(format!(
            "Row count {} exceeds maximum tabular limit of {} rows",
            total_data_rows, options.limits.max_tabular_rows
        )));
    }

    let col_count = raw_headers
        .len()
        .max(raw_data_rows.iter().map(Vec::len).max().unwrap_or(0));

    if col_count > options.limits.max_tabular_columns {
        return Err(StructuredError::LimitExceeded(format!(
            "Column count {} exceeds maximum tabular limit of {} columns",
            col_count, options.limits.max_tabular_columns
        )));
    }

    // Normalize headers
    let mut headers = Vec::with_capacity(col_count);
    for idx in 0..col_count {
        let raw_h = raw_headers.get(idx).map(|s| s.as_str()).unwrap_or("");
        let normalized_h = if raw_h.trim().is_empty() {
            format!("column_{}", idx + 1)
        } else {
            raw_h.trim().to_string()
        };
        headers.push(normalized_h);
    }

    // Normalize and pad data rows
    let mut data_rows = Vec::with_capacity(raw_data_rows.len());
    for row in raw_data_rows {
        let mut normalized_row = Vec::with_capacity(col_count);
        for col_idx in 0..col_count {
            let cell = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            normalized_row.push(normalize_cell(cell, options));
        }
        data_rows.push(normalized_row);
    }

    // Compute column profiles and statistics
    let mut columns = Vec::with_capacity(col_count);
    for (col_idx, header) in headers.iter().enumerate() {
        let col_values: Vec<&str> = data_rows
            .iter()
            .map(|r| r[col_idx].as_str())
            .filter(|s| !s.is_empty())
            .collect();

        let data_type = if options.tabular_infer_types {
            infer_column_type(&col_values)
        } else {
            ColumnDataType::String
        };

        let non_empty_count = col_values.len();
        let null_count = data_rows.len().saturating_sub(non_empty_count);

        let mut unique_set = HashSet::new();
        let mut sample_values = Vec::new();
        let mut min_val: Option<String> = None;
        let mut max_val: Option<String> = None;

        for &val in &col_values {
            if unique_set.insert(val) && sample_values.len() < 5 {
                sample_values.push(val.to_string());
            }

            match &min_val {
                None => min_val = Some(val.to_string()),
                Some(current_min) if val < current_min.as_str() => min_val = Some(val.to_string()),
                _ => {}
            }

            match &max_val {
                None => max_val = Some(val.to_string()),
                Some(current_max) if val > current_max.as_str() => max_val = Some(val.to_string()),
                _ => {}
            }
        }

        columns.push(ColumnStats {
            index: col_idx,
            name: header.clone(),
            data_type,
            total_count: data_rows.len(),
            non_empty_count,
            null_count,
            unique_count: unique_set.len(),
            sample_values,
            min_value: min_val,
            max_value: max_val,
        });
    }

    let extracted_rows = data_rows.len();
    let truncated = total_data_rows > extracted_rows;

    Ok(TabularTable {
        name,
        delimiter,
        has_headers,
        headers,
        columns,
        rows: data_rows,
        total_rows: total_data_rows,
        extracted_rows,
        truncated,
    })
}
