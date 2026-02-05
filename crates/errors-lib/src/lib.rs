/*
 * Core library logic and the Sink Extension Trait.
 *
 * This version uses:
 * 1. rootcause: For tree-based error propagation.
 * 2. tracing: For automatic structured logging.
 * 3. serde: For machine-readable API error serialization.
 */

pub mod types;

use types::LibError;
use miette::{Diagnostic, SourceCode};
use rootcause::Report;
use serde::{Serialize, Serializer};
use tracing::error;
use nanoid::nanoid;

pub use rootcause;
pub use miette::Result as CliResult;

#[derive(Debug)]
pub struct LibReport(pub Report<LibError>);

pub type LibResult<T> = std::result::Result<T, LibReport>;

#[derive(Debug, Serialize)]
pub struct ErrorFrame {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub git_hash: String,
    pub docs_url: String,
    pub correlation_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(serialize_with = "serialize_history_flat")]
    pub history: Vec<ErrorFrame>,
}

fn serialize_history_flat<S>(history: &[ErrorFrame], serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    let flat: Vec<&str> = history.iter().map(|f| f.message.as_str()).collect();
    flat.serialize(serializer)
}

/* * DIAGNOSTIC IMPLEMENTATION * */
impl Diagnostic for LibReport {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.current_context().code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.0.current_context().severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.current_context().help()
    }

    /* * RESTORED: Dynamic URL Generation
     * Maps the error code to a clickable link in the terminal.
     */
    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let base = env!("ERROR_DOCS_URL");
        self.code().map(|c| {
            let link = format!("{}/#{}", base, c);
            Box::new(link) as Box<dyn std::fmt::Display>
        })
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.0.current_context().source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.0.current_context().labels()
    }
}

impl std::fmt::Display for LibReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LibReport {}

pub trait ReportExt {
    fn to_api_error(&self) -> ApiError;
}

impl ReportExt for LibReport {
    fn to_api_error(&self) -> ApiError {
        let mut history = Vec::new();
        for node in self.0.iter_reports() {
            for attachment in node.attachments().iter() {
                history.push(ErrorFrame { message: attachment.to_string() });
            }
        }

        let ctx = self.0.current_context();
        let api_err = ApiError {
            git_hash: env!("GIT_HASH").to_string(),
            docs_url: env!("ERROR_DOCS_URL").to_string(),
            correlation_id: nanoid!(8),
            title: ctx.to_string(),
            code: LibError::code(ctx).map(|c| c.to_string()),
            help: LibError::help(ctx).map(|h| h.to_string()),
            history,
        };

        error!(
            hash = %api_err.git_hash,
            docs = %api_err.docs_url,
            id = %api_err.correlation_id,
            title = %api_err.title,
            code = api_err.code.as_deref(),
            history = ?api_err.history.iter().map(|h| &h.message).collect::<Vec<_>>(),
            "Internal error reported to API sink"
        );

        api_err
    }
}

pub fn handle_error_logic(report: &LibReport) {
    for node in report.0.iter_reports() {
        if let Some(io_err) = node.downcast_current_context::<std::io::Error>() {
            if matches!(io_err.kind(), std::io::ErrorKind::NotFound) {
                println!("--- LOGIC CHECK: Missing file detected ---");
            }
        }
    }
}

pub fn perform_task() -> LibResult<()> {
    let err = LibError::ConfigParseError {
        path: "config.json".into(),
        src: miette::NamedSource::new("config.json", "{ \"key\": !!invalid }".to_string()),
        span: (10, 9).into(),
    };

    let report = Report::new(err)
        .attach("The application cannot proceed without a valid config.");

    Err(LibReport(report))
}
