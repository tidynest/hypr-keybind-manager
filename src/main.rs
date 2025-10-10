//! CLI entry point for hypr-keybind-manager
//!
//! Provides command-line interface for checking conflicts,
//! listing keybindings, and launching the GUI.

use clap::{Parser, Subcommand};
use colored::*;
use hypr_keybind_manager::core::{parser::parse_config_file, conflict::ConflictDetector};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hypr-keybind-manager")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
    Gui,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { config } => check_conflicts(&config)?,
        Commands::List { config } => list_keybindings(&config)?,
        Commands::Gui => {
            println!("{}", "GUI not yet implemented".yellow());
            println!("Coming in Phase 5!");
        }
    }

    Ok(())
}

/// Check config for keybinding conflicts
fn check_conflicts(config_path: &PathBuf) -> anyhow::Result<()> {
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
                let args = binding.args.as_ref().map(|s| s.as_str()).unwrap_or("");

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

/// List all keybinding in the config
fn list_keybindings(config_path: &PathBuf) -> anyhow::Result<()> {
    // Expand tilde in path
    let expanded_path = shellexpand::tilde(
        config_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?
    );
    let path = std::path::Path::new(expanded_path.as_ref());

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