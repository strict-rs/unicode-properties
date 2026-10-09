//! Consumer-owned registry for `just x <name>` commands.

use std::ops::ControlFlow;

use template_core::cli::command::CommandSet;
use template_core::cli::parse::ParseReport;

/// Build this repository's intentionally empty extension registry.
///
/// # Errors
///
/// Returns a typed registration error if the controlled `x` router metadata
/// is invalid.
pub fn commands() -> template_stask::Result<CommandSet<ControlFlow<ParseReport>>> {
    template_stask::empty_registry("rust-template extensions")
}
