/*
 * Structured Error Definitions.
 */

#![allow(unused_assignments)]

use miette::{Diagnostic, SourceSpan, NamedSource};
use snafu::prelude::*;

#[derive(Debug, Snafu, Diagnostic)]
#[snafu(visibility(pub))]
#[diagnostic(
    code(config::invalid_format),
    help("Ensure the configuration file is valid JSON."),
)]
pub enum LibError {
    /// Error when parsing configuration files.
    #[snafu(display("Failed to parse config at {path}"))]
    ConfigParseError {
        /// The path to the file.
        path: String,
        /// The source code for snippet rendering.
        #[source_code]
        src: NamedSource<String>,
        /// The location of the error.
        #[label("syntax error here")]
        span: SourceSpan,
    },

    /// Error when a network operation times out.
    #[snafu(display("Network timeout after {timeout}s"))]
    NetworkError {
        /// Timeout duration in seconds.
        timeout: u64
    },
}
