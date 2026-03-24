# Design Document: Dispute Initiation Workflow

## Overview

This document describes the design for adding a dispute initiation workflow to the TalentTrust escrow smart contract. The contract is written in Rust using the Soroban SDK and deployed on the Stellar network.

The feature introduces:
- A `DisputeRecord` data type stored immutably in persistent storage.
- A `DisputeError` enum for typed error handling.
- An `initiate_dispute` entry point with authorization enforcement and state-transition logic.
- A `get_dispute` query function.
- Full persistent storage of escrow state (client, freelancer, status) to support access control and status checks.

## Architecture

The contract follows the existing single-file Soroban pattern (`lib.rs`). All state is stored in `env.storage().persistent()` keyed by typed enum variants. No external dependencies are added beyond the existing `soroban-sdk`.

```
┌─────────────────────────────────────────────────────┐
│                  Escrow Contract                    │
│                                                     │
│  initiate_dispute(env, contract_id, initiator,      │
│                   reason)                           │
│    ├─ initiator.require_auth()                      │
│    ├─ load EscrowState from storage                 │
│    ├─ validate initiator == client || freelancer    │
│    ├─ validate status in {Funded, Completed}        │
│    ├─ write status = Disputed                       │
│    └─ write DisputeRecord (immutable)               │
│                                                     │
│  get_dispute(env, contract_id)                      │
│    └─ return Option<DisputeRecord> from storage     │
└─────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Storage Keys

```rust
#[contracttype]
pub enum DataKey {
    EscrowState(u32),   // maps contract_id -> EscrowState
    Dispute(u32),       // maps contract_id -> DisputeRecord
}
```

### Data Models

```rust
/// Full state of an escrow contract stored on-chain.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowState {
    pub client: Address,
    pub freelancer: Address,
    pub status: ContractStatus,
    pub milestones: Vec<Milestone>,
}

/// Immutable record created when a dispute is initiated.
/// Written once and never overwritten.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DisputeRecord {
    /// The address (client or freelancer) that initiated the dispute.
    pub initiator: Address,
    /// A short human-readable reason for the dispute.
    pub reason: String,
    /// Ledger timestamp at the moment the dispute was recorded.
    pub timestamp: u64,
}
```

### Error Enum

```rust
/// Typed errors returned by dispute-related contract functions.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeError {
    /// The escrow contract was not found in storage.
    NotFound = 1,
    /// The caller is not the client or freelancer of this escrow.
    Unauthorized = 2,
    /// The escrow status does not allow dispute initiation (e.g. Created).
    InvalidStatus = 3,
    /// A dispute record already exists for this escrow.
    AlreadyDisputed = 4,
}
```

### Public Interface

```rust
/// Initiate a dispute on an existing escrow.
pub fn initiate_dispute(
    env: Env,
    contract_id: u32,
    initiator: Address,
    reason: String,
) -> Result<(), DisputeError>

/// Retrieve the dispute record for an escrow, if one exists.
pub fn get_dispute(
    env: Env,
    contract_id: u32,
) -> Option<DisputeRecord>
```

## Data Models

See "Components and Interfaces" above. Key design decisions:

- `EscrowState` is introduced to persist client/freelancer addresses and status so that `initiate_dispute` can enforce access control and status checks. Previously these were stub functions with no storage.
- `DisputeRecord` is written with `env.storage().persistent().set(...)` and guarded by an existence check so it is written exactly once.
- `reason` is a `soroban_sdk::String` (bounded, no-std compatible).

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

---

**Property 1: Valid status transition**

*For any* escrow whose status is `Funded` or `Completed`, calling `initiate_dispute` with a valid initiator (client or freelancer) and any non-empty reason SHALL result in the escrow status becoming `Disputed`.

**Validates: Requirements 1.1, 1.2**

---

**Property 2: Unauthorized caller rejection**

*For any* escrow and *for any* address that is neither the client nor the freelancer of that escrow, calling `initiate_dispute` SHALL return `DisputeError::Unauthorized` and SHALL NOT mutate the escrow status or create a dispute record.

**Validates: Requirements 1.5, 4.3**

---

**Property 3: Dispute record round-trip**

*For any* valid `initiate_dispute` call that succeeds, a subsequent call to `get_dispute` with the same `contract_id` SHALL return a `DisputeRecord` whose `initiator` and `reason` fields match the inputs provided to `initiate_dispute`, and whose `timestamp` is greater than zero.

**Validates: Requirements 2.1, 2.3**

---

**Property 4: Unauthenticated call rejection**

*For any* call to `initiate_dispute` where `require_auth` is not satisfied (no authorization provided), the contract SHALL panic/revert before performing any state mutation, leaving the escrow state unchanged.

**Validates: Requirements 4.1, 4.2**

---

## Error Handling

| Scenario | Error returned |
|---|---|
| `contract_id` not in storage | `DisputeError::NotFound` |
| Initiator ≠ client and ≠ freelancer | `DisputeError::Unauthorized` |
| Status is `Created` | `DisputeError::InvalidStatus` |
| Status is already `Disputed` | `DisputeError::AlreadyDisputed` |
| `require_auth` fails | Soroban auth panic (automatic revert) |

## Testing Strategy

### Framework

- Unit and property-based tests use the Soroban SDK's built-in `testutils` feature (`soroban-sdk = { features = ["testutils"] }`).
- Property-based tests use the [`proptest`](https://crates.io/crates/proptest) crate (added as a `dev-dependency`).
- Each property-based test runs a minimum of 100 iterations.

### Unit Tests (in `contracts/escrow/src/test.rs`)

Cover specific examples and edge cases:
- Dispute initiation from client on a `Funded` escrow succeeds.
- Dispute initiation from freelancer on a `Funded` escrow succeeds.
- Dispute on `Created` escrow returns `InvalidStatus`.
- Dispute on already-`Disputed` escrow returns `AlreadyDisputed`.
- `get_dispute` on a non-disputed escrow returns `None`.
- `get_dispute` after successful dispute returns correct record.

### Property-Based Tests (in `contracts/escrow/src/dispute_tests.rs`)

Each property-based test is tagged with the format:
`// Feature: dispute-initiation-workflow, Property {N}: {property_text}`

- **Property 1** — Generate random `(client, freelancer, status ∈ {Funded, Completed}, reason)`. Assert status becomes `Disputed` after `initiate_dispute`.
- **Property 2** — Generate random `(escrow, third_party_address)` where third party ≠ client and ≠ freelancer. Assert `initiate_dispute` returns `Unauthorized` and state is unchanged.
- **Property 3** — Generate random valid `initiate_dispute` inputs. Assert `get_dispute` returns a record with matching `initiator`, `reason`, and `timestamp > 0`.
- **Property 4** — Call `initiate_dispute` without setting up `mock_auths`. Assert the call panics/reverts and storage is unchanged.
