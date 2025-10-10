//! CLI entry point for hypr-keybind-manager
//!
//! Provides command-line interface for checking conflicts,
//! listing keybindings, and launching the GUI.

use clap::{Parser, Subcommand};
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { config } => {
            println!("Checking for conflicts in: {:?}", config);
            println!("TODO: Implement conflict checking");
        }
        Commands::List { config } => {
            println!("Listing keybindings from: {:?}", config);
            println!("TODO: Implement keybinding listing");
        }
        Commands::Gui => {
            println!("TODO: Launch GTK4 GUI");
        }
    }
}

