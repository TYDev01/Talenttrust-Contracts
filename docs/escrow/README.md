# TalentTrust Escrow Contract

## Overview 
The TalentTrust Escrow contract manages milestone-based payments and state transitions between a `Client` and `Freelancer`.

## State Machine
The contract follows strict state transitions:
- `Created`: Initial state. Awaiting funding.
- `Funded`: Escrow holds the total amount required for all milestones.
- `Completed`: All milestones have been released successfully.
- `Disputed`: Flagged for dispute resolution.

## Invariants & Property-Based Tests
We use `proptest` to enforce correctness across a massive variety of scenarios.
Properties tested:
1. **Balance Consistency**: `Current Balance + Total Released Amount == Total Deposited Amount`
2. **Status Consistency**: Contract strictly transitions according to the state machine above. No out-of-order state transitions allowed.
3. **Milestone Logic**: Releasing an already-released milestone is impossible.
4. **Funding Validity**: Depositing less than the total milestone sum is rejected.

To run the property tests, use:
```bash
cargo test
```
