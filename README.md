# jinja.rs

> A powerful, configuration-driven template rendering engine combining MiniJinja templates with Rhai scripting and shell command execution.

![Licenses](https://github.com/yonasBSD/jinja.rs/actions/workflows/licenses.yaml/badge.svg)
![Linting](https://github.com/yonasBSD/jinja.rs/actions/workflows/lint.yaml/badge.svg)
![Testing](https://github.com/yonasBSD/jinja.rs/actions/workflows/test-with-coverage.yaml/badge.svg)
![Packaging](https://github.com/yonasBSD/jinja.rs/actions/workflows/release-packaging.yaml/badge.svg)
![Cross-Build](https://github.com/yonasBSD/jinja.rs/actions/workflows/cross-build.yaml/badge.svg)

![Security Audit](https://github.com/yonasBSD/jinja.rs/actions/workflows/security.yaml/badge.svg)
![Scorecard Audit](https://github.com/yonasBSD/jinja.rs/actions/workflows/scorecard.yaml/badge.svg)
[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=yonasBSD_jinja.rs&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=yonasBSD_jinja.rs)
[![Security Rating](https://sonarcloud.io/api/project_badges/measure?project=yonasBSD_jinja.rs&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=yonasBSD_jinja.rs)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=yonasBSD_jinja.rs&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=yonasBSD_jinja.rs)
<!--[![codecov](https://codecov.io/gh/yonasBSD/jinja.rs/branch/main/graph/badge.svg?token=SLIHSUWHT2)](https://codecov.io/gh/yonasBSD/jinja.rs)-->
<!--[![ghcr.io](https://img.shields.io/badge/ghcr.io-download-blue)](https://github.com/yonasBSD/jinja.rs/pkgs/container/jinja.rs)-->
<!--[![Docker Pulls](https://img.shields.io/docker/pulls/jinja.rs/example.svg)](https://hub.docker.com/r/jinja.rs/example)-->
<!--[![Quay.io](https://img.shields.io/badge/Quay.io-download-blue)](https://quay.io/repository/jinja.rs/example)-->

![GitHub last commit](https://img.shields.io/github/last-commit/yonasBSD/jinja.rs)
[![Dependency Status](https://deps.rs/repo/github/yonasBSD/jinja.rs/status.svg)](https://deps.rs/repo/github/yonasBSD/jinja.rs)
![Rust](https://img.shields.io/badge/Built%20With-Rust-orange?logo=rust)
[![GitHub Release](https://img.shields.io/github/release/yonasBSD/jinja.rs.svg)](https://github.com/yonasBSD/jinja.rs/releases/latest)
[![License](https://img.shields.io/github/license/yonasBSD/jinja.rs.svg)](https://github.com/yonasBSD/jinja.rs/blob/main/LICENSE.txt)
<!--[![Matrix Chat](https://img.shields.io/matrix/vaultwarden:matrix.org.svg?logo=matrix)](https://matrix.to/#/#vaultwarden:matrix.org)-->

## ✨ Features

* 🎨 **MiniJinja Templates** - Full-featured Jinja2-compatible templating for Rust.
* 🦀 **Rhai Scripting** - Embedded scripting engine for complex, dynamic variable generation.
* 🐚 **Embedded Fish Shell** - Includes a portable **fish shell runtime** embedded in the binary.
    * **Zero Dependencies:** No need to have fish, bash, or zsh installed on the host system.
    * **Consistency:** Ensures shell commands (`cmd`/`cmds`) run identically across Linux, FreeBSD, and OpenBSD.
    * **Auto-Provisioning:** Automatically extracts to `~/.cache/jinja-rs/` and manages permissions on first run.
* ⚙️ **Declarative Configuration** - Define your entire data pipeline in a clean `j2.yaml` file.
* 🎯 **Custom Filters** - Turn any Rhai function into a reusable MiniJinja filter.
* 🔧 **Flexible Execution** - Per-variable shell overrides, custom environment variables, and specific working directories.
* 🚀 **High Performance** - Parallel variable resolution and efficient binary extraction.

## 🚀 Quick Start

### Installation

```bash
cargo install jinja-rs
```

Or build from source:

```bash
git clone https://github.com/yonasBSD/jinja.rs
cd jinja.rs
cargo build --release
```

### Basic Usage

1. Create a configuration file `j2.yaml`:

```yaml
default_shell: bash

vars:
  # Script-based variable
  - name: timestamp
    script: "1234567890"
  
  # Shell command variable
  - name: username
    cmd: "whoami"
  
  # Custom filter function
  - function: upper
    arguments:
      - name: text
    script: "text.to_upper()"
```

2. Create a template `template.j2`:

```jinja
Hello {{ username }}!
Timestamp: {{ timestamp }}
Shouting: {{ username | upper }}
```

3. Render the template:

```bash
jinja-rs --template template.j2
```

Output:
```
Hello alice!
Timestamp: 1234567890
Shouting: ALICE
```

## 📖 Documentation

### Configuration File (`j2.yaml`)

The configuration file drives all behavior. It supports:

#### Global Settings

```yaml
default_shell: bash  # Default shell for command execution (optional)
```

#### Variable Types

**1. Rhai Script Variables**

Execute Rhai scripts to generate values:

```yaml
vars:
  - name: calculation
    script: "2 + 2 * 10"
  
  - name: greeting
    script: "\"Hello, \" + \"World!\""
```

**2. Single Command Variables**

Execute a shell command and capture output:

```yaml
vars:
  - name: hostname
    cmd: "hostname"
  
  - name: current_date
    cmd: "date +%Y-%m-%d"
    shell: sh  # Override default shell
```

**3. Multi-Command Variables**

Execute multiple commands and join results:

```yaml
vars:
  - name: system_info
    cmds:
      - "uname -s"
      - "uname -r"
      - "uname -m"
```

**4. Custom Filters**

Define Rhai functions that become MiniJinja filters:

```yaml
vars:
  - function: reverse
    arguments:
      - name: text
    script: |
      let chars = text.split("");
      chars.reverse();
      chars.join("")
  
  - function: multiply
    arguments:
      - name: value
      - name: factor
    script: "parse_int(value) * parse_int(factor)"
```

Use in templates:
```jinja
{{ "hello" | reverse }}
{{ "5" | multiply(3) }}
```

#### Advanced Configuration

**Environment Variables**

```yaml
vars:
  - name: custom_path
    cmd: "echo $MY_VAR"
    env:
      MY_VAR: "/custom/path"
      ANOTHER: "value"
```

**Working Directory**

```yaml
vars:
  - name: files
    cmd: "ls -la"
    cwd: "/tmp"
```

**Shell Selection Precedence**

1. Per-variable `shell` (highest priority)
2. Global `default_shell`
3. `fish` (hardcoded fallback)

```yaml
default_shell: bash

vars:
  - name: uses_bash
    cmd: "echo $SHELL"
  
  - name: uses_sh
    cmd: "echo $SHELL"
    shell: sh  # Overrides default
```

### Template Syntax

jinja.rs uses MiniJinja, which is compatible with Jinja2:

```jinja
{# Comments #}

{{ variable }}  {# Variable substitution #}

{{ variable | filter }}  {# Apply filter #}

{% if condition %}
  ...
{% endif %}

{% for item in items %}
  {{ item }}
{% endfor %}
```

## 🎯 Use Cases

### Configuration File Generation

Generate Nginx configs, systemd units, or any configuration files:

```yaml
# j2.yaml
vars:
  - name: server_name
    cmd: "hostname -f"
  
  - name: worker_processes
    script: "4"
```

```nginx
# nginx.conf.j2
server {
    server_name {{ server_name }};
    worker_processes {{ worker_processes }};
}
```

### Dynamic Documentation

Create documentation with live system information:

```yaml
vars:
  - name: version
    cmd: "git describe --tags"
  
  - name: build_date
    cmd: "date -u +%Y-%m-%d"
  
  - name: contributors
    cmds:
      - "git log --format='%an' | sort -u | head -5"
```

### DevOps Automation

Generate deployment manifests with environment-specific values:

```yaml
vars:
  - name: environment
    cmd: "echo $DEPLOY_ENV"
  
  - name: replicas
    script: |
      if environment == "prod" { 5 } else { 2 }
```

## 🏗️ Architecture

```mermaid
graph TD
    A[j2.yaml<br/>Configuration] --> B{Variable Type}
    B -->|script| C[Rhai Engine]
    B -->|cmd/cmds| D[Shell Executor]
    B -->|function| E[Rhai Functions]
    
    C --> F[Variables]
    D --> F
    E --> G[Custom Filters]
    
    H[Template .j2] --> I[MiniJinja Engine]
    F --> I
    G --> I
    
    I --> J[Rendered Output]
    
    style A fill:#e1f5ff,stroke:#0288d1,stroke-width:2px
    style H fill:#e1f5ff,stroke:#0288d1,stroke-width:2px
    style I fill:#fff9c4,stroke:#f57f17,stroke-width:2px
    style J fill:#c8e6c9,stroke:#388e3c,stroke-width:2px
    style C fill:#ffe0b2,stroke:#e64a19,stroke-width:2px
    style D fill:#ffe0b2,stroke:#e64a19,stroke-width:2px
    style E fill:#ffe0b2,stroke:#e64a19,stroke-width:2px
```

### Design Principles

1. **Configuration over Code** - All logic defined in YAML, no code changes needed
2. **Separation of Concerns** - Variables, filters, and templates are independent
3. **Composability** - Mix Rhai scripts, shell commands, and template logic freely
4. **Fail-Safe** - Errors are captured and reported, not silently ignored

## 🧪 Testing

Comprehensive test suite with 60+ tests covering:

- Configuration deserialization
- Command execution with various shells
- Rhai script evaluation
- MiniJinja template rendering
- Integration scenarios
- Edge cases and error handling

Run tests:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

See [TESTING.md](TESTING.md) for detailed test documentation.

## 🛣️ Roadmap

- [ ] CLI argument for config file path (currently hardcoded to `j2.yaml`)
- [ ] Template auto-discovery
- [ ] Multi-template rendering in one invocation
- [ ] JSON/TOML config format support
- [ ] Watch mode for live reloading
- [ ] Built-in filter library
- [ ] Plugin system for custom functions
- [ ] Performance optimizations for large-scale rendering

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yonasBSD/jinja.rs
cd jinja.rs

# Run tests
cargo test

# Run with example
cargo run -- --template examples/demo.j2

# Build release
cargo build --release
```

## 📝 License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

- [MiniJinja](https://github.com/mitsuhiko/minijinja) - Jinja2 template engine for Rust
- [Rhai](https://github.com/rhaiscript/rhai) - Embedded scripting language
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing

## 📧 Contact

- **Author**: yonasBSD
- **Repository**: [github.com/yonasBSD/jinja.rs](https://github.com/yonasBSD/jinja.rs)
- **Issues**: [github.com/yonasBSD/jinja.rs/issues](https://github.com/yonasBSD/jinja.rs/issues)

---

<p align="center">Made with ❤️ and 🦀</p>
