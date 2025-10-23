//! UI builder modules
//!
//! Contains modular builders for constructing the main application UI:
//! - Header bar creation
//! - Layout construction
//! - Event handler wiring

pub mod handlers;
pub mod header;
pub mod layout;

pub use handlers::wire_up_handlers;
pub use header::build_header_bar;
pub use layout::build_main_layout;
