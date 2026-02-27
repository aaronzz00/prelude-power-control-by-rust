pub mod error;
pub mod power;

// Exporting typical Integration file for Tauri as module (not compiled strictly by default unless invoked)
pub mod tauri_integration;

pub use error::{PowerControllerError, Result};
pub use power::{DeviceSide, PowerController, WireMode};
