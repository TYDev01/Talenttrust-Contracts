# Requirements Document

## Introduction

This feature implements the dispute initiation workflow for the TalentTrust escrow smart contract on the Stellar/Soroban network. It introduces dispute state transitions (from `Funded` or `Completed` to `Disputed`) and immutable dispute records that capture the initiating party, reason, and timestamp. The implementation must be secure, well-tested, and aligned with the existing Soroban contract architecture.

## Glossary

- **Escrow**: The TalentTrust smart contract that holds funds on behalf of a client and freelancer until milestones are completed or a dispute is resolved.
- **Client**: The party who creates the escrow and deposits funds.
- **Freelancer**: The party who performs work and receives milestone payments.
- **Dispute**: A formal on-chain record indicating that the client or freelancer has raised a conflict over the escrow contract.
- **DisputeRecord**: An immutable data structure stored in contract persistent storage that captures who initiated the dispute, the reason, and the ledger timestamp.
- **ContractStatus**: An enum representing the lifecycle state of an escrow: `Created`, `Funded`, `Completed`, `Disputed`.
- **Initiator**: The `Address` (client or freelancer) who calls `initiate_dispute`.
- **Ledger timestamp**: The `env.ledger().timestamp()` value at the time the dispute is recorded — used as the immutable creation time.
- **Soroban SDK**: The Rust SDK for writing smart contracts on the Stellar network (`soroban-sdk`).
- **Persistent storage**: `env.storage().persistent()` — the Soroban storage tier that survives ledger archival and is used for long-lived contract state.

## Requirements

### Requirement 1

**User Story:** As a client or freelancer, I want to initiate a dispute on a funded or active escrow, so that I can formally signal a conflict and halt further automatic payments.

#### Acceptance Criteria

1. WHEN a client or freelancer calls `initiate_dispute` on an escrow in `Funded` status, THE Escrow contract SHALL transition the escrow status to `Disputed`.
2. WHEN a client or freelancer calls `initiate_dispute` on an escrow in `Completed` status, THE Escrow contract SHALL transition the escrow status to `Disputed`.
3. IF `initiate_dispute` is called on an escrow whose status is `Created`, THEN THE Escrow contract SHALL reject the call and return an error indicating the contract is not yet funded.
4. IF `initiate_dispute` is called on an escrow that is already in `Disputed` status, THEN THE Escrow contract SHALL reject the call and return an error indicating a dispute is already active.
5. WHEN `initiate_dispute` is called by an address that is neither the client nor the freelancer of the escrow, THE Escrow contract SHALL reject the call and return an authorization error.

---

### Requirement 2

**User Story:** As an auditor or arbitrator, I want dispute records to be immutable and queryable on-chain, so that I can verify the history of a dispute without trusting any off-chain source.

#### Acceptance Criteria

1. WHEN a dispute is successfully initiated, THE Escrow contract SHALL store a `DisputeRecord` in persistent storage containing the initiator address, a reason string, and the ledger timestamp.
2. WHEN a `DisputeRecord` is written to persistent storage, THE Escrow contract SHALL write it exactly once and SHALL NOT overwrite an existing record for the same escrow.
3. THE Escrow contract SHALL expose a `get_dispute` function that, for any escrow identifier, returns the stored `DisputeRecord` if one exists.
4. WHEN `get_dispute` is called for an escrow with no dispute record, THE Escrow contract SHALL return a result indicating no dispute exists.

---

### Requirement 3

**User Story:** As a developer integrating with TalentTrust, I want the dispute workflow to be documented with structured doc comments, so that I can understand the contract interface without reading the full source.

#### Acceptance Criteria

1. THE Escrow contract SHALL include structured doc comments (Rust `///` style) on every public function added or modified as part of this feature.
2. THE Escrow contract SHALL include a `DisputeRecord` type definition with doc comments describing each field.
3. THE Escrow contract SHALL include an error enum (`DisputeError`) with doc comments describing each variant.

---

### Requirement 4

**User Story:** As a security reviewer, I want the dispute initiation to enforce caller authorization, so that only legitimate parties can raise a dispute.

#### Acceptance Criteria

1. WHEN `initiate_dispute` is called, THE Escrow contract SHALL call `initiator.require_auth()` to enforce Soroban-level authorization before performing any state mutation.
2. WHEN authorization fails, THE Escrow contract SHALL revert all state changes and propagate the authorization error to the caller.
3. THE Escrow contract SHALL validate that the authorized initiator matches either the stored client address or the stored freelancer address for the given escrow.
