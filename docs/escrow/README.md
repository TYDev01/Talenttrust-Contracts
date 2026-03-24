# Escrow Contract

Soroban smart contract implementing milestone-based escrow with a freelancer acceptance handshake for the TalentTrust protocol.

---

## Overview

The escrow contract mediates payment between a client and a freelancer. Funds are locked until both parties have agreed to the engagement, enforced by a two-step handshake before any value is committed.

---

## State machine

```
Created ──► Accepted ──► Funded ──► Completed
                                └──► Disputed
```

| Status      | Description                                                |
| ----------- | ---------------------------------------------------------- |
| `Created`   | Contract initialised by the client; no funds involved yet. |
| `Accepted`  | Freelancer has explicitly accepted; client may now fund.   |
| `Funded`    | Client deposited funds; milestones are releasable.         |
| `Completed` | All milestones released.                                   |
| `Disputed`  | Under dispute resolution.                                  |

---

## Data structures

### `ContractRecord`

| Field        | Type             | Description                              |
| ------------ | ---------------- | ---------------------------------------- |
| `client`     | `Address`        | Party that created and funds the escrow. |
| `freelancer` | `Address`        | Party that accepts and delivers work.    |
| `milestones` | `Vec<Milestone>` | Ordered list of payment milestones.      |
| `status`     | `ContractStatus` | Current lifecycle state.                 |

### `Milestone`

| Field      | Type   | Description                          |
| ---------- | ------ | ------------------------------------ |
| `amount`   | `i128` | Payment amount for this deliverable. |
| `released` | `bool` | Whether funds have been released.    |

---

## Functions

### `create_contract`

```
create_contract(env, client, freelancer, milestone_amounts) -> u32
```

Creates a new escrow engagement. Returns the `contract_id`.

- **Auth**: `client` must authorise.
- **Preconditions**: `milestone_amounts` must be non-empty.
- **Post-state**: `Created`.

---

### `accept_contract`

```
accept_contract(env, contract_id)
```

Freelancer accepts the terms of the contract. This is the required handshake before any funds can be deposited. Without this call, `deposit_funds` will always fail.

- **Auth**: `freelancer` (as stored in the record) must authorise.
- **Preconditions**: Status must be `Created`.
- **Post-state**: `Accepted`.

---

### `deposit_funds`

```
deposit_funds(env, contract_id, amount) -> bool
```

Client deposits funds into escrow.

- **Auth**: `client` must authorise.
- **Preconditions**: Status must be `Accepted`; `amount > 0`.
- **Post-state**: `Funded`.

> **Security note**: The guard on `Accepted` status is the enforcement point of the handshake. A contract stuck in `Created` cannot receive funds, preventing the client from funding a deal the freelancer has not agreed to.

---

### `release_milestone`

```
release_milestone(env, contract_id, milestone_id) -> bool
```

Releases a single milestone payment to the freelancer.

- **Auth**: `client` must authorise.
- **Preconditions**: Status must be `Funded`; `milestone_id` in range; milestone not yet released.
- **Post-state**: `Funded` (milestone marked released).

---

### `get_status`

```
get_status(env, contract_id) -> ContractStatus
```

Returns the current status of a contract. Panics if the `contract_id` does not exist.

---

### `issue_reputation`

```
issue_reputation(env, freelancer, rating) -> bool
```

Issues a reputation credential after contract completion.

---

## Security considerations

| Threat                                                 | Mitigation                                                                                     |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| Client funds a contract the freelancer never agreed to | `deposit_funds` asserts `status == Accepted`; impossible without prior `accept_contract` call. |
| Wrong party calls `accept_contract`                    | `freelancer.require_auth()` enforces the stored freelancer address.                            |
| Wrong party deposits funds                             | `client.require_auth()` enforces the stored client address.                                    |
| Replay / double-acceptance                             | State guard (`Created` only) prevents a second call from succeeding.                           |
| Zero or negative deposit                               | Explicit `amount > 0` assertion.                                                               |
| Out-of-range or double milestone release               | Range check + `released` flag guard.                                                           |

---

## Test coverage

All branches are covered by `contracts/escrow/src/test.rs`:

- Happy-path: full `Created → Accepted → Funded` flow
- Funding without acceptance (must fail)
- Double acceptance (must fail)
- Accepting a funded contract (must fail)
- Zero / negative deposit (must fail)
- Non-existent contract IDs
- Milestone out-of-range / double-release
- Multiple independent contracts with independent states
