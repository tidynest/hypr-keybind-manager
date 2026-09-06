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
use hypr_keybind_manager::{
    config::{default_config_path, load_bindings_from},
    core::conflict::ConflictDetector,
    ui::App,
};
use std::path::{Path, PathBuf};

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
        /// Path to Hyprland config file [default: ~/.config/hypr/hyprland.conf, or hyprland.lua when only that exists]
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// List all keybindings
    List {
        /// Path to Hyprland config file [default: ~/.config/hypr/hyprland.conf, or hyprland.lua when only that exists]
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Print the bindings as JSON instead of a table
        #[arg(long)]
        json: bool,
    },

    /// Launch GUI overlay
    Gui {
        /// Path to Hyprland config file [default: ~/.config/hypr/hyprland.conf, or hyprland.lua when only that exists]
        #[arg(short, long)]
        config: Option<PathBuf>,
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
    // SAFETY: called before any threads are spawned
    unsafe {
        std::env::set_var("G_MESSAGES_DEBUG", "");
        std::env::set_var("GTK_DEBUG", "");
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Check { config } => check_conflicts(&resolve_config_path(config)?)?,
        Commands::List { config, json } => list_keybindings(&resolve_config_path(config)?, json)?,
        Commands::Gui { config } => launch_gui(&resolve_config_path(config)?)?,
    }

    Ok(())
}

/// Expands `~` in a given path, or picks the config Hyprland would load
fn resolve_config_path(config: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let Some(config) = config else {
        return Ok(default_config_path());
    };
    let text = config
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;
    Ok(PathBuf::from(shellexpand::tilde(text).as_ref()))
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
fn check_conflicts(path: &Path) -> anyhow::Result<()> {
    println!("{} Parsing config: {}", "→".cyan(), path.display());

    let bindings = load_bindings_from(path)?.bindings;

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
            let submap = conflict
                .submap
                .as_deref()
                .map(|s| format!(" in submap {s}"))
                .unwrap_or_default();
            println!(
                "{} {}{}",
                format!("Conflict {}", i + 1).yellow().bold(),
                format!("{}", conflict.key_combo).cyan(),
                submap.dimmed()
            );

            for (idx, binding) in conflict.conflicting_bindings.iter().enumerate() {
                let args = binding.args.as_deref().unwrap_or("");

                println!(
                    "  {} {} → {} {}",
                    format!("{}.", idx + 1).dimmed(),
                    format!("{}", binding.bind_type).magenta(),
                    binding.dispatcher,
                    args,
                );
            }
            println!();
        }

        println!(
            "{}",
            "⚠ These keybindings will conflict at runtime!".yellow()
        );
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
fn list_keybindings(path: &Path, json: bool) -> anyhow::Result<()> {
    let bindings = load_bindings_from(path)?.bindings;

    if json {
        println!("{}", serde_json::to_string_pretty(&bindings)?);
        return Ok(());
    }

    println!(
        "{}",
        format!("Keybindings from: {}\n", path.display()).bold()
    );

    let total = bindings.len();

    // Display each binding
    for binding in bindings {
        let key_combo = format!("{}", binding.key_combo).cyan().bold();
        let dispatcher = binding.dispatcher.green();
        let args = binding.args.unwrap_or_default();
        let submap = binding
            .submap
            .map(|s| format!("[{s}] ").dimmed().to_string())
            .unwrap_or_default();
        let description = binding
            .description
            .map(|d| format!("  # {d}").dimmed().to_string())
            .unwrap_or_default();

        println!(
            "{submap}{} → {} {}{description}",
            key_combo, dispatcher, args
        );
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
fn launch_gui(path: &Path) -> anyhow::Result<()> {
    eprintln!("{} Launching GUI...", "→".cyan());

    let app =
        App::new(path.to_path_buf()).map_err(|e| anyhow::anyhow!("Failed to create app: {}", e))?;

    app.run();

    Ok(())
}
