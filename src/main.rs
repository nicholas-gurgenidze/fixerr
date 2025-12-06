// src/main.rs
//! Fixerr - CSV Repair Utility
//! 
//! Main entry point with interactive menu system.

use fixerr::{reconstruct_records, write_output_csv, HeaderMode, Delimiter, Stats};
use std::error::Error;
use std::path::Path;
use std::time::Instant;

mod ui;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub delimiter: Delimiter,
    pub header_mode: HeaderMode,
    pub input_file: String,
    pub output_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            delimiter: Delimiter::Comma,
            header_mode: HeaderMode::HasHeaders,
            input_file: "data.csv".to_string(),
            output_file: "output.csv".to_string(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::default();
    
    loop {
        ui::display_welcome();
        
        let choice = ui::get_menu_choice(1, 3, "\nEnter your choice (1-3): ")?;
        
        match choice {
            1 => {
                if let Err(e) = process_csv(&config) {
                    ui::show_error_message(&format!("Processing failed: {e}"));
                }
                ui::pause();
            }
            2 => configure_settings(&mut config)?,
            3 => {
                ui::clear_screen();
                println!("\n✨ Thank you for using Fixerr! Goodbye.\n");
                break;
            }
            _ => unreachable!(), // Validation prevents this
        }
    }
    
    Ok(())
}

/// Process CSV file with current configuration
fn process_csv(config: &Config) -> Result<(), Box<dyn Error>> {
    // Validate input file exists
    if !Path::new(&config.input_file).exists() {
        return Err(format!("Input file '{}' not found", config.input_file).into());
    }
    
    ui::display_processing_header(config);
    
    let mut stats = Stats::default();
    let total_start = Instant::now();
    
    // Phase 1: Reconstruct records
    println!("🔄 Phase 1: Analyzing and reconstructing records...");
    let process_start = Instant::now();
    
    let records = reconstruct_records(
        &config.input_file,
        config.header_mode,
        config.delimiter,
        &mut stats,
    )?;
    
    ui::print_elapsed("   Processing Time", process_start);
    
    // Phase 2: Write output
    println!("\n💾 Phase 2: Writing cleaned CSV...");
    let write_start = Instant::now();
    
    write_output_csv(&config.output_file, &records, config.delimiter)?;
    
    ui::print_elapsed("   Writing Time", write_start);
    
    // Total time
    println!();
    ui::print_elapsed("   Total Time", total_start);
    
    // Summary statistics
    ui::display_summary(&stats, records.len(), &config.output_file);
    
    Ok(())
}

/// Configure application settings with submenu
fn configure_settings(config: &mut Config) -> Result<(), Box<dyn Error>> {
    loop {
        ui::display_settings_menu(config);
        
        let choice = ui::get_menu_choice(1, 6, "\nEnter your choice (1-6): ")?;
        
        match choice {
            1 => change_delimiter(config)?,
            2 => change_header_mode(config)?,
            3 => change_input_file(config)?,
            4 => change_output_file(config)?,
            5 => {
                *config = Config::default();
                ui::show_success_message("Settings reset to defaults!");
            }
            6 => break, // Back to main menu
            _ => unreachable!(), // Validation prevents this
        }
        
        if choice != 6 {
            ui::pause();
        }
    }
    
    Ok(())
}

/// Change delimiter setting
fn change_delimiter(config: &mut Config) -> Result<(), Box<dyn Error>> {
    let current = format!("{:?}", config.delimiter);
    ui::display_delimiter_menu(&current);
    
    let choice = ui::get_menu_choice(1, 4, "Select delimiter (1-4): ")?;
    
    config.delimiter = match choice {
        1 => Delimiter::Comma,
        2 => Delimiter::Semicolon,
        3 => Delimiter::Tab,
        4 => Delimiter::Pipe,
        _ => unreachable!(), // Validation prevents this
    };
    
    ui::show_success_message(&format!("Delimiter changed to: {:?}", config.delimiter));
    
    Ok(())
}

/// Change header mode setting
fn change_header_mode(config: &mut Config) -> Result<(), Box<dyn Error>> {
    let current = format!("{:?}", config.header_mode);
    ui::display_header_mode_menu(&current);
    
    let choice = ui::get_menu_choice(1, 2, "Select mode (1-2): ")?;
    
    config.header_mode = match choice {
        1 => HeaderMode::HasHeaders,
        2 => HeaderMode::NoHeaders,
        _ => unreachable!(), // Validation prevents this
    };
    
    ui::show_success_message(&format!("Header mode changed to: {:?}", config.header_mode));
    
    Ok(())
}

/// Change input file path
fn change_input_file(config: &mut Config) -> Result<(), Box<dyn Error>> {
    ui::display_file_path_screen("INPUT", &config.input_file);
    
    let input = ui::get_string_input("Enter new input file path (or press Enter to cancel): ")?;
    
    if input.is_empty() {
        ui::show_error_message("Input file path not changed.");
        return Ok(());
    }
    
    // Check if file exists
    if Path::new(&input).exists() {
        config.input_file = input.clone();
        ui::show_success_message(&format!("Input file changed to: {input}"));
    } else {
        ui::show_warning_message(&format!("File '{input}' does not exist."));
        
        if ui::get_confirmation("Set path anyway?")? {
            config.input_file = input.clone();
            ui::show_success_message(&format!("Input file changed to: {input}"));
        } else {
            ui::show_error_message("Input file path not changed.");
        }
    }
    
    Ok(())
}

/// Change output file path
fn change_output_file(config: &mut Config) -> Result<(), Box<dyn Error>> {
    ui::display_file_path_screen("OUTPUT", &config.output_file);
    
    let input = ui::get_string_input("Enter new output file path (or press Enter to cancel): ")?;
    
    if input.is_empty() {
        ui::show_error_message("Output file path not changed.");
        return Ok(());
    }
    
    // Check if file already exists
    if Path::new(&input).exists() {
        ui::show_warning_message(&format!("File '{input}' already exists."));
        
        if ui::get_confirmation("Overwrite?")? {
            config.output_file = input.clone();
            ui::show_success_message(&format!("Output file changed to: {input}"));
        } else {
            ui::show_error_message("Output file path not changed.");
        }
    } else {
        config.output_file = input.clone();
        ui::show_success_message(&format!("Output file changed to: {input}"));
    }
    
    Ok(())
}
