//! Command-line interface module for FerrumC.
//!
//! This module provides all CLI argument parsing and command handling
//! functionality for the FerrumC Minecraft server.
//!
//! # Submodules
//!
//! - [`clear`] - Clear command for removing server data
//!
//! # Example
//!
//! ```bash
//! # Run the server
//! ionic run
//!
//! # Clear all server data
//! ionic clear --all
//! ```

pub mod args;
pub mod clear;
pub mod errors;
