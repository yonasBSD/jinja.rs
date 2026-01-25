use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use clap::Parser;
use minijinja::{Environment, value::Value};
use rhai::{AST, Dynamic, Engine, Scope};
use serde::Deserialize;

//
// ──────────────────────────────────────────────────────────────────────────────
//  CLI ARGUMENTS
// ──────────────────────────────────────────────────────────────────────────────
//
// The CLI only accepts a template path. All variable definitions and execution
// behavior come from the YAML configuration (j2.yaml). This keeps the runtime
// interface simple while allowing the configuration file to drive all logic.
//

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long)]
    template: PathBuf,
}

//
// ──────────────────────────────────────────────────────────────────────────────
//  CONFIGURATION STRUCTURES
// ──────────────────────────────────────────────────────────────────────────────
//
// These structs represent the YAML schema. They intentionally mirror the
// structure of j2.yaml so that serde_yaml can deserialize directly into them.
//
// ArgumentSpec: describes a single argument for a Rhai function.
// VarSpec: describes a variable that can be produced via:
//   - a Rhai script
//   - a Rhai function (exposed as a MiniJinja filter)
//   - a single shell command
//   - multiple shell commands
//
// RootConfig: top‑level configuration including global defaults.
//

#[derive(Debug, Deserialize, Clone)]
pub struct ArgumentSpec {
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VarSpec {
    // Name of the variable exposed to the template context.
    // Required for script/cmd/cmds variables; optional for filters.
    name: Option<String>,

    // Rhai function name to expose as a MiniJinja filter.
    function: Option<String>,

    // Function arguments (for Rhai function definitions).
    #[serde(default)]
    arguments: Vec<ArgumentSpec>,

    // Raw Rhai script body (used for script variables or function bodies).
    #[serde(default)]
    script: String,

    // Single shell command to evaluate.
    #[serde(default)]
    cmd: Option<String>,

    // Multiple shell commands to evaluate and join.
    #[serde(default)]
    cmds: Option<Vec<String>>,

    // Per‑variable shell override (e.g., "bash", "fish").
    #[serde(default)]
    shell: Option<String>,

    // Per‑variable working directory override.
    #[serde(default)]
    cwd: Option<String>,

    // Per‑variable environment variable overrides.
    #[serde(default)]
    env: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct RootConfig {
    // Global default shell used when a variable does not specify one.
    #[serde(default)]
    default_shell: Option<String>,

    // List of variable specifications.
    #[serde(default)]
    vars: Vec<VarSpec>,
}

//
// ──────────────────────────────────────────────────────────────────────────────
//  COMMAND EXECUTION
// ──────────────────────────────────────────────────────────────────────────────
//
// eval_cmd executes a shell command with layered precedence for shell
// selection:
//   1. per‑variable shell override
//   2. global default_shell
//   3. "fish" as a hard default
//
// It also applies per‑variable working directory and environment overrides.
// The function returns stdout as a trimmed UTF‑8 string, or an error message.
//

pub fn eval_cmd(
    cmd: &str,
    shell: Option<&str>,
    global_default: Option<&str>,
    cwd: Option<&str>,
    env: Option<&HashMap<String, String>>,
) -> String {
    // Determine which shell to use.
    let shell_bin = shell.or(global_default).unwrap_or("fish");

    let args = ["-c", cmd];

    let mut command = std::process::Command::new(shell_bin);
    command.args(&args);

    // Apply working directory override.
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    // Apply environment variable overrides.
    if let Some(env_map) = env {
        for (k, v) in env_map {
            command.env(k, v);
        }
    }

    // Execute and capture output.
    let output = command.output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(e) => format!("ERROR: {}", e),
    }
}

//
// ──────────────────────────────────────────────────────────────────────────────
//  MAIN EXECUTION PIPELINE
// ──────────────────────────────────────────────────────────────────────────────
//
// The main function orchestrates the entire workflow:
//
//   1. Parse CLI arguments.
//   2. Load and deserialize YAML configuration.
//   3. Build a Rhai engine and dynamically compile function definitions.
//   4. Register Rhai functions as MiniJinja filters.
//   5. Evaluate script/cmd/cmds variables into a MiniJinja context.
//   6. Load and render the template.
//   7. Print the rendered output.
//
// This design cleanly separates configuration, evaluation, and rendering.
//

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // ──────────────────────────────────────────────────────────────────────────
    // Load YAML configuration (j2.yaml).
    // The file name is currently hard‑coded; future versions may allow passing
    // this as a CLI argument or auto‑discovering config files.
    // ──────────────────────────────────────────────────────────────────────────
    let yaml = fs::read_to_string("j2.yaml")?;
    let root: RootConfig = serde_yaml::from_str(&yaml)?;
    let specs = &root.vars;

