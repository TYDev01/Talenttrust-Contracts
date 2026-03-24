# Implementation Plan

- [x] 1. Set up persistent storage and core data models
  - Add `DataKey` enum with `EscrowState(u32)` and `Dispute(u32)` variants to `lib.rs`
  - Add `EscrowState` struct (client, freelancer, status, milestones) with `#[contracttype]`
  - Add `DisputeRecord` struct (initiator, reason, timestamp) with `#[contracttype]` and doc comments
  - Add `DisputeError` enum with `#[contracterror]` and doc comments for all four variants
  - Update `create_contract` to persist an `EscrowState` in `env.storage().persistent()`
  - _Requirements: 2.1, 2.2, 3.2, 3.3_

- [x] 2. Implement `initiate_dispute` entry point
  - [x] 2.1 Implement authorization and state-load logic
    - Call `initiator.require_auth()` as the first operation
    - Load `EscrowState` from persistent storage; return `DisputeError::NotFound` if absent
    - Validate initiator equals `state.client` or `state.freelancer`; return `DisputeError::Unauthorized` otherwise
    - _Requirements: 1.5, 4.1, 4.3_
  - [x] 2.2 Implement status-transition and immutable record write
    - Validate `state.status` is `Funded` or `Completed`; return `DisputeError::InvalidStatus` for `Created`
    - Return `DisputeError::AlreadyDisputed` if a `DisputeRecord` already exists for this `contract_id`
    - Set `state.status = ContractStatus::Disputed` and persist updated `EscrowState`
    - Build and persist a `DisputeRecord` with `initiator`, `reason`, and `env.ledger().timestamp()`
    - Add `///` doc comments to the function
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 2.2, 3.1_
  - [ ]* 2.3 Write property test for Property 1 (valid status transition)
    - **Property 1: Valid status transition**
    - **Validates: Requirements 1.1, 1.2**
  - [ ]* 2.4 Write property test for Property 4 (unauthenticated call rejection)
    - **Property 4: Unauthenticated call rejection**
    - **Validates: Requirements 4.1, 4.2**

- [x] 3. Implement `get_dispute` query function
  - Add `get_dispute(env: Env, contract_id: u32) -> Option<DisputeRecord>` to `lib.rs`
  - Read from `env.storage().persistent()` using `DataKey::Dispute(contract_id)`
  - Return `Some(record)` if found, `None` otherwise
  - Add `///` doc comments
  - _Requirements: 2.3, 2.4, 3.1_
  - [ ]* 3.1 Write property test for Property 3 (dispute record round-trip)
    - **Property 3: Dispute record round-trip**
    - **Validates: Requirements 2.1, 2.3**

- [x] 4. Write unit tests covering all acceptance criteria
  - Add tests to `contracts/escrow/src/test.rs`:
    - `test_initiate_dispute_from_client` — client initiates on Funded escrow, status becomes Disputed
    - `test_initiate_dispute_from_freelancer` — freelancer initiates on Funded escrow, status becomes Disputed
    - `test_dispute_on_created_escrow_fails` — returns `InvalidStatus`
    - `test_dispute_already_disputed_fails` — returns `AlreadyDisputed`
    - `test_get_dispute_no_record` — returns `None` before any dispute
    - `test_get_dispute_returns_record` — returns correct record after dispute
  - [ ]* 4.1 Write property test for Property 2 (unauthorized caller rejection)
    - **Property 2: Unauthorized caller rejection**
    - **Validates: Requirements 1.5, 4.3**

- [ ] 5. Checkpoint — Ensure all tests pass, ask the user if questions arise.

- [x] 6. Update documentation
  - Update `README.md` to describe the dispute workflow, new functions, and error codes
  - Create `docs/escrow/dispute-workflow.md` with full function reference, state machine diagram, and security notes
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 7. Final Checkpoint — Ensure all tests pass, ask the user if questions arise.
