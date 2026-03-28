# TalentTrust Contracts

Soroban smart contracts for the TalentTrust decentralized freelancer escrow protocol on the Stellar network.

## What's in this repo

- **Escrow contract** (`contracts/escrow`): Holds funds in escrow, supports milestone-based payments and reputation credential issuance.

## Escrow refund support

The escrow contract now supports partial refunds for unreleased milestone balances.

- `create_contract` stores milestone definitions and initializes tracked balances.
- `deposit_funds` accepts client-authorized deposits up to the contract total.
- `release_milestone` marks a milestone as paid and decreases the refundable escrow balance.
- `refund_unreleased_milestones` refunds any selected unreleased milestones back to the client and prevents double refund or refund-after-release.
- `get_contract`, `get_milestones`, and `get_refundable_balance` expose review and integration state without mutating storage.

Reviewer notes:

- Refunds are milestone-based, not arbitrary-amount based, so the remaining funded balance always maps to unresolved milestones.
- Double spend protection is enforced through milestone flags plus balance accounting.
- Terminal contracts reject further deposit, release, and refund actions.
- Detailed contract behavior and security assumptions are documented in [docs/escrow/README.md](/c:/Users/ADMIN/Desktop/midea-drips/Talenttrust-Contracts/docs/escrow/README.md).

## Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.75+)
- `rustfmt`: `rustup component add rustfmt`
- Optional: [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli) for deployment

## Setup

```bash
# Clone (or you're already in the repo)
git clone <your-repo-url>
cd talenttrust-contracts

# Build
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all
```

## Contributing

1. Fork the repo and create a branch from `main`.
2. Make changes; keep tests and formatting passing:
   - `cargo fmt --all`
   - `cargo test`
   - `cargo build`
3. Open a pull request. CI runs `cargo fmt --all -- --check`, `cargo build`, and `cargo test` on push/PR to `main`.

## CI/CD

On every push and pull request to `main`, GitHub Actions:

- Checks formatting (`cargo fmt --all -- --check`)
- Builds the workspace (`cargo build`)
- Runs tests (`cargo test`)

Ensure these pass locally before pushing.

## Escrow testing

The escrow test suite is organized by behavior area:

- contract creation validation
- deposits and overfunding protection
- milestone release paths
- partial refund and refund failure cases

Run the escrow-specific suite with:

```bash
cargo test -p escrow
```

## License

MIT
