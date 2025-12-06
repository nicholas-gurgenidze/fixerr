// src/ui.rs
//! User interface module
//!
//! Handles all display output and user input for the application.
//! Separates presentation logic from business logic.

use crate::{Config, Stats};
use std::io::{self, Write};
use std::time::Instant;

// ============================================
// Display Functions
// ============================================

/// Display welcome screen with main menu
pub fn display_welcome() {
    clear_screen();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║                    FIXERR                        ║");
    println!("║        CSV Repair Utility for Georgian           ║");
    println!("║              Revenue Service Files               ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("📋 MAIN MENU");
    println!("────────────────────────────────────────────────────");
    println!("  1. Fix CSV Records");
    println!("  2. Settings");
    println!("  3. Exit");
    println!("────────────────────────────────────────────────────");
}

/// Display settings menu with current configuration
pub fn display_settings_menu(config: &Config) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║                   SETTINGS                       ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    
    println!("📋 Current Configuration:");
    println!("  Delimiter:    {:?}", config.delimiter);
    println!("  Header Mode:  {:?}", config.header_mode);
    println!("  Input File:   {}", config.input_file);
    println!("  Output File:  {}", config.output_file);
    
    println!("\n────────────────────────────────────────────────────");
    println!("  1. Change Delimiter");
    println!("  2. Change Header Mode");
    println!("  3. Change Input File Path");
    println!("  4. Change Output File Path");
    println!("  5. Reset to Defaults");
    println!("  6. Back to Main Menu");
    println!("────────────────────────────────────────────────────");
}

/// Display delimiter selection menu
pub fn display_delimiter_menu(current: &str) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║              CHANGE DELIMITER                    ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    
    println!("Current Delimiter: {current}");
    println!("\n📌 Available Delimiters:");
    println!("  1. Comma (,)");
    println!("  2. Semicolon (;)");
    println!("  3. Tab (\\t)");
    println!("  4. Pipe (|)");
    println!();
}

/// Display header mode selection menu
pub fn display_header_mode_menu(current: &str) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║            CHANGE HEADER MODE                    ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    
    println!("Current Mode: {current}");
    println!("\n📌 Header Mode Options:");
    println!("  1. Has Headers (first row is header)");
    println!("  2. No Headers (all rows are data)");
    println!();
}

/// Display file path change screen
pub fn display_file_path_screen(setting_name: &str, current_path: &str) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║          CHANGE {} FILE PATH{: <18}║", 
             setting_name.to_uppercase(),
             "");
    println!("╚══════════════════════════════════════════════════╝\n");
    
    println!("Current Path: {current_path}");
    println!();
}

/// Display processing header with configuration
pub fn display_processing_header(config: &Config) {
    clear_screen();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║              PROCESSING CSV FILE                 ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    
    println!("📁 Input File      : {}", config.input_file);
    println!("📁 Output File     : {}", config.output_file);
    println!("⚙️  Delimiter       : {:?}", config.delimiter);
    println!("⚙️  Header Mode     : {:?}", config.header_mode);
    println!("────────────────────────────────────────────────────\n");
}

/// Display processing summary with statistics
pub fn display_summary(stats: &Stats, total_records: usize, output_file: &str) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║                  SUMMARY                         ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!("📊 Total lines read       : {}", stats.total_rows);
    println!("✅ Fixed/Merged rows      : {}", stats.fixed_rows);
    println!("❌ Discarded rows         : {}", stats.removed_rows);
    println!("📝 Total valid records    : {total_records}");
    
    let success_rate = calculate_success_rate(stats);
    println!("📈 Success Rate           : {success_rate:.1}%");
    
    println!("────────────────────────────────────────────────────");
    println!("✨ Success! Output written to: {output_file}\n");
}

// ============================================
// Input Functions
// ============================================

/// Get and validate menu choice within a range
///
/// Loops until user provides valid input
///
/// # Arguments
/// * `min` - Minimum valid choice (inclusive)
/// * `max` - Maximum valid choice (inclusive)
/// * `prompt` - Prompt to display to user
///
/// # Returns
/// Valid choice within [min, max]
pub fn get_menu_choice(min: u32, max: u32, prompt: &str) -> io::Result<u32> {
    loop {
        print!("{prompt}");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().parse::<u32>() {
            Ok(choice) if choice >= min && choice <= max => {
                return Ok(choice);
            }
            Ok(choice) => {
                println!("\n❌ Invalid choice: {choice}. Please enter a number between {min} and {max}.\n");
            }
            Err(_) => {
                println!("\n❌ Invalid input. Please enter a number between {min} and {max}.\n");
            }
        }
    }
}

/// Get string input from user
///
/// # Arguments
/// * `prompt` - Prompt to display
///
/// # Returns
/// Trimmed string input
pub fn get_string_input(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

/// Get yes/no confirmation from user
///
/// # Arguments
/// * `prompt` - Question to ask user
///
/// # Returns
/// true if user confirms (y/yes), false otherwise
pub fn get_confirmation(prompt: &str) -> io::Result<bool> {
    print!("{prompt} (y/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().eq_ignore_ascii_case("y") || 
       input.trim().eq_ignore_ascii_case("yes"))
}

// ============================================
// Utility Functions
// ============================================

/// Clear terminal screen (cross-platform)
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Pause and wait for user to press Enter
pub fn pause() {
    print!("\nPress Enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

/// Print elapsed time in a formatted way
///
/// # Arguments
/// * `label` - Description of what was timed
/// * `start` - Starting Instant
pub fn print_elapsed(label: &str, start: Instant) {
    let duration = start.elapsed();
    let total_secs = duration.as_secs();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    let millis = duration.as_millis();
    
    if total_secs == 0 {
        println!("{label:<26}: {millis} ms");
    } else {
        println!("{label:<26}: {mins}m {secs}s");
    }
}

/// Display success message
pub fn show_success_message(msg: &str) {
    println!("\n✅ {msg}\n");
}

/// Display error message
pub fn show_error_message(msg: &str) {
    println!("\n❌ {msg}\n");
}

/// Display warning message
pub fn show_warning_message(msg: &str) {
    println!("\n⚠️  {msg}\n");
}

/// Calculate and display processing efficiency percentage
///
/// # Arguments
/// * `stats` - The processing statistics
///
/// # Returns
/// Success rate as a percentage (0-100)
pub fn calculate_success_rate(stats: &Stats) -> f64 {
    if stats.total_rows == 0 {
        return 0.0;
    }
    
    let successful = stats.total_rows - stats.removed_rows;
    (successful as f64 / stats.total_rows as f64) * 100.0
}
