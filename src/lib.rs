// src/lib.rs
//! Fixerr - A Library for Repairing Fragmented CSV Records
//!
//! This library provides functionality to reconstruct CSV files that have been
//! improperly formatted, particularly those with line breaks within quoted fields
//! that span multiple physical lines in the file.
//!
//! ---
//! **Development Note**
//!
//! Core logic, algorithms, and architecture were fully designed and implemented by **Nicholas Gurgenidze**.
//! AI assistance was used mostly for non-functional improvements, including:
//!
//! - Display parts of UI boilerplate  
//! - README drafting and formatting  
//! - Regular and documentation comments
//! - Minor safe refactorings
//! ---
//!
//! # Example
//!
//! ```no_run
//! use fixerr::{reconstruct_records, write_output_csv, HeaderMode, Delimiter, Stats};
//!
//! let mut stats = Stats::default();
//! let records = reconstruct_records(
//!     "input.csv",
//!     HeaderMode::HasHeaders,
//!     Delimiter::Comma,
//!     &mut stats
//! ).unwrap();
//!
//! write_output_csv("output.csv", &records, Delimiter::Comma).unwrap();
//! println!("Processed {} rows, fixed {} rows", stats.total_rows, stats.fixed_rows);
//! ```
mod engine;

// Re-export public API
pub use engine::{
    reconstruct_records,
    write_output_csv,
    build_csv_reader,
    HeaderMode,
    Delimiter,
    Stats,
};