    // ──────────────────────────────────────────────────────────────────────────
    // Build a Rhai engine instance.
    // This engine is shared across all script evaluations and filter calls.
    // ──────────────────────────────────────────────────────────────────────────
    let engine = Engine::new();

    // ──────────────────────────────────────────────────────────────────────────
    // Construct Rhai function definitions dynamically.
    //
    // Each VarSpec with a `function` field becomes a Rhai function whose body is
    // the provided script. These functions are later exposed as MiniJinja filters.
    //
    // Example generated code:
    //     fn my_filter(arg1, arg2) { <script> }
    //
    // This allows users to define custom template filters entirely in YAML.
    // ──────────────────────────────────────────────────────────────────────────
    let mut func_defs = String::new();
    for spec in specs {
        if let Some(func_name) = &spec.function {
            let arg_list = spec
                .arguments
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", ");

            func_defs.push_str(&format!(
                "fn {}({}) {{ {} }}\n",
                func_name, arg_list, spec.script
            ));
        }
    }

    // Compile all dynamically generated Rhai functions into an AST.
    let ast: AST = engine.compile(func_defs)?;

    // ──────────────────────────────────────────────────────────────────────────
    // Initialize MiniJinja environment.
    // Filters will be registered here, and the final template will be rendered
    // using this environment.
    // ──────────────────────────────────────────────────────────────────────────
    let mut env = Environment::new();

    let arc_engine = Arc::new(engine);
    let arc_ast = Arc::new(ast);

    // ──────────────────────────────────────────────────────────────────────────
    // Register Rhai functions as MiniJinja filters.
    //
    // Each filter is a closure capturing:
    //   - the Rhai engine
    //   - the compiled AST
    //   - the function name
    //
    // Filters accept a single string argument for now. Future extensions may
    // support multiple arguments by mapping MiniJinja values into Rhai Dynamics.
    // ──────────────────────────────────────────────────────────────────────────
    for spec in specs {
        if let Some(func_name) = &spec.function {
            let fn_name = func_name.clone();
            let e = Arc::clone(&arc_engine);
            let a = Arc::clone(&arc_ast);

            env.add_filter(
                fn_name.clone(),
                move |name: String| -> Result<String, minijinja::Error> {
                    let mut scope = Scope::new();

                    let result: Dynamic =
                        e.call_fn(&mut scope, &a, &fn_name, (name,))
                            .map_err(|err| {
                                minijinja::Error::new(
                                    minijinja::ErrorKind::InvalidOperation,
                                    format!("Rhai Call Error: {err}"),
                                )
                            })?;

                    Ok(result.to_string())
                },
            );
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Build the MiniJinja template context.
    //
    // For each VarSpec with a `name`, we evaluate one of:
    //   - a Rhai script
    //   - a single command
    //   - multiple commands
    //
    // The resulting value is inserted into the template context under the
    // variable's name. Multi‑command results are joined with newlines.
    // ──────────────────────────────────────────────────────────────────────────
    let mut ctx: HashMap<String, Value> = HashMap::new();

    for spec in specs {
        if let Some(name) = &spec.name {
            // Evaluate Rhai script variables (non‑filter).
            if spec.function.is_none() && !spec.script.trim().is_empty() {
                let mut scope = Scope::new();

                let result: Dynamic = arc_engine
                    .eval_with_scope(&mut scope, &spec.script)
                    .map_err(|err| anyhow::anyhow!("Rhai Script Error: {}", err))?;

                ctx.insert(name.clone(), Value::from(result.to_string()));
            }

            // Evaluate single command variables.
            if let Some(cmd) = &spec.cmd {
                let result = eval_cmd(
                    cmd,
                    spec.shell.as_deref(),
                    root.default_shell.as_deref(),
                    spec.cwd.as_deref(),
                    spec.env.as_ref(),
                );
                ctx.insert(name.clone(), Value::from(result));
            }

            // Evaluate multi‑command variables.
            if let Some(cmd_list) = &spec.cmds {
                let mut results = Vec::new();

                for cmd in cmd_list {
                    let out = eval_cmd(
                        cmd,
                        spec.shell.as_deref(),
                        root.default_shell.as_deref(),
                        spec.cwd.as_deref(),
                        spec.env.as_ref(),
                    );
                    results.push(out);
                }

                let joined = results.join("\n");
                ctx.insert(name.clone(), Value::from(joined));
            }
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Load and render the MiniJinja template.
    //
    // The template is added under the name "main" and rendered using the
    // previously constructed context. Any filter or variable defined above is
    // now available to the template.
    // ──────────────────────────────────────────────────────────────────────────
    let template_text = fs::read_to_string(&cli.template)?;
    env.add_template("main", &template_text)?;

    let tmpl = env.get_template("main")?;
    let output = tmpl.render(ctx)?;

    println!("{output}");

    Ok(())
}

mod common;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
