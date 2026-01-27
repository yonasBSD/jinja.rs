# Test Suite Documentation

This project includes a comprehensive test suite with **60+ professional, robust tests** covering all major components and edge cases.

## Test Categories

### 1. Configuration Deserialization (12 tests)
Tests for parsing and validating YAML configuration files:
- `test_deserialize_empty_config` - Handles empty configuration files
- `test_deserialize_config_with_default_shell` - Parses global default shell setting
- `test_deserialize_script_variable` - Loads script-based variables
- `test_deserialize_cmd_variable` - Loads single command variables
- `test_deserialize_cmds_variable` - Loads multi-command variables
- `test_deserialize_function_with_arguments` - Parses Rhai function definitions with arguments
- `test_deserialize_variable_with_env_override` - Handles environment variable overrides
- `test_deserialize_variable_with_cwd_override` - Handles working directory overrides
- `test_deserialize_variable_with_shell_override` - Handles per-variable shell overrides
- `test_deserialize_complex_config` - Parses complex multi-variable configurations

### 2. Command Execution (12 tests)
Tests for shell command execution with various configurations:
- `test_eval_cmd_simple_echo` - Basic command execution
- `test_eval_cmd_with_global_default_shell` - Uses global default shell
- `test_eval_cmd_hardcoded_fish_fallback` - Falls back to fish when no shell specified
- `test_eval_cmd_shell_override_precedence` - Verifies shell precedence rules
- `test_eval_cmd_with_cwd` - Executes commands in specific working directories
- `test_eval_cmd_with_env_vars` - Passes environment variables to commands
- `test_eval_cmd_with_multiple_env_vars` - Handles multiple environment variables
- `test_eval_cmd_trims_whitespace` - Trims whitespace from command output
- `test_eval_cmd_invalid_command` - Handles invalid commands gracefully
- `test_eval_cmd_multiline_output` - Preserves multiline command output
- `test_eval_cmd_empty_output` - Handles commands with no output

### 3. Rhai Script Evaluation (6 tests)
Tests for Rhai scripting engine integration:
- `test_rhai_simple_arithmetic` - Evaluates arithmetic expressions
- `test_rhai_string_operations` - Performs string manipulation
- `test_rhai_function_compilation` - Compiles and executes Rhai functions
- `test_rhai_multiple_functions` - Handles multiple function definitions
- `test_rhai_string_function` - Executes string-based functions
- `test_rhai_invalid_script` - Handles syntax errors gracefully

### 4. MiniJinja Template Rendering (6 tests)
Tests for template engine functionality:
- `test_minijinja_simple_variable` - Renders basic variable substitution
- `test_minijinja_multiple_variables` - Handles multiple variables in templates
- `test_minijinja_with_filter` - Applies filters to variables
- `test_minijinja_multiline_template` - Renders multiline templates
- `test_minijinja_missing_variable` - Handles missing variables gracefully

### 5. Integration Tests (6 tests)
Tests that verify end-to-end functionality:
- `test_integration_script_variable_in_template` - Rhai script → template rendering
- `test_integration_cmd_variable_in_template` - Shell command → template rendering
- `test_integration_cmds_variable_in_template` - Multiple commands → template rendering
- `test_integration_rhai_filter` - Rhai function as MiniJinja filter
- `test_integration_mixed_variables_and_filters` - Complex scenario with multiple variable types and filters

### 6. File I/O (2 tests)
Tests for file system operations:
- `test_read_yaml_config_from_file` - Reads YAML configuration from files
- `test_read_template_from_file` - Reads template content from files

### 7. Edge Cases (12 tests)
Tests for unusual inputs and boundary conditions:
- `test_empty_script` - Handles empty script definitions
- `test_variable_without_name` - Handles filter-only variables (no name)
- `test_function_without_arguments` - Handles zero-argument functions
- `test_special_characters_in_template` - Preserves special characters in output
- `test_unicode_in_variables` - Handles Unicode and emoji characters
- `test_very_long_script` - Handles very long script content
- `test_command_with_exit_code_non_zero` - Handles command failures
- `test_multiple_variables_same_type` - Handles multiple variables of the same type
- `test_shell_precedence_three_levels` - Verifies complete shell precedence hierarchy

## Running the Tests

```sh
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_deserialize_empty_config

# Run tests in a specific module
cargo test integration
```

## Test Features

- **Isolated unit tests** - Each component tested independently
- **Integration tests** - Verify component interactions work correctly
- **Edge case coverage** - Tests unusual inputs and boundary conditions
- **Error handling verification** - Ensures graceful failure on invalid inputs
- **Temporary files** - Uses `tempfile` crate to avoid side effects
- **Clear naming** - Test names clearly describe what's being tested
- **Comprehensive assertions** - Verifies all expected behavior

## Coverage

The test suite provides excellent coverage for:
- Regression testing during refactoring
- Validating new features don't break existing functionality
- Documentation of expected behavior
- Confidence in production deployments
