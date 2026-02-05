/*
 * CLI Entry point.
 *
 * Sets up dual diagnostics:
 * 1. Miette: For handled, structured errors.
 * 2. Color-Eyre: For unhandled panics and developer context.
 * 3. File: Structured JSON logs in ./logs/api-errors.log.
 */

use errors_lib::{perform_task, handle_error_logic, CliResult, ReportExt};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> CliResult<()> {
    // 1. Install Color-Eyre for beautiful panic reports
    // This provides suggestions and backtrace cleaning if the app crashes.
    color_eyre::install().expect("Failed to install color-eyre");

    // 2. Setup file appender
    let file_appender = tracing_appender::rolling::daily("logs", "api-errors.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 3. Respect RUST_LOG or default to 'off'
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("off"));

    tracing_subscriber::registry()
        .with(fmt::layer().json().with_writer(non_blocking))
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .compact()
                .with_filter(filter)
        )
        .init();

    // 4. Miette hook for standard diagnostics
    miette::set_panic_hook();

    println!("--- Starting Task ---");

    if let Err(report) = perform_task() {
        handle_error_logic(&report);

        // API Sink (File)
        let api_err = report.to_api_error();

        // Empathy UI
        eprintln!("\n[Diagnostic ID: {}]", api_err.correlation_id);

        // Terminal Sink (miette)
        return Err(miette::Report::new(report));
    }

    Ok(())
}
