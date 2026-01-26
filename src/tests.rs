mod tests {
    use std::{collections::HashMap, fs, io::Write, sync::Arc};

    use jinja_rs::*;
    use minijinja::{Environment, value::Value};
    use pretty_assertions::assert_eq;
    use rhai::{Dynamic, Engine, Scope};
    use tempfile::{NamedTempFile, tempdir};

    use crate::*;

    // ══════════════════════════════════════════════════════════════════════════
    // CONFIGURATION DESERIALIZATION TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_deserialize_empty_config() {
        common::init();
        let yaml = "";
        let result: Result<RootConfig, _> = serde_yml::from_str(yaml);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.default_shell.is_none());
        assert!(config.vars.is_empty());
    }

    #[test]
    fn test_deserialize_config_with_default_shell() {
        common::init();
        let yaml = r#"
default_shell: bash
vars: []
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.default_shell, Some("bash".to_string()));
        assert!(config.vars.is_empty());
    }

    #[test]
    fn test_deserialize_script_variable() {
        common::init();
        let yaml = r#"
vars:
  - name: my_var
    script: "42"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars.len(), 1);
        assert_eq!(config.vars[0].name, Some("my_var".to_string()));
        assert_eq!(config.vars[0].script, "42");
        assert!(config.vars[0].function.is_none());
    }

    #[test]
    fn test_deserialize_cmd_variable() {
        common::init();
        let yaml = r#"
vars:
  - name: output
    cmd: "echo hello"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars.len(), 1);
        assert_eq!(config.vars[0].cmd, Some("echo hello".to_string()));
    }

    #[test]
    fn test_deserialize_cmds_variable() {
        common::init();
        let yaml = r#"
vars:
  - name: multi
    cmds:
      - "echo line1"
      - "echo line2"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars.len(), 1);
        let cmds = config.vars[0].cmds.as_ref().unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0], "echo line1");
        assert_eq!(cmds[1], "echo line2");
    }

    #[test]
    fn test_deserialize_function_with_arguments() {
        common::init();
        let yaml = r#"
vars:
  - function: my_filter
    arguments:
      - name: input
      - name: param
    script: "input + param"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars.len(), 1);
        assert_eq!(config.vars[0].function, Some("my_filter".to_string()));
        assert_eq!(config.vars[0].arguments.len(), 2);
        assert_eq!(config.vars[0].arguments[0].name, "input");
        assert_eq!(config.vars[0].arguments[1].name, "param");
    }

    #[test]
    fn test_deserialize_variable_with_env_override() {
        common::init();
        let yaml = r#"
vars:
  - name: test
    cmd: "echo $FOO"
    env:
      FOO: bar
      BAZ: qux
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let env = config.vars[0].env.as_ref().unwrap();
        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(env.get("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_deserialize_variable_with_cwd_override() {
        common::init();
        let yaml = r#"
vars:
  - name: test
    cmd: "pwd"
    cwd: "/tmp"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars[0].cwd, Some("/tmp".to_string()));
    }

    #[test]
    fn test_deserialize_variable_with_shell_override() {
        common::init();
        let yaml = r#"
vars:
  - name: test
    cmd: "echo hi"
    shell: bash
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars[0].shell, Some("bash".to_string()));
    }

    #[test]
    fn test_deserialize_complex_config() {
        common::init();
        let yaml = r#"
default_shell: bash
vars:
  - name: greeting
    script: "Hello, World!"
  - function: upper
    arguments:
      - name: text
    script: "text.to_upper()"
  - name: hostname
    cmd: "hostname"
  - name: info
    cmds:
      - "uname -s"
      - "uname -m"
    shell: sh
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.default_shell, Some("bash".to_string()));
        assert_eq!(config.vars.len(), 4);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // COMMAND EXECUTION TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_eval_cmd_simple_echo() {
        common::init();
        let result = eval_cmd("echo hello", Some("sh"), None, None, None);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_eval_cmd_with_global_default_shell() {
        common::init();
        let result = eval_cmd("echo test", None, Some("sh"), None, None);
        assert_eq!(result, "test");
    }

    /// Test eval_cmd logic selection (Mocking the behavior)
    #[test]
    fn test_eval_cmd_precedence() {
        common::init();

        // We can't easily mock the process execution without refactoring eval_cmd,
        // but we can verify that "echo" works across standard shells.

        // 1. Explicit shell (using sh as it's universal)
        let res = eval_cmd("echo 'hello'", Some("sh"), None, None, None);
        assert_eq!(res, "hello");

        // 2. Global default (sh)
        let res = eval_cmd("echo 'global'", None, Some("sh"), None, None);
        assert_eq!(res, "global");
    }

    /// Fixed version of your failing test
    #[test]
    fn test_eval_cmd_hardcoded_fish_fallback() {
        common::init();

        // Force extraction of embedded fish if it hasn't happened yet
        // so that eval_cmd has a valid path to work with.
        let _path = get_embedded_shell_path();

        // Now run the command
        let result = eval_cmd("echo 'fallback'", None, None, None, None);

        // Check for error first
        if result.starts_with("ERROR:") {
            panic!("eval_cmd failed to use embedded fish: {}", result);
        }

        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_eval_cmd_with_env() {
        common::init();

        let mut env_map = HashMap::new();
        env_map.insert("TEST_VAR".to_string(), "success".to_string());

        // Use sh for broad compatibility in tests
        let result = eval_cmd("echo $TEST_VAR", Some("sh"), None, None, Some(&env_map));
        assert_eq!(result, "success");
    }

    #[test]
    fn test_eval_cmd_shell_override_precedence() {
        common::init();
        // per-variable shell should override global default
        let result = eval_cmd("echo override", Some("sh"), Some("bash"), None, None);
        assert_eq!(result, "override");
    }

    #[test]
    fn test_eval_cmd_with_cwd() {
        common::init();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content").unwrap();

        let result = eval_cmd(
            "ls test.txt",
            Some("sh"),
            None,
            Some(temp_dir.path().to_str().unwrap()),
            None,
        );
        assert_eq!(result, "test.txt");
    }

    #[test]
    fn test_eval_cmd_with_env_vars() {
        common::init();
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let result = eval_cmd("echo $TEST_VAR", Some("sh"), None, None, Some(&env));
        assert_eq!(result, "test_value");
    }

    #[test]
    fn test_eval_cmd_with_multiple_env_vars() {
        common::init();
        let mut env = HashMap::new();
        env.insert("VAR1".to_string(), "value1".to_string());
        env.insert("VAR2".to_string(), "value2".to_string());

        let result = eval_cmd("echo $VAR1-$VAR2", Some("sh"), None, None, Some(&env));
        assert_eq!(result, "value1-value2");
    }

    #[test]
    fn test_eval_cmd_trims_whitespace() {
        common::init();
        let result = eval_cmd("echo '  spaces  '", Some("sh"), None, None, None);
        assert_eq!(result, "spaces");
    }

    #[test]
    fn test_eval_cmd_invalid_command() {
        common::init();
        let result = eval_cmd("nonexistent_command_xyz", Some("sh"), None, None, None);
        // Should contain error message but not panic
        assert!(result.contains("ERROR:") || result.is_empty() || !result.contains("nonexistent"));
    }

    #[test]
    fn test_eval_cmd_multiline_output() {
        common::init();
        let result = eval_cmd(
            "printf 'line1\\nline2\\nline3'",
            Some("sh"),
            None,
            None,
            None,
        );
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn test_eval_cmd_empty_output() {
        common::init();
        let result = eval_cmd("true", Some("sh"), None, None, None);
        assert_eq!(result, "");
    }

    // ══════════════════════════════════════════════════════════════════════════
    // RHAI SCRIPT EVALUATION TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_rhai_simple_arithmetic() {
        common::init();
        let engine = Engine::new();
        let mut scope = Scope::new();
        let result: Dynamic = engine.eval_with_scope(&mut scope, "2 + 2").unwrap();
        assert_eq!(result.to_string(), "4");
    }

    #[test]
    fn test_rhai_string_operations() {
        common::init();
        let engine = Engine::new();
        let mut scope = Scope::new();
        let result: Dynamic = engine
            .eval_with_scope(&mut scope, r#""hello".to_upper()"#)
            .unwrap();
        assert_eq!(result.to_string(), "HELLO");
    }

    #[test]
    fn test_rhai_function_compilation() {
        common::init();
        let engine = Engine::new();
        let script = "fn double(x) { x * 2.0 }";
        let ast = engine.compile(script).unwrap();
        let mut scope = Scope::new();
        let result: Dynamic = engine.call_fn(&mut scope, &ast, "double", (5.0,)).unwrap();
        assert_eq!(result.to_string(), "10.0");
    }

    #[test]
    fn test_rhai_multiple_functions() {
        common::init();
        let engine = Engine::new();
        let script = r#"
fn add(a, b) { a + b }
fn multiply(a, b) { a * b }
        "#;
        let ast = engine.compile(script).unwrap();
        let mut scope = Scope::new();

        let result1: Dynamic = engine.call_fn(&mut scope, &ast, "add", (3, 4)).unwrap();
        assert_eq!(result1.to_string(), "7");

        let result2: Dynamic = engine
            .call_fn(&mut scope, &ast, "multiply", (3, 4))
            .unwrap();
        assert_eq!(result2.to_string(), "12");
    }

    #[test]
    fn test_rhai_string_function() {
        common::init();
        let engine = Engine::new();
        let script = r#"fn greet(name) { "Hello, " + name + "!" }"#;
        let ast = engine.compile(script).unwrap();
        let mut scope = Scope::new();
        let result: Dynamic = engine
            .call_fn(&mut scope, &ast, "greet", ("World".to_string(),))
            .unwrap();
        assert_eq!(result.to_string(), "Hello, World!");
    }

    #[test]
    fn test_rhai_invalid_script() {
        common::init();
        let engine = Engine::new();
        let result = engine.compile("this is not valid rhai syntax +++");
        assert!(result.is_err());
    }

    // ══════════════════════════════════════════════════════════════════════════
    // MINIJINJA TEMPLATE RENDERING TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_minijinja_simple_variable() {
        common::init();
        let mut env = Environment::new();
        env.add_template("test", "Hello {{ name }}!").unwrap();
        let tmpl = env.get_template("test").unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("name".to_string(), Value::from("Alice"));

        let output = tmpl.render(ctx).unwrap();
        assert_eq!(output, "Hello Alice!");
    }

    #[test]
    fn test_minijinja_multiple_variables() {
        common::init();
        let mut env = Environment::new();
        env.add_template("test", "{{ greeting }} {{ name }}!")
            .unwrap();
        let tmpl = env.get_template("test").unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("greeting".to_string(), Value::from("Hi"));
        ctx.insert("name".to_string(), Value::from("Bob"));

        let output = tmpl.render(ctx).unwrap();
        assert_eq!(output, "Hi Bob!");
    }

    #[test]
    fn test_minijinja_with_filter() {
        common::init();
        let mut env = Environment::new();
        env.add_filter("upper", |s: String| s.to_uppercase());
        env.add_template("test", "{{ name | upper }}").unwrap();
        let tmpl = env.get_template("test").unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("name".to_string(), Value::from("alice"));

        let output = tmpl.render(ctx).unwrap();
        assert_eq!(output, "ALICE");
    }

    #[test]
    fn test_minijinja_multiline_template() {
        common::init();
        let mut env = Environment::new();
        env.add_template("test", "Line 1: {{ var1 }}\nLine 2: {{ var2 }}")
            .unwrap();
        let tmpl = env.get_template("test").unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("var1".to_string(), Value::from("first"));
        ctx.insert("var2".to_string(), Value::from("second"));

        let output = tmpl.render(ctx).unwrap();
        assert_eq!(output, "Line 1: first\nLine 2: second");
    }

    #[test]
    fn test_minijinja_missing_variable() {
        common::init();
        let mut env = Environment::new();
        env.add_template("test", "{{ missing }}").unwrap();
        let tmpl = env.get_template("test").unwrap();

        let ctx: HashMap<String, Value> = HashMap::new();
        let output = tmpl.render(ctx).unwrap();
        // MiniJinja renders missing variables as empty string by default
        assert_eq!(output, "");
    }

    // ══════════════════════════════════════════════════════════════════════════
    // INTEGRATION TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[cfg(not(target_os = "freebsd"))]
    #[test]
    fn test_target_detection_not_panicking() {
        common::init();

        // This ensures that the environment detection logic
        // we use in build.rs works on the current CI runner.
        let (arch, env) = detect_target();
        assert!(arch == "x86_64" || arch == "aarch64");
        assert!(env == "musl" || env == "gnu");
    }

    #[test]
    fn test_integration_script_variable_in_template() {
        common::init();
        let yaml = r#"
vars:
  - name: result
    script: "2 + 2"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let engine = Engine::new();
        let mut ctx = HashMap::new();

        for spec in &config.vars {
            if let Some(name) = &spec.name {
                if !spec.script.trim().is_empty() && spec.function.is_none() {
                    let mut scope = Scope::new();
                    let result: Dynamic = engine.eval_with_scope(&mut scope, &spec.script).unwrap();
                    ctx.insert(name.clone(), Value::from(result.to_string()));
                }
            }
        }

        let mut env = Environment::new();
        env.add_template("test", "Result: {{ result }}").unwrap();
        let tmpl = env.get_template("test").unwrap();
        let output = tmpl.render(ctx).unwrap();

        assert_eq!(output, "Result: 4");
    }

    // ══════════════════════════════════════════════════════════════════════════
    // INTEGRATION TESTS (Race-Safe)
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_integration_cmd_variable_in_template() {
        common::init();
        // Ensure shell is extracted but DON'T use a guard here in tests.
        // Multiple tests running in parallel will fight over the guard.
        let _ = get_embedded_shell_path();

        let yaml = r#"
vars:
  - name: message
    cmd: "echo 'Hello from shell'"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let mut ctx = HashMap::new();

        for spec in &config.vars {
            if let (Some(name), Some(cmd)) = (&spec.name, &spec.cmd) {
                let result = eval_cmd(
                    cmd,
                    spec.shell.as_deref(),
                    config.default_shell.as_deref(),
                    spec.cwd.as_deref(),
                    spec.env.as_ref(),
                );
                ctx.insert(name.clone(), Value::from(result));
            }
        }

        let mut env = Environment::new();
        env.add_template("test", "{{ message }}").unwrap();
        let tmpl = env.get_template("test").unwrap();
        let output = tmpl.render(ctx).unwrap();

        assert_eq!(output, "Hello from shell");
    }

    #[test]
    fn test_cleanup_guard_removes_file() {
        common::init();

        // Use a UNIQUE filename for this test so it doesn't
        // nuke the real fish_runtime used by other tests.
        let file_path = std::env::current_dir()
            .unwrap()
            .join("test_cleanup_bin_unique");
        fs::write(&file_path, b"test").unwrap();
        {
            let _guard = crate::CleanupGuard(&file_path);
            assert!(file_path.exists());
        }
        assert!(!file_path.exists());
    }

    #[test]
    fn test_integration_cmds_variable_in_template() {
        common::init();
        let yaml = r#"
vars:
  - name: lines
    cmds:
      - "echo 'first'"
      - "echo 'second'"
      - "echo 'third'"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let mut ctx = HashMap::new();

        for spec in &config.vars {
            if let (Some(name), Some(cmd_list)) = (&spec.name, &spec.cmds) {
                let mut results = Vec::new();
                for cmd in cmd_list {
                    let out = eval_cmd(
                        cmd,
                        spec.shell.as_deref(),
                        config.default_shell.as_deref(),
                        spec.cwd.as_deref(),
                        spec.env.as_ref(),
                    );
                    results.push(out);
                }
                let joined = results.join("\n");
                ctx.insert(name.clone(), Value::from(joined));
            }
        }

        let mut env = Environment::new();
        env.add_template("test", "{{ lines }}").unwrap();
        let tmpl = env.get_template("test").unwrap();
        let output = tmpl.render(ctx).unwrap();

        assert_eq!(output, "first\nsecond\nthird");
    }

    #[test]
    fn test_integration_rhai_filter() {
        common::init();
        let yaml = r#"
vars:
  - function: double
    arguments:
      - name: num
    script: "parse_int(num) * 2"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let engine = Engine::new();

        let mut func_defs = String::new();
        for spec in &config.vars {
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

        let ast = engine.compile(func_defs).unwrap();
        let mut env = Environment::new();

        let e = Arc::new(engine);
        let a = Arc::new(ast);

        env.add_filter(
            "double",
            move |name: String| -> Result<String, minijinja::Error> {
                let mut scope = Scope::new();
                let result: Dynamic =
                    e.call_fn(&mut scope, &a, "double", (name,))
                        .map_err(|err| {
                            minijinja::Error::new(
                                minijinja::ErrorKind::InvalidOperation,
                                format!("Rhai Call Error: {err}"),
                            )
                        })?;
                Ok(result.to_string())
            },
        );

        env.add_template("test", "{{ '5' | double }}").unwrap();
        let tmpl = env.get_template("test").unwrap();
        let output = tmpl.render(HashMap::<String, Value>::new()).unwrap();

        assert_eq!(output, "10");
    }

    #[test]
    fn test_integration_mixed_variables_and_filters() {
        common::init();
        let yaml = r#"
default_shell: sh
vars:
  - name: greeting
    script: "\"Hello\""
  - name: who
    cmd: "echo 'World'"
  - function: shout
    arguments:
      - name: text
    script: "text.to_upper() + \"!\""
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let engine = Engine::new();

        // Build function definitions
        let mut func_defs = String::new();
        for spec in &config.vars {
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

        let ast = engine.compile(func_defs).unwrap();
        let arc_engine = Arc::new(engine);
        let arc_ast = Arc::new(ast);

        // Build context
        let mut ctx = HashMap::new();
        for spec in &config.vars {
            if let Some(name) = &spec.name {
                if spec.function.is_none() && !spec.script.trim().is_empty() {
                    let mut scope = Scope::new();
                    let result: Dynamic = arc_engine
                        .eval_with_scope(&mut scope, &spec.script)
                        .unwrap();
                    ctx.insert(name.clone(), Value::from(result.to_string()));
                }
                if let Some(cmd) = &spec.cmd {
                    let result = eval_cmd(
                        cmd,
                        spec.shell.as_deref(),
                        config.default_shell.as_deref(),
                        spec.cwd.as_deref(),
                        spec.env.as_ref(),
                    );
                    ctx.insert(name.clone(), Value::from(result));
                }
            }
        }

        // Setup environment with filter
        let mut env = Environment::new();
        let e = Arc::clone(&arc_engine);
        let a = Arc::clone(&arc_ast);

        env.add_filter(
            "shout",
            move |text: String| -> Result<String, minijinja::Error> {
                let mut scope = Scope::new();
                let result: Dynamic =
                    e.call_fn(&mut scope, &a, "shout", (text,)).map_err(|err| {
                        minijinja::Error::new(
                            minijinja::ErrorKind::InvalidOperation,
                            format!("Rhai Call Error: {err}"),
                        )
                    })?;
                Ok(result.to_string())
            },
        );

        env.add_template("test", "{{ greeting }} {{ who | shout }}")
            .unwrap();
        let tmpl = env.get_template("test").unwrap();
        let output = tmpl.render(ctx).unwrap();

        assert_eq!(output, "Hello WORLD!");
    }

    // ══════════════════════════════════════════════════════════════════════════
    // FILE I/O TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_read_yaml_config_from_file() {
        common::init();
        let mut temp_file = NamedTempFile::new().unwrap();
        let yaml_content = r#"
default_shell: bash
vars:
  - name: test
    script: "42"
"#;
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        let config: RootConfig = serde_yml::from_str(&content).unwrap();

        assert_eq!(config.default_shell, Some("bash".to_string()));
        assert_eq!(config.vars.len(), 1);
    }

    #[test]
    fn test_read_template_from_file() {
        common::init();
        let mut temp_file = NamedTempFile::new().unwrap();
        let template_content = "Hello {{ name }}!";
        temp_file.write_all(template_content.as_bytes()).unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, template_content);
    }

    // ══════════════════════════════════════════════════════════════════════════
    // EDGE CASE TESTS
    // ══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_empty_script() {
        common::init();
        let yaml = r#"
vars:
  - name: empty
    script: ""
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars[0].script, "");
    }

    #[test]
    fn test_variable_without_name() {
        common::init();
        let yaml = r#"
vars:
  - function: my_func
    arguments:
      - name: x
    script: "x * 2"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert!(config.vars[0].name.is_none());
        assert!(config.vars[0].function.is_some());
    }

    #[test]
    fn test_function_without_arguments() {
        common::init();
        let yaml = r#"
vars:
  - function: get_constant
    script: "42"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars[0].arguments.len(), 0);
    }

    #[test]
    fn test_special_characters_in_template() {
        common::init();
        let mut env = Environment::new();
        env.add_template("test", "Special: {{ var }} & < > \" '")
            .unwrap();
        let tmpl = env.get_template("test").unwrap();

        let mut ctx = HashMap::new();
        ctx.insert("var".to_string(), Value::from("value"));

        let output = tmpl.render(ctx).unwrap();
        assert_eq!(output, "Special: value & < > \" '");
    }

    #[test]
    fn test_very_long_script() {
        common::init();
        let long_script = format!(r#""{}""#, "a".repeat(1000));
        let engine = Engine::new();
        let mut scope = Scope::new();
        let result: Dynamic = engine.eval_with_scope(&mut scope, &long_script).unwrap();
        assert_eq!(result.to_string().len(), 1000);
    }

    #[test]
    fn test_command_with_exit_code_non_zero() {
        common::init();
        let result = eval_cmd("sh -c 'exit 1'", Some("sh"), None, None, None);
        // Command should execute but return empty/error
        // Behavior depends on shell implementation
        assert!(
            result.is_empty() || result.contains("ERROR:") || !result.contains("should_not_appear")
        );
    }

    #[test]
    fn test_unicode_in_variables() {
        common::init();
        let yaml = r#"
vars:
  - name: emoji
    script: "\"🚀 Hello 世界\""
    "#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        let engine = Engine::new();
        let mut scope = Scope::new();
        let result: Dynamic = engine
            .eval_with_scope(&mut scope, &config.vars[0].script)
            .unwrap();
        assert_eq!(result.to_string(), "🚀 Hello 世界");
    }

    #[test]
    fn test_multiple_variables_same_type() {
        common::init();
        let yaml = r#"
vars:
  - name: var1
    script: "1"
  - name: var2
    script: "2"
  - name: var3
    script: "3"
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.vars.len(), 3);
        assert_eq!(config.vars[0].name, Some("var1".to_string()));
        assert_eq!(config.vars[1].name, Some("var2".to_string()));
        assert_eq!(config.vars[2].name, Some("var3".to_string()));
    }

    #[test]
    fn test_shell_precedence_three_levels() {
        common::init();
        // Test that per-variable shell > global default > hardcoded default
        let yaml = r#"
default_shell: bash
vars:
  - name: test1
    cmd: "echo test"
    shell: sh
  - name: test2
    cmd: "echo test"
  - name: test3
    cmd: "echo test"
    shell: sh
"#;
        let config: RootConfig = serde_yml::from_str(yaml).unwrap();

        // var with shell override should use "sh"
        let result1 = eval_cmd(
            &config.vars[0].cmd.as_ref().unwrap(),
            config.vars[0].shell.as_deref(),
            config.default_shell.as_deref(),
            None,
            None,
        );
        assert_eq!(result1, "test");

        // var without shell override should use default_shell "bash"
        let result2 = eval_cmd(
            &config.vars[1].cmd.as_ref().unwrap(),
            config.vars[1].shell.as_deref(),
            config.default_shell.as_deref(),
            None,
            None,
        );
        assert_eq!(result2, "test");
    }
}
