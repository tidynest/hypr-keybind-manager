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
//! ├── actions.rs      // GTK action setup (quit, export, import)
//! ├── builders/       // UI building functions
//! └── components/     // Reusable UI widgets
//! ```

mod actions;
pub mod app;
mod builders;
pub mod components;
pub mod controller;
pub mod file_watcher;

pub use {app::App, controller::Controller};

#[cfg(test)]
mod tests;
