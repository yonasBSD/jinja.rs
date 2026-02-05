## 🏗️ Architecture of errors.rs

This architecture leverages Rust's trait system to create a unified diagnostic engine. It transitions from **Structured Data** (SNAFU) to **Contextual History** (error-stack) and finally to **Visual/Machine Output** (Miette/Problemo).

---

### 1. The Definition Layer (Identity)

* **[SNAFU](https://github.com/shepmaster/snafu):** Your core error "factory." Use this to define domain-specific enums. It provides the metadata (labels, help text, error codes) that later layers will consume.
* **[anyhow](https://github.com/dtolnay/anyhow) / [color-eyre](https://github.com/eyre-rs/eyre):** Reserved for top-level application boundaries. `color-eyre` is specifically integrated to provide rich panic hooks and "Section" based reporting during development.

### 2. The Propagation Layer (Logic)

* **[error-stack](https://github.com/hashintel/hash):** The primary wrapper. It provides the `Report<E>` type, which allows you to "stack" context. Unlike `anyhow`, it maintains type-safety for the original error defined in SNAFU.
* **[rootcause](https://github.com/mcarvalho624/rootcause) / [handle-this](https://github.com/BrandonLeeDotDev/handle-this):** Used for control-flow logic. `rootcause` allows you to inspect the very bottom of an `error-stack` to make branching decisions based on the original `std::io::Error` or similar.

### 3. The Presentation Layer (Output Sinks)

* **[Miette](https://github.com/zkat/miette) & [Ariadne](https://github.com/zesterer/ariadne):** The "Visual Sink." Miette defines the `Diagnostic` trait which your errors implement. When a report is printed to a TTY, Ariadne is triggered to render the source code snippets and labels.
* **[Problemo](https://github.com/tliron/problemo):** The "API Sink." Converts the internal `Report` into **RFC 7807** JSON for web services (Axum/Actix).
* **[exn](https://github.com/fast/exn) / [fast-serialization](https://github.com/fast-serialization/fast-serialization):** Used to flatten the complex stack into a machine-readable JSON format for structured logging (tracing).

---

### 💻 Conceptual Implementation (Core Logic)

```rust
/**
 * Unified Diagnostic Provider
 * Integrating SNAFU for definition and error-stack for propagation.
 */

use miette::{Diagnostic, SourceSpan};
use snafu::prelude::*;
use error_stack::{Report, ResultExt};

// 1. Define the Structured Error with Miette metadata
#[derive(Debug, Snafu, Diagnostic)]
#[snafu(visibility(pub))]
#[diagnostic(
    code(env::missing_var),
    help("Please set the variable in your .env file or shell."),
    url("[https://docs.rs/mylib/errors/env](https://docs.rs/mylib/errors/env)")
)]
pub enum ConfigError {
    #[snafu(display("Environment variable '{var}' not found"))]
    MissingVar {
        var: String,
        #[label("required variable")]
        span: SourceSpan,
    },
}

// 2. Function using error-stack to wrap SNAFU errors
pub fn initialize_app() -> Result<(), Report<ConfigError>> {
    let var_name = "API_KEY".to_string();

    // Triggering an error and attaching printable context
    Err(ConfigError::MissingVar {
        var: var_name,
        span: (0, 7).into(),
    })
    .into_report()
    .attach_printable("Failed to initialize the security subsystem")
}

/* * NOTE: Presentation logic (Miette vs Problemo) 
 * occurs at the application boundary (main.rs or controller.rs).
 */
```

### 🛠️ The Extension Trait: Mapping Reports to Sinks

This trait allows you to call `.to_problem()` or `.to_diagnostic()` directly on any `Result` or `Report` in your codebase.

```rust
/**
 * Extension trait to bridge error-stack with Problemo and Miette.
 */

use error_stack::Report;
use problemo::Problem;
use miette::Diagnostic;

pub trait ReportExt {
    /// Converts the report into an RFC 7807 Problem Detail for APIs.
    fn to_problem(&self) -> Problem;

    /// Renders the report using Miette/Ariadne for the terminal.
    fn render_diagnostic(&self);
}

impl<E> ReportExt for Report<E> 
where 
    E: Diagnostic + std::fmt::Display + 'static 
{
    fn to_problem(&self) -> Problem {
        // Extracting the root cause and printable attachments
        let title = self.current_context().to_string();
        let detail = format!("{:?}", self); // Captured stack trace as detail

        Problem::new("[https://api.myapp.com/errors/internal](https://api.myapp.com/errors/internal)")
            .with_title(title)
            .with_detail(detail)
            // Problemo can take extra fields for machine-readable metadata
            .with_value("code", E::code(&self.current_context()).map(|c| c.to_string()))
    }

    fn render_diagnostic(&self) {
        // Miette can wrap the error-stack report for Ariadne rendering
        // This is where the terminal 'pretty printing' magic happens
        println!("{:?}", miette::Report::new(self.current_context()));
    }
}

/* * Usage Example:
 * let result = initialize_app().map_err(|e| e.to_problem());
 */
```

---

## Final Summary of the Flow

* **Define** with **SNAFU** (Attributes for Miette).
* **Wrap** with **error-stack** (Add `attach_printable` context).
* **Inspect** with **rootcause** (If logic branching is needed).
* **Finalize** with the **Extension Trait** (Output to **Ariadne** for CLI or **Problemo** for Web).
