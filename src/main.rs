// Copyright 2025 Eric Jingryd (tidynest@proton.me)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! CLI entry point for Hyprland Keybinding Manager
//!
//! Provides a command-line interface for managing Hyprland keybindings with
//! three main commands: conflict checking, listing bindings, and launching
//! the graphical user interface.
//!
//! # Usage
//!
//! ```bash
//! # Check for conflicts
//! hypr-keybind-manager check -c ~/.config/hypr/hyprland.conf
//!
//! # List all keybindings
//! hypr-keybind-manager list
//!
//! # Launch GUI
//! hypr-keybind-manager gui
//! ```

use clap::{Parser, Subcommand};
use colored::*;
use std::{fs, path::{Path, PathBuf}};

use hypr_keybind_manager::core::{parser::parse_config_file, conflict::ConflictDetector};

/// Command-line interface for Hyprland Keybinding Manager.
///
/// Provides subcommands for checking conflicts, listing keybindings,
/// and launching the graphical interface.
#[derive(Parser)]
#[command(name = "hypr-keybind-manager")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Check for keybinding conflicts
    Check {
        /// Path to Hyprland config file
        #[arg(short, long, default_value = "~/.config/hypr/hyprland.conf")]
        config: PathBuf,
    },

    /// List all keybindings
    List {
        /// Path to Hyprland config file
        #[arg(short, long, default_value = "~/.config/hypr/hyprland.conf")]
        config: PathBuf,
    },

    /// Launch GUI overlay
    Gui {
        /// Path to Hyprland config file
        #[arg(short, long, default_value = "~/.config/hypr/hyprland.conf")]
        config: PathBuf,
    },
}

/// Main entry point for the CLI application.
///
/// Parses command-line arguments and dispatches to the appropriate subcommand handler.
/// Suppresses GTK debug output to keep terminal clean.
///
/// # Returns
///
/// * `Ok(())` - Command executed successfully
/// * `Err(_)` - Command failed with error details
fn main() -> anyhow::Result<(), Box<dyn std::error::Error>> {
    // Suppress GTK warnings and debug messages
    std::env::set_var("G_MESSAGES_DEBUG", "");
    std::env::set_var("GTK_DEBUG", "");

    let cli = Cli::parse();

    match cli.command {
        Commands::Check { config } => check_conflicts(&config)?,
        Commands::List { config } => list_keybindings(&config)?,
        Commands::Gui { config } => launch_gui(&config)?,
    }

    Ok(())
}

/// Checks configuration file for keybinding conflicts.
///
/// Parses the Hyprland config, detects duplicate key combinations,
/// and displays conflicts with coloured output. Exits with code 1
/// if conflicts are found.
///
/// # Arguments
///
/// * `config_path` - Path to Hyprland configuration file (supports tilde expansion)
///
/// # Returns
///
/// * `Ok(())` - No conflicts found
/// * `Err(_)` - File read or parse error
///
/// # Exits
///
/// Exits with code 1 if conflicts are detected
fn check_conflicts(config_path: &Path) -> anyhow::Result<()> {
    // Expand tilde in path
    let expanded_path = shellexpand::tilde(
        config_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?,
    );
    let path = std::path::Path::new(expanded_path.as_ref());

    // Read config file
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    println!("{} Parsing config: {}", "→".cyan(), path.display());

    // Parse bindings
    let bindings = parse_config_file(&content, path)?;

    println!("{} Found {} keybindings\n", "✓".green(), bindings.len());

    // Build conflict detector
    let mut detector = ConflictDetector::new();
    for binding in bindings {
        detector.add_binding(binding);
    }

    // Find conflicts
    let conflicts = detector.find_conflicts();

    if conflicts.is_empty() {
        println!("{} {}", "✓".green().bold(), "No conflicts detected!".bold());
        println!("\nYour keybindings are clean! ✓");
    } else {
        println!(
            "{} Found {} conflict{}:\n",
            "✗".red().bold(),
            conflicts.len(),
            if conflicts.len() == 1 { "" } else { "s" }
        );

        for (i, conflict) in conflicts.iter().enumerate() {
            println!("{} {}",
                 format!("Conflict {}", i + 1).yellow().bold(),
                 format!("{}", conflict.key_combo).cyan()
            );

            for (idx, binding) in conflict.conflicting_bindings.iter().enumerate() {
                let args = binding.args.as_deref().unwrap_or("");

                println!("  {} {} → {} {}",
                     format!("{}.", idx + 1).dimmed(),
                     format!("{}", binding.bind_type).magenta(),
                     binding.dispatcher,
                     args,
                );
            }
            println!();
        }

        println!("{}", "⚠ These keybindings will conflict at runtime!".yellow());
        std::process::exit(1);
    }

    Ok(())
}

/// Lists all keybindings from the configuration file.
///
/// Parses the Hyprland config and displays all keybindings with
/// formatted, colourised output showing key combinations, dispatchers,
/// and arguments.
///
/// # Arguments
///
/// * `config_path` - Path to Hyprland configuration file (supports tilde expansion)
///
/// # Returns
///
/// * `Ok(())` - Successfully listed bindings
/// * `Err(_)` - File read or parse error
fn list_keybindings(config_path: &Path) -> anyhow::Result<()> {
    // Expand tilde in path
    let expanded_path = shellexpand::tilde(
        config_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?
    );
    let path = Path::new(expanded_path.as_ref());

    // Read and parse
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    let bindings = parse_config_file(&content, path)?;

    println!("{}", format!("Keybindings from: {}\n", path.display()).bold());

    let total = bindings.len();

    // Display each binding
    for binding in bindings {
        let key_combo = format!("{}", binding.key_combo).cyan().bold();
        let dispatcher = binding.dispatcher.green();
        let args = binding.args.unwrap_or_default();

        println!("{} → {} {}", key_combo, dispatcher, args);
    }

    println!("\n{} Total: {} bindings", "✓".green(), total);

    Ok(())
}

/// Launches the graphical user interface.
///
/// Creates and runs the GTK4 application window for visual keybinding
/// management with real-time conflict detection and editing capabilities.
///
/// # Arguments
///
/// * `config_path` - Path to Hyprland configuration file (supports tilde expansion)
///
/// # Returns
///
/// * `Ok(())` - GUI closed successfully
/// * `Err(_)` - Failed to create or run application
///
/// # Blocking
///
/// This function blocks until the GUI window is closed by the user.
fn launch_gui(config_path: &Path) -> anyhow::Result<()> {
    use hypr_keybind_manager::ui::App;

    // Expand tilde in path
    let expanded_path = shellexpand::tilde(
        config_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?,
    );
    let expanded_path = PathBuf::from(expanded_path.as_ref());

    eprintln!("{} Launching GUI...", "→".cyan());

    // Create and run app
    let app = App::new(expanded_path)
        .map_err(|e| anyhow::anyhow!("Failed to create app: {}", e))?;

    app.run();

    Ok(())
}
