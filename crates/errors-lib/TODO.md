# TODOs

## Error Oriented Programmiing

New standard scripting language. Program towards exceptions before they arise.

- Collect a large library of exception handlers.
  - cross-platform, usable in any programming environment
- Embeds into existing programming languages
- Future-proofing
  - network-based operating systems will require resilient programs that never go down


### Error Handling
- errors bumble up and down the tech stack
  - no boundaries, go beyond parent process
  - errors travel across the entire system, into network, etc.
- do not have to crash
  - no more core dumps
- critical section
  - prove you can handle most common errors before being allowed to access a resource


### Error Messaging
- globally unique error codes
  - includes nano-second timestamp
- security
  - integrity
    - message framing and hashing
      - received the entire message
    - append-only
      - blockchain merkle tree
  - authentication
    - signed
  - privacy
    - encrypted


### Self-Diagnostics
- LLM
  - provides solutions to the problem
- status monitor (all green)
  - background process


### Self-Healing
- patch existing source code
  - add tests: unit, integration, regression, e2e
  - replace running process
- playbooks
  - codified fallback plans
  - eg. S3 is down
    - Plan A: a) use local cache b) update S3 when service restored
    - Plan B: ...
    - Plan C: ...
- proactive
  - solve problems before user hits a wall

```json
{
  "$schema": "https://error-oriented-programming.org/schema/v1",
  "definitions": {
    "ErrorDefinition": {
      "type": "object",
      "required": ["uuid", "namespace", "code", "severity", "definedAt"],
      "properties": {
        "uuid": {
          "type": "string",
          "format": "uuid",
          "description": "Globally unique identifier for this error type"
        },
        "namespace": {
          "type": "string",
          "pattern": "^[a-z0-9]+(\\.[a-z0-9]+)*$",
          "description": "Reverse-DNS style namespace (e.g., com.acme.payments.gateway)"
        },
        "code": {
          "type": "string",
          "pattern": "^[A-Z_][A-Z0-9_]*$",
          "description": "Human-readable error code (e.g., PAYMENT_GATEWAY_TIMEOUT)"
        },
        "severity": {
          "type": "string",
          "enum": ["fatal", "error", "warning", "info"],
          "description": "Error severity level"
        },
        "definedAt": {
          "type": "string",
          "format": "date-time",
          "description": "ISO 8601 timestamp with nanosecond precision when error was defined"
        },
        "version": {
          "type": "integer",
          "minimum": 1,
          "default": 1,
          "description": "Schema version for this error definition"
        },
        "deprecated": {
          "type": "boolean",
          "default": false,
          "description": "Whether this error type is deprecated"
        },
        "replacedBy": {
          "type": "string",
          "format": "uuid",
          "description": "UUID of replacement error if deprecated"
        },
        "category": {
          "type": "string",
          "enum": [
            "network",
            "database",
            "authentication",
            "authorization",
            "validation",
            "resource",
            "timeout",
            "rate_limit",
            "internal",
            "external_service",
            "data_integrity",
            "configuration"
          ],
          "description": "Error category for classification"
        },
        "title": {
          "type": "string",
          "description": "Short human-readable title"
        },
        "description": {
          "type": "string",
          "description": "Detailed description of when this error occurs"
        },
        "userMessage": {
          "type": "string",
          "description": "End-user friendly message template"
        },
        "documentation": {
          "type": "string",
          "format": "uri",
          "description": "URL to detailed documentation"
        },
        "retryable": {
          "type": "boolean",
          "default": false,
          "description": "Whether the operation can be safely retried"
        },
        "retryPolicy": {
          "type": "object",
          "properties": {
            "maxAttempts": {
              "type": "integer",
              "minimum": 1
            },
            "backoffStrategy": {
              "type": "string",
              "enum": ["linear", "exponential", "fibonacci"]
            },
            "initialDelayMs": {
              "type": "integer",
              "minimum": 0
            },
            "maxDelayMs": {
              "type": "integer",
              "minimum": 0
            }
          }
        },
        "playbooks": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["name", "steps"],
            "properties": {
              "name": {
                "type": "string",
                "description": "Playbook identifier (e.g., 'Plan A', 'Fallback')"
              },
              "priority": {
                "type": "integer",
                "minimum": 1,
                "description": "Execution priority (1 = highest)"
              },
              "conditions": {
                "type": "array",
                "items": {
                  "type": "string"
                },
                "description": "Conditions when this playbook applies"
              },
              "steps": {
                "type": "array",
                "items": {
                  "type": "string"
                },
                "description": "Ordered remediation steps"
              },
              "automatable": {
                "type": "boolean",
                "default": false,
                "description": "Whether steps can be automated"
              }
            }
          }
        },
        "metadata": {
          "type": "object",
          "properties": {
            "owner": {
              "type": "string",
              "description": "Team or individual responsible"
            },
            "sla": {
              "type": "object",
              "properties": {
                "responseTimeMinutes": {
                  "type": "integer"
                },
                "resolutionTimeMinutes": {
                  "type": "integer"
                }
              }
            },
            "tags": {
              "type": "array",
              "items": {
                "type": "string"
              }
            }
          }
        },
        "relatedErrors": {
          "type": "array",
          "items": {
            "type": "string",
            "format": "uuid"
          },
          "description": "UUIDs of related error definitions"
        }
      }
    },

    "ErrorInstance": {
      "type": "object",
      "required": ["errorUuid", "instanceId", "occurredAt", "context"],
      "properties": {
        "errorUuid": {
          "type": "string",
          "format": "uuid",
          "description": "References the ErrorDefinition UUID"
        },
        "instanceId": {
          "type": "string",
          "format": "uuid",
          "description": "Unique ID for this specific occurrence"
        },
        "traceId": {
          "type": "string",
          "description": "Distributed trace ID (OpenTelemetry compatible)"
        },
        "spanId": {
          "type": "string",
          "description": "Span ID within the trace"
        },
        "occurredAt": {
          "type": "string",
          "format": "date-time",
          "description": "ISO 8601 timestamp with nanosecond precision"
        },
        "context": {
          "type": "object",
          "required": ["host", "process", "environment"],
          "properties": {
            "host": {
              "type": "string",
              "description": "Hostname or container ID"
            },
            "process": {
              "type": "object",
              "properties": {
                "pid": {"type": "integer"},
                "name": {"type": "string"},
                "version": {"type": "string"}
              }
            },
            "environment": {
              "type": "string",
              "enum": ["production", "staging", "development", "test"]
            },
            "region": {
              "type": "string",
              "description": "Geographical region or availability zone"
            },
            "user": {
              "type": "object",
              "properties": {
                "id": {"type": "string"},
                "sessionId": {"type": "string"}
              }
            }
          }
        },
        "stackTrace": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "file": {"type": "string"},
              "line": {"type": "integer"},
              "column": {"type": "integer"},
              "function": {"type": "string"}
            }
          }
        },
        "additionalData": {
          "type": "object",
          "description": "Error-specific contextual data"
        },
        "causedBy": {
          "type": "string",
          "format": "uuid",
          "description": "Instance ID of the error that caused this one"
        },
        "signature": {
          "type": "string",
          "description": "Cryptographic signature for integrity verification"
        },
        "merkleRoot": {
          "type": "string",
          "description": "Merkle tree root for append-only verification"
        }
      }
    }
  }
}
```

## 💡 Integration Checklist

When building out the logic with these dependencies, keep these three tips in mind:

  - Feature Gating: If your library is meant to be used in both CLI and Web contexts, consider gating miette/fancy behind a cli feature and problemo behind a web feature to keep the binary small.
  - SourceSpan Mapping: To get Ariadne to point to the right place, your SNAFU error variants should store a miette::SourceSpan. You can then use the #[label] attribute to make the terminal output pop.
  - The Panic Hook: Don't forget to initialize color_eyre::install()? and miette::set_panic_hook() in your main function to ensure that even unhandled crashes use your beautiful new diagnostic format.
