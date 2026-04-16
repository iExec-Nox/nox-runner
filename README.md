# Nox · Runner

[![License](https://img.shields.io/badge/license-BUSL--1.1-blue)](./LICENSE) [![Docs](https://img.shields.io/badge/docs-nox--protocol-purple)](https://docs.iex.ec) [![Discord](https://img.shields.io/badge/chat-Discord-5865F2)](https://discord.com/invite/5TewNUnJHN) [![Ship](https://img.shields.io/github/v/tag/iExec-Nox/nox-runner?label=ship)](https://github.com/iExec-Nox/nox-runner/releases)

> Off-chain computation worker for confidential operations in the Nox Protocol.

## Table of Contents

- [Nox · Runner](#nox--runner)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Prerequisites](#prerequisites)
  - [Getting Started](#getting-started)
  - [Environment Variables](#environment-variables)
  - [API Reference](#api-reference)
    - [Service Endpoints](#service-endpoints)
      - [`GET /`](#get-)
      - [`GET /health`](#get-health)
      - [`GET /metrics`](#get-metrics)
    - [NATS Interface](#nats-interface)
      - [Message Format](#message-format)
      - [Supported Operators](#supported-operators)
        - [Encryption](#encryption)
        - [Arithmetic](#arithmetic)
        - [Boolean Comparisons](#boolean-comparisons)
        - [Control Flow](#control-flow)
        - [Token Operations](#token-operations)
  - [Related Repositories](#related-repositories)
  - [License](#license)

---

## Overview

The Runner is the off-chain computation layer of the Nox Protocol. It subscribes to a NATS JetStream fed by [nox-ingestor](https://github.com/iExec-Nox/nox-ingestor), executes confidential operations over encrypted values, and publishes results back to [nox-handle-gateway](https://github.com/iExec-Nox/nox-handle-gateway). Plaintext values are never persisted and exist only transiently in memory during a computation.

**Receiving a computation request:** The runner consumes `TransactionMessage` objects from a NATS JetStream pull consumer. Each message represents a single on-chain transaction and contains one or more ordered `TransactionEvent` objects, each carrying a typed `Operator` variant. On startup, the runner reads the KMS public key and the Handle Gateway signer address from the `NoxCompute` contract on-chain to bootstrap its crypto context.

**Executing computations:** For each event in the transaction, the runner fetches the encrypted operand handles from the Handle Gateway via `GET /v0/compute/operands`, decrypts them locally with ECIES (secp256k1 ECDH + HKDF-SHA256 + AES-256-GCM), performs the computation on plaintext Solidity-typed values, and re-encrypts each result under the KMS public key.

**Operand caching:** Within a single transaction, decrypted values are kept in an in-memory cache. If a result produced by one event is needed as an operand by a subsequent event in the same transaction, it is served from cache without an additional gateway round-trip. The cache is cleared after each transaction.

**Publishing results:** Once all events in a transaction are processed, all result handles are submitted to the Handle Gateway in a single `POST /v0/compute/results` call. This preserves transaction atomicity: results are either all published or none are.

**Supported types:** The runner operates on Solidity-compatible types encoded as 32-byte big-endian values — `bool`, `uint16`, `uint256`, `int16`, and `int256`.

---

## Prerequisites

- Rust >= 1.85 (edition 2024)
- Access to an Ethereum RPC endpoint with a deployed [NoxCompute](https://github.com/iExec-Nox/nox-protocol-contracts) contract
- A running [nox-handle-gateway](https://github.com/iExec-Nox/nox-handle-gateway) instance
- A running NATS server with JetStream enabled and a stream populated by [nox-ingestor](https://github.com/iExec-Nox/nox-ingestor)

---

## Getting Started

```bash
git clone https://github.com/iExec-Nox/nox-runner.git
cd nox-runner

# Set required environment variables
export NOX_RUNNER_WALLET_KEY="0x..."
export NOX_RUNNER_RPC_URL="https://..."
export NOX_RUNNER_NOX_COMPUTE_CONTRACT_ADDRESS="0x..."
export NOX_RUNNER_HANDLE_GATEWAY_URL="https://..."
export NOX_RUNNER_NATS_URL="nats://..."

# Build and run
cargo run --release
```

---

## Environment Variables

Configuration is loaded from environment variables with the `NOX_RUNNER_` prefix. Nested properties use double underscore (`__`) as separator.

| Variable | Description | Required | Default |
| -------- | ----------- | -------- | ------- |
| `NOX_RUNNER_SERVER__HOST` | Bind address for the HTTP server | No | `127.0.0.1` |
| `NOX_RUNNER_SERVER__PORT` | Port for the HTTP server | No | `8080` |
| `NOX_RUNNER_CHAIN_ID` | Chain ID for EIP-712 signing | No | `421614` (Arbitrum Sepolia) |
| `NOX_RUNNER_RPC_URL` | Ethereum RPC endpoint for reading the `NoxCompute` contract | No | `http://localhost:8545` |
| `NOX_RUNNER_NOX_COMPUTE_CONTRACT_ADDRESS` | `NoxCompute` contract address | No | `0x0000...0000` |
| `NOX_RUNNER_NATS__URL` | NATS server URL | No | `nats://localhost:4222` |
| `NOX_RUNNER_NATS__STREAM_NAME` | Name of the JetStream stream to consume | No | `nox_ingestor` |
| `NOX_RUNNER_NATS__CONSUMER_NAME` | Durable consumer name | No | `nox_ingestor_consumer` |
| `NOX_RUNNER_NATS__CONSUMER_MAX_DELIVER` | Maximum redelivery attempts per message | No | `10` |
| `NOX_RUNNER_NATS__MAX_ACK_PENDING` | Buffer size of unacknowledged messages | No | `10` |
| `NOX_RUNNER_NATS__MAX_BATCH` | Maximum number of messages the runner can pull from the stream | No | `10` |
| `NOX_RUNNER_HANDLE_GATEWAY_URL` | Handle Gateway base URL | No | `http://localhost:3000` |
| `NOX_RUNNER_WALLET_KEY` | Private key used to sign Handle Gateway requests (hex, with or without `0x` prefix) | **Yes** | — |

Logging level is controlled via the `RUST_LOG` environment variable:

```bash
RUST_LOG=info    # Default
RUST_LOG=debug   # Verbose logging
```

---

## API Reference

### Service Endpoints

#### `GET /`

Returns basic service information.

**Response:**

```json
{
  "service": "Runner",
  "timestamp": "2026-02-25T10:30:00.000Z"
}
```

#### `GET /health`

Health check endpoint for monitoring and orchestration.

**Response:**

```json
{
  "status": "ok"
}
```

#### `GET /metrics`

Prometheus metrics endpoint for observability.

**Response:** Prometheus text format metrics.

The following Nox Runner metrics are available:

| Metric | Description |
| ------ | ----------- |
| `nox_runner.transaction.received` | Counter to count each required transaction computation. |
| `nox_runner.transaction.block_number` | Blockchain block number of the last transaction to compute. |
| `nox_runner.operation` | Counter to count each operation. An `operator` label allows to distinguish all operators. |
| `nox_runner.transaction.result` | Counter to observe computation results following 3 statuses (`SUCCESS`, `FAILURE`, `NOT_ACK`). |

---

### NATS Interface

The runner consumes messages from a NATS JetStream pull consumer. It does not produce any NATS messages. Messages are published to the stream by [nox-ingestor](https://github.com/iExec-Nox/nox-ingestor).

#### Message Format

Each message payload is a JSON-encoded `TransactionMessage`:

```json
{
  "chainId": 421614,
  "blockNumber": 12345678,
  "caller": "0x...",
  "transactionHash": "0x...",
  "events": [
    {
      "logIndex": 0,
      "caller": "0x...",
      "type": "add",
      "leftHandOperand": "0x...",
      "rightHandOperand": "0x...",
      "result": "0x..."
    }
  ]
}
```

| Field | Description |
| ----- | ----------- |
| `chainId` | Chain ID where the on-chain events were emitted |
| `blockNumber` | Block number of the transaction |
| `caller` | Address of the account that sent the transaction |
| `transactionHash` | Hash of the on-chain transaction |
| `events` | Ordered list of confidential operations to execute, one per emitted event log |

Each entry in `events` carries a `type` discriminant that selects the operator, plus the handle fields required for that operator. Events are processed in `logIndex` order.

#### Supported Operators

##### Encryption

| Type | Description | Fields |
| ---- | ----------- | ------ |
| `plaintext_to_encrypted` | Encrypts a plaintext `bytes32` value and stores it under a result handle | `value`, `teeType`, `handle` |
| `wrap_as_public_handle` | Same as `plaintext_to_encrypted` for handles marked as publicly decryptable | `value`, `teeType`, `handle` |

##### Arithmetic

| Type | Description | Fields |
| ---- | ----------- | ------ |
| `add` | Addition | `leftHandOperand`, `rightHandOperand`, `result` |
| `sub` | Subtraction | `leftHandOperand`, `rightHandOperand`, `result` |
| `mul` | Multiplication | `leftHandOperand`, `rightHandOperand`, `result` |
| `div` | Division (returns `MAX` on division by zero) | `leftHandOperand`, `rightHandOperand`, `result` |
| `safe_add` | Addition with overflow detection | `leftHandOperand`, `rightHandOperand`, `success`, `result` |
| `safe_sub` | Subtraction with underflow detection | `leftHandOperand`, `rightHandOperand`, `success`, `result` |
| `safe_mul` | Multiplication with overflow detection | `leftHandOperand`, `rightHandOperand`, `success`, `result` |
| `safe_div` | Division with division-by-zero detection | `leftHandOperand`, `rightHandOperand`, `success`, `result` |

Safe variants produce two result handles: `success` (a `bool` indicating whether the operation did not overflow) and `result` (the computed value).

##### Boolean Comparisons

| Type | Description | Fields |
| ---- | ----------- | ------ |
| `eq` | Equal | `leftHandOperand`, `rightHandOperand`, `result` |
| `ne` | Not equal | `leftHandOperand`, `rightHandOperand`, `result` |
| `ge` | Greater than or equal | `leftHandOperand`, `rightHandOperand`, `result` |
| `gt` | Greater than | `leftHandOperand`, `rightHandOperand`, `result` |
| `le` | Less than or equal | `leftHandOperand`, `rightHandOperand`, `result` |
| `lt` | Less than | `leftHandOperand`, `rightHandOperand`, `result` |

All comparison operators produce a single `bool` result handle.

##### Control Flow

| Type | Description | Fields |
| ---- | ----------- | ------ |
| `select` | Ternary selection — returns `ifTrue` if `condition` is non-zero, `ifFalse` otherwise | `condition`, `ifTrue`, `ifFalse`, `result` |

##### Token Operations

| Type | Description | Fields |
| ---- | ----------- | ------ |
| `transfer` | ERC-20-equivalent transfer between two encrypted balances | `balanceFrom`, `balanceTo`, `amount`, `success`, `newBalanceFrom`, `newBalanceTo` |
| `mint` | ERC-20-equivalent mint into an encrypted balance | `balanceTo`, `amount`, `totalSupply`, `success`, `newBalanceTo`, `newTotalSupply` |
| `burn` | ERC-20-equivalent burn from an encrypted balance | `balanceFrom`, `amount`, `totalSupply`, `success`, `newBalanceFrom`, `newTotalSupply` |

Token operations produce a `success` bool handle plus updated balance handle(s). A transfer fails (success = false) when `balanceFrom < amount`. A mint or burn fails when it would cause the total supply to overflow or underflow respectively.

---

## Related Repositories

| Repository | Role |
| ---------- | ---- |
| [nox-ingestor](https://github.com/iExec-Nox/nox-ingestor) | Event ingestor — listens for on-chain `NoxCompute` events and publishes `TransactionMessage` objects to the NATS stream |
| [nox-handle-gateway](https://github.com/iExec-Nox/nox-handle-gateway) | Handle Gateway — provides encrypted operands to the runner and stores result handles |
| [nox-protocol-contracts](https://github.com/iExec-Nox/nox-protocol-contracts) | On-chain contracts — the `NoxCompute` contract exposes the KMS public key and gateway address read by the runner on startup |

---

## License

The Nox Protocol source code is released under the Business Source License 1.1 (BUSL-1.1).

The license will automatically convert to the MIT License under the conditions described in the [LICENSE](./LICENSE) file.

The full text of the MIT License is provided in the [LICENSE-MIT](./LICENSE-MIT) file.
