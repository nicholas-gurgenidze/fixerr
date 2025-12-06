// src/engine.rs
//! Core CSV processing engine
//!
//! Contains the main logic for reconstructing malformed CSV records.
//!
//! # Architecture Note
//! This module handles the "business logic" of the application. It is designed to be
//! decoupled from the UI, allowing for future integration into other frontends (e.g., WebAssembly).

use csv::{Reader, ReaderBuilder, StringRecord, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

// ============================================
// Public Types
// ============================================

/// Header mode for CSV files
#[derive(Default, Debug, Clone, Copy)]
pub enum HeaderMode {
    /// File has a header row (default)
    #[default]
    HasHeaders,
    /// File has no header row
    NoHeaders,
}

impl HeaderMode {
    /// Convert to boolean for CSV reader
    pub fn as_bool(&self) -> bool {
        matches!(self, HeaderMode::HasHeaders)
    }
}

/// Delimiter character for CSV files
#[derive(Default, Debug, Clone, Copy)]
pub enum Delimiter {
    /// Comma separator (default)
    #[default]
    Comma,
    /// Semicolon separator
    Semicolon,
    /// Tab separator
    Tab,
    /// Pipe separator
    Pipe,
}

impl Delimiter {
    /// Convert to byte for CSV reader/writer
    pub fn as_byte(&self) -> u8 {
        match self {
            Delimiter::Comma => b',',
            Delimiter::Semicolon => b';',
            Delimiter::Tab => b'\t',
            Delimiter::Pipe => b'|',
        }
    }
}

/// Statistics about CSV processing
#[derive(Default, Debug)]
pub struct Stats {
    /// Total physical rows read from file
    pub total_rows: usize,
    /// Number of rows that were reconstructed from multiple physical rows
    pub fixed_rows: usize,
    /// Number of rows that couldn't be reconstructed and were discarded
    pub removed_rows: usize,
}

// ============================================
// Public API Functions
// ============================================

/// Build a configured CSV reader
pub fn build_csv_reader<R: std::io::Read>(
    reader: R,
    header_mode: HeaderMode,
    delimiter: Delimiter,
) -> Reader<R> {
    ReaderBuilder::new()
        .has_headers(header_mode.as_bool())
        .delimiter(delimiter.as_byte())
        .flexible(true) // Allow varying column counts to handle broken rows
        .from_reader(reader)
}

/// Reconstruct malformed CSV records into proper format
///
/// This function reads a CSV file that may have malformed records (e.g., records
/// split across multiple physical lines due to embedded newlines) and reconstructs
/// them into proper CSV records.
pub fn reconstruct_records(
    file_path: &str,
    header_mode: HeaderMode,
    delimiter: Delimiter,
    stats: &mut Stats,
) -> Result<Vec<StringRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = build_csv_reader(file, header_mode, delimiter);

    // Detect expected column count
    let (expected_columns, maybe_headers) = detect_column_count(&mut reader, header_mode)?;

    let mut logical_rows: Vec<StringRecord> = Vec::new();

    // Add headers to output if present
    if let Some(h) = maybe_headers {
        logical_rows.push(h);
    }

    // Buffer for accumulating fields across multiple physical rows
    let mut buffer: Vec<String> = Vec::new();

    for result in reader.records() {
        stats.total_rows += 1;
        let record = result?;
        let rec_len = record.len();

        // Check: Immediate Over-Length Check
        //
        // If the physical row itself has more columns than expected, it is 
        // statistically impossible for it to be a valid part of a split record 
        // (which should be shorter) or a valid full record. Discard immediately.
        if rec_len > expected_columns {
            stats.removed_rows += 1;
            continue;
        }

        // Case 1: Starting a new logical row
        if buffer.is_empty() {
            if rec_len == expected_columns {
                // Complete row - add directly
                logical_rows.push(record);
            } else {
                // Incomplete row - start buffering
                buffer.extend(record.iter().map(|s| s.to_string()));
            }
            continue;
        }

        // Case 2: Continuing a buffered row
        // Append first field to last buffered field (handles embedded newlines)
        if let Some(first_part) = record.get(0) {
            if let Some(last_col) = buffer.last_mut() {
                if !last_col.is_empty() {
                    // DESIGN DECISION: Preserve the newline in the in-memory representation.
                    // We maintain the data fidelity here (stitching exactly as it was broken).
                    // Sanitization is deferred to the writing phase to separate concerns.
                    last_col.push('\n'); 
                }
                last_col.push_str(first_part);
            }
        }

        // Append remaining fields
        for i in 1..rec_len {
            buffer.push(record.get(i).unwrap_or("").to_string());
        }

        // Case 3: Check if row is now complete
        if buffer.len() == expected_columns {
            logical_rows.push(StringRecord::from(buffer.clone()));
            stats.fixed_rows += 1;
            buffer.clear();
        } else if buffer.len() > expected_columns {
            // Row has too many columns - discard and log
            stats.removed_rows += 1;
            buffer.clear();
        }
    }

    // Handle any remaining incomplete row
    if !buffer.is_empty() {
        stats.removed_rows += 1;
    }

    Ok(logical_rows)
}

