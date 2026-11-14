//! The schemas the `json-schema` command writes, one entry per command
//! printing a result.
//!
//! A key is the command path joined with hyphens and prefixed `mml-`,
//! as the man pages are named. The commands producing a MIME message
//! print it as [`Bytes`], the template commands a [`TemplateOutput`],
//! `interpret` an [`InterpretOutput`]. The one key naming no command is
//! `mml-error`, the failure object any command prints instead.
//!
//! [`Bytes`]: pimalaya_cli::output::Bytes
//! [`TemplateOutput`]: crate::cli::template::TemplateOutput
//! [`InterpretOutput`]: crate::cli::interpret::InterpretOutput

use pimalaya_cli::{
    clap::commands::json_schema::JsonSchemaRegistry, error::ErrorReport, output::Bytes,
};

use crate::cli::{error::ErrorCode, interpret::InterpretOutput, template::TemplateOutput};

/// Builds the command-to-schema registry `json-schema` writes.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("mml", env!("CARGO_PKG_VERSION"));

    registry.insert::<Bytes>("mml-compile");

    registry.insert::<InterpretOutput>("mml-interpret");
    registry.insert::<TemplateOutput>("mml-template-compose");
    registry.insert::<TemplateOutput>("mml-template-reply");
    registry.insert::<TemplateOutput>("mml-template-forward");

    for key in ["mml-compose", "mml-reply", "mml-forward", "mml-mailto"] {
        registry.insert::<Bytes>(key);
    }

    registry.insert::<ErrorReport<ErrorCode>>("mml-error");

    registry
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::{Cli, json_schema::registry};

    /// A bare `mml` prints the help or the welcome, nothing a schema
    /// could describe.
    const SILENT: &[&str] = &["mml"];

    /// Every runnable command is described, and every key names one.
    #[test]
    fn the_registry_matches_the_command_tree() {
        assert_eq!(
            registry().check(&Cli::command(), SILENT),
            Vec::<String>::new()
        );
    }

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
    }
}
