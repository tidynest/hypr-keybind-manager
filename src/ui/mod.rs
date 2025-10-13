//! GTK4 user interface with MVC architecture
//!
//! # Architecture
//!
//! - **Model**: ConfigManager, ConflictDetector (in `config` and `core` modules)
//! - **View**: GTK4 components (in `components/` submodule)
//! - **Controller**: Mediates between Model and View (in `controller.rs`)
//!
//! # Module Structure
//!
//! ```text
//! ui/
//! ├── mod.rs          // This file - exports and initialisation
//! ├── app.rs          // GTK4 Application setup
//! ├── controller.rs   // MVC Controller
//! └── components/     // Reusable UI widgets
//! ```

pub mod app;
pub mod controller;
pub mod components;

pub use app::App;
pub use controller::Controller;