/// Write cleaned CSV records to output file
///
/// This function handles the final output generation. It applies whitespace
/// normalization to every field to ensure clean data.
pub fn write_output_csv(
    output_path: &str,
    rows: &[StringRecord],
    delimiter: Delimiter,
) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new()
        .delimiter(delimiter.as_byte())
        .from_path(output_path)?;

    for record in rows {
        // Apply cleaning logic to every field before writing
        let cleaned = record.iter().map(clean_and_normalize_field);
        writer.write_record(cleaned)?;
    }

    writer.flush()?;
    Ok(())
}

// ============================================
// Private Helper Functions
// ============================================

fn detect_column_count(
    reader: &mut Reader<File>,
    header_mode: HeaderMode,
) -> Result<(usize, Option<StringRecord>), Box<dyn Error>> {
    match header_mode {
        HeaderMode::HasHeaders => {
            let headers = reader.headers()?.clone();
            let col_count = headers.len();
            Ok((col_count, Some(headers)))
        }
        HeaderMode::NoHeaders => {
            print!("Enter expected number of columns: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let col_count = input.trim().parse::<usize>()?;

            Ok((col_count, None))
        }
    }
}

// DESIGN DECISION: Whitespace Normalization
// During the reconstruction process, joining split lines often results in "double spaces"
// (one original trailing space + one space replacing the newline).
//
// Instead of a simple .replace('\n', " "), we use full tokenization via split_whitespace().
// This automatically:
// 1. Trims leading/trailing whitespace (common artifact of manual data entry).
// 2. Collapses multiple internal spaces into a single space.
// 3. Flattens newlines and tabs.
fn clean_and_normalize_field(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

// ============================================
// Unit Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_case_1_trailing_newline_split() {
        // Case 1: Newline character is added at the end of field value.
        // Data: 941315,Tbilisi Waters,Georgian Product \n ,1722.63
        let filename = "test_case_1.csv";
        let content = "ID,Organization,Details,Amount\n9413154,Tbilisi Waters,Georgian Product\n,1722.63";
        
        {
            let mut file = File::create(filename).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let mut stats = Stats::default();
        let result = reconstruct_records(filename, HeaderMode::HasHeaders, Delimiter::Comma, &mut stats).unwrap();
        let _ = fs::remove_file(filename);

        assert_eq!(result.len(), 2); 
        
        // Internal memory check: Should have "Georgian Product\n"
        let stitched_details = &result[1][2]; 
        assert!(stitched_details.contains("Georgian Product"));
        assert!(stitched_details.contains('\n'));

        // Normalization check: "Georgian Product" (Trimmed)
        assert_eq!(clean_and_normalize_field(stitched_details), "Georgian Product");
    }

    #[test]
    fn test_case_2_mid_value_split() {
        // Case 2: Field value is split in the middle.
        // Data: 9413159,Bodorna Waters,Mineral water from \n Bodorna, 2909.20
        let filename = "test_case_2.csv";
        let content = "ID,Organization,Details,Amount\n9413155,Bodorna Waters,Mineral water from\nBodorna, 2909.20";
        
        {
            let mut file = File::create(filename).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let mut stats = Stats::default();
        let result = reconstruct_records(filename, HeaderMode::HasHeaders, Delimiter::Comma, &mut stats).unwrap();
        let _ = fs::remove_file(filename);

        assert_eq!(result.len(), 2);
        
        // Normalization check: "Mineral water from Bodorna"
        // Note: The source has "Mineral water from". The split is after "from".
        // The algorithm will stitch "from\nBodorna".
        // Normalizer turns "from \n Bodorna" into "from Bodorna".
        let details = &result[1][2];
        assert_eq!(clean_and_normalize_field(details), "Mineral water from Bodorna");
    }

    #[test]
    fn test_case_3_multi_column_cascading_split() {
        // Case 3: Multiple field values are split in the middle (two field values).
        // This creates a "staircase" effect across rows.
        // Row 1: 9413157,Gori
        // Row 2: Beverages,Product from
        // Row 3: Gori, 3427.50
        let filename = "test_case_3.csv";
        let content = "ID,Organization,Details,Amount\n9413156,Gori\nBeverages,Product from\nGori, 3427.50";
        
        {
            let mut file = File::create(filename).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let mut stats = Stats::default();
        let result = reconstruct_records(filename, HeaderMode::HasHeaders, Delimiter::Comma, &mut stats).unwrap();
        let _ = fs::remove_file(filename);

        assert_eq!(result.len(), 2);
        
        // Verify Field 1 (Org): "Gori\nBeverages"
        let org = &result[1][1];
        assert_eq!(clean_and_normalize_field(org), "Gori Beverages");

        // Verify Field 2 (Details): "Product from\nGori"
        let details = &result[1][2];
        assert_eq!(clean_and_normalize_field(details), "Product from Gori");
    }

    #[test]
    fn test_case_4_single_field_multi_row_fragmentation() {
        // Case 4: One field value is split on multiple rows.
        // Data: 9413156,Sairme Waters,This \n Product \n Is from \n Sarime,1736.10
        let filename = "test_case_4.csv";
        let content = "ID,Organization,Details,Amount\n9413157,Sairme Waters,This\nProduct\nIs from\nSarime,1736.10";
        
        {
            let mut file = File::create(filename).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let mut stats = Stats::default();
        let result = reconstruct_records(filename, HeaderMode::HasHeaders, Delimiter::Comma, &mut stats).unwrap();
        let _ = fs::remove_file(filename);
        
        assert_eq!(result.len(), 2);
        
        // Normalization check: "This Product Is from Sarime"
        let details = &result[1][2];
        assert_eq!(clean_and_normalize_field(details), "This Product Is from Sarime");
    }

    #[test]
    fn test_case_5_quoted_newline_flattening() {
        // Case 5: Quoted split (Technicaly valid CSV, but we flatten it for legacy systems)
        // Data: 9413156,Svaneti Waters,"Mestia, \n Georgia",2505.25
        let filename = "test_case_5.csv";
        let content = "ID,Organization,Details,Amount\n9413158,Svaneti Waters,\"Mestia,\nGeorgia\",2505.25";
        
        {
            let mut file = File::create(filename).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let mut stats = Stats::default();
        let result = reconstruct_records(filename, HeaderMode::HasHeaders, Delimiter::Comma, &mut stats).unwrap();
        let _ = fs::remove_file(filename);
        
        assert_eq!(result.len(), 2);
        
        // Normalization check: "Mestia, Georgia"
        let details = &result[1][2];
        assert_eq!(clean_and_normalize_field(details), "Mestia, Georgia");
    }

    #[test]
    fn test_clean_and_normalize_logic() {
        assert_eq!(clean_and_normalize_field("Word \n"), "Word");
        assert_eq!(clean_and_normalize_field("Hello  World"), "Hello World");
        assert_eq!(clean_and_normalize_field(" Item \t 1 "), "Item 1");
    }
}
