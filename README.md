# TalentTrust Contracts

Soroban smart contracts for the TalentTrust decentralized freelancer escrow protocol on the Stellar network.

## What's in this repo

- **Escrow contract** (`contracts/escrow`): Holds funds in escrow, supports milestone-based payments, reputation credential issuance, and **dispute resolution mechanism**.

## Features

### Core Escrow Functionality
- Create escrow contracts with milestone-based payments
- Deposit and release funds securely
- Issue reputation credentials

### Dispute Resolution Mechanism
- **Admin/Arbitrator roles**: Secure access control for dispute resolution
- **Deterministic payout outcomes**: Four resolution types with predictable results
  - `FullRefund`: Client gets 100% refund
  - `PartialRefund`: Client gets 70%, freelancer gets 30%
  - `FullPayout`: Freelancer gets 100%
  - `Split`: Custom split determined by arbitrator
- **Evidence tracking**: Store dispute reasons and evidence
- **Secure workflow**: Only authorized parties can create and resolve disputes

## Security Features

- **Access control**: Role-based permissions for admin, arbitrator, client, and freelancer
- **State validation**: Contracts must be in correct states for operations
- **Deterministic payouts**: Mathematical guarantees for fund distribution
- **Authorization checks**: All operations require proper authentication
- **Input validation**: Prevents invalid splits and unauthorized actions

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

## License

MIT
