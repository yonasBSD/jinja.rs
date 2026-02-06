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
    - schema
      - JSON Schema
    - append-only
      - blockchain merkle tree
    - MessagePack / CBOR
      - fast, small, typed JSON object format
    - ZFS backed file
      - BLAKE-3 checksums
      - RAID-Z1/Z2/Z3 + mirrors self-healing
  - authentication
    - minisign
  - privacy
    - ZFS encrypted
- workbench
  - standard datasets
    - /source-code
    - /build-info
      - version
      - OS env
      - CLI flags
      - build flags and full log
    - /findings
    - /solutions
      - work-around
      - PR
      - regression tests
    - /comments
    - /links
- compressed
  - ZSTD-19


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

# Error Definition Example
---

```yaml
$schema: "https://error-oriented-programming.org/schema/v1"
uuid: "550e8400-e29b-41d4-a716-446655440000"
namespace: "com.acme.payments.gateway"
code: "PAYMENT_GATEWAY_TIMEOUT"
severity: "error"
definedAt: "2024-03-15T14:32:17.123456789Z"
version: 2
deprecated: false
replacedBy: null
category: "timeout"
title: "Payment Gateway Timeout"
description: "Occurs when the external payment gateway fails to respond within the configured timeout period (default 30s). This typically indicates network issues, gateway overload, or service degradation."
userMessage: "We're having trouble processing your payment right now. Please try again in a few moments."
documentation: "https://docs.acme.com/errors/payment-gateway-timeout"
retryable: true
retryPolicy:
  maxAttempts: 3
  backoffStrategy: "exponential"
  initialDelayMs: 1000
  maxDelayMs: 16000
playbooks:
  - name: "Plan A: Cache and Queue"
    priority: 1
    conditions:
      - "gateway.status == 'degraded'"
      - "transaction.amount < 1000.00"
    steps:
      - "Store transaction in local cache with TTL of 1 hour"
      - "Add transaction to retry queue with exponential backoff"
      - "Send user confirmation with pending status"
      - "Monitor gateway status endpoint every 30 seconds"
      - "Retry transaction when gateway returns to healthy status"
      - "Update user via webhook/email when transaction completes"
    automatable: true

  - name: "Plan B: Fallback Gateway"
    priority: 2
    conditions:
      - "gateway.status == 'down'"
      - "fallback_gateway.status == 'healthy'"
      - "transaction.amount < 5000.00"
    steps:
      - "Route transaction to fallback payment gateway (Stripe)"
      - "Log gateway failover event with original error details"
      - "Update metrics dashboard with failover counter"
      - "Alert on-call engineer if failover count > 10 in 5 minutes"
    automatable: true

  - name: "Plan C: Manual Intervention"
    priority: 3
    conditions:
      - "all_gateways.status == 'down'"
      - "transaction.amount >= 5000.00"
    steps:
      - "Create high-priority ticket in incident management system"
      - "Page on-call payment team lead"
      - "Store transaction in secure offline queue"
      - "Send customer service notification with transaction details"
      - "Require manual approval before retry"
      - "Generate incident report after resolution"
    automatable: false

metadata:
  owner: "payments-platform-team@acme.com"
  sla:
    responseTimeMinutes: 15
    resolutionTimeMinutes: 120
  tags:
    - "payments"
    - "external-dependency"
    - "customer-facing"
    - "sla-critical"
    - "pci-scope"

relatedErrors:
  - "660e8400-e29b-41d4-a716-446655440111"  # PAYMENT_GATEWAY_UNAVAILABLE
  - "770e8400-e29b-41d4-a716-446655440222"  # PAYMENT_AUTHORIZATION_FAILED
  - "880e8400-e29b-41d4-a716-446655440333"  # NETWORK_CONNECTION_ERROR
```

---
# Error Instance Example
---

```yaml
errorUuid: "550e8400-e29b-41d4-a716-446655440000"
instanceId: "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
traceId: "4bf92f3577b34da6a3ce929d0e0e4736"
spanId: "00f067aa0ba902b7"
occurredAt: "2026-02-06T18:45:32.987654321Z"

context:
  host: "payment-service-prod-us-east-1-pod-42"
  process:
    pid: 18472
    name: "payment-processor"
    version: "v3.14.2"
  environment: "production"
  region: "us-east-1"
  user:
    id: "usr_7h3k9s2l1m4n"
    sessionId: "sess_x8y9z0a1b2c3d4e5"

stackTrace:
  - file: "/app/src/payment/gateway_client.rs"
    line: 247
    column: 18
    function: "execute_payment_request"
  - file: "/app/src/payment/gateway_client.rs"
    line: 189
    column: 9
    function: "send_with_timeout"
  - file: "/app/src/http/client.rs"
    line: 412
    column: 23
    function: "post_async"
  - file: "/app/src/http/client.rs"
    line: 156
    column: 12
    function: "await_response"

additionalData:
  transactionId: "txn_9876543210"
  gatewayEndpoint: "https://api.payment-gateway.example.com/v2/charge"
  requestMethod: "POST"
  timeoutMs: 30000
  actualDurationMs: 30142
  httpStatusCode: null
  gatewayRequestId: "gw_req_abc123xyz789"
  amount: 249.99
  currency: "USD"
  merchantId: "mch_acme_retail_001"
  cardType: "visa"
  cardLast4: "4242"
  attemptNumber: 1
  gatewayHealthCheckStatus: "degraded"
  fallbackGatewayAvailable: true

causedBy: "550e8400-aaaa-bbbb-cccc-dddddddddddd"  # Network socket timeout instance

signature: "3045022100f1e8a9c2d5b3e7f4a6c8d9e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e90220"
merkleRoot: "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
```

### Key design decisions

- Two-tier system: ErrorDefinition (the type) vs ErrorInstance (runtime occurrence). Definitions are checked into source control, instances are logged.
- Namespace hierarchy: Reverse-DNS allows organizational boundaries and prevents collisions.
- Versioning built-in: Supports schema evolution without breaking compatibility.
- Playbooks as first-class: Your self-healing concept integrated directly, with automation flags.
- OpenTelemetry compatibility: traceId/spanId allow integration with existing observability tools.
- Security primitives: signature and merkleRoot fields support your blockchain/integrity requirements.
- Retry semantics: Declarative retry policies avoid scattered retry logic.

## 💡 Integration Checklist

When building out the logic with these dependencies, keep these three tips in mind:

  - Feature Gating: If your library is meant to be used in both CLI and Web contexts, consider gating miette/fancy behind a cli feature and problemo behind a web feature to keep the binary small.
  - SourceSpan Mapping: To get Ariadne to point to the right place, your SNAFU error variants should store a miette::SourceSpan. You can then use the #[label] attribute to make the terminal output pop.
  - The Panic Hook: Don't forget to initialize color_eyre::install()? and miette::set_panic_hook() in your main function to ensure that even unhandled crashes use your beautiful new diagnostic format.
