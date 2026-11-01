# Invoice Management on Stellar / Soroban

A Rust smart-contract prototype for creating invoices and recording their paid state. The implementation is an **invoice system**, despite the older README describing gate passes and the package retaining the starter name `hello-world`.

## Implemented contract

[contracts/hello-world/src/lib.rs](contracts/hello-world/src/lib.rs) defines `InvoiceManagementContract`:

- `create_invoice`: requires seller authorization and records buyer, amount, description and ledger time.
- `pay_invoice`: requires authorization from the invoice’s buyer and marks the record paid.
- `get_invoice` and `get_stats`: read invoice records and aggregate counters.

**Recording payment does not transfer tokens.** This is a contract experiment, not an audited payment or accounting product. Review amount validation, storage lifetime, overflow behavior and authorization before any deployment.

## Build and test

Install a Rust toolchain compatible with the pinned Soroban SDK. From the repository root:

```sh
cargo test --workspace
cargo build --release
```

The workspace uses Soroban SDK 23.0.2 in [Cargo.toml](Cargo.toml). Contract tests are in [contracts/hello-world/src/test.rs](contracts/hello-world/src/test.rs); the package’s [Makefile](contracts/hello-world/Makefile) contains additional build targets. The contract test module covers positive-amount validation, invoice state/statistics, sequential IDs, seller/buyer authorization, wrong-buyer attempts, repeated payment and missing invoices. GitHub Actions runs `cargo test --locked --workspace` on pull requests and pushes to `main`.

## Layout

| Path | Purpose |
| --- | --- |
| [Cargo.toml](Cargo.toml) | Workspace and release profiles |
| [Cargo.lock](Cargo.lock) | Resolved dependency versions |
| [contracts/hello-world](contracts/hello-world) | Contract source, tests and package manifest |

Start with local tests and a test network. Never commit wallet secrets or present a paid-state flag as proof of settlement. No deployed contract address or security audit is asserted here.
