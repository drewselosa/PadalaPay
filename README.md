# Padala Pay

> Instant, trust-minimised OFW remittances to Philippine families — no bank account required.

---

## Problem

An OFW nurse in Riyadh sends ₱5,000 home every week through a remittance centre that charges 5–8% in fees, takes 1–3 days, and requires her sister in Bulacan to travel to a pawnshop — losing half a day's income just to collect.

## Solution

Padala Pay lets the OFW lock USDC on Stellar in seconds (< $0.01 fee), then sends a one-time release code via SMS to the family member, who claims the funds directly from a mobile-first web app — no bank account, no pawnshop queue.

---

## Stellar Features Used

| Feature | Purpose |
|---|---|
| USDC transfers | Stablecoin remittance (no FX volatility) |
| Soroban smart contracts | Lock funds, enforce hashed-code claim logic |
| Trustlines | Recipient wallet opts in to USDC before claiming |
| XLM | Gas for transactions (fractions of a cent) |

---

## Timeline

| Phase | Deliverable |
|---|---|
| Day 1 | Contract + unit tests passing locally |
| Day 2 | Deploy to testnet; wire up React frontend |
| Day 3 | End-to-end demo: send → SMS code → claim |

---

## Vision and Purpose

Remittances account for ~9% of Philippine GDP yet families lose billions yearly to middlemen. Padala Pay removes the intermediary layer, making cross-border value transfer as simple as sending a text message — building on Stellar's speed and near-zero cost.

---

## Prerequisites

- Rust `1.74+` with WASM target: `rustup target add wasm32-unknown-unknown`
- Stellar CLI: `cargo install --locked stellar-cli`
- Freighter Wallet browser extension (testnet mode)

---

## Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

---

## Test

```bash
cargo test
```

All 5 tests should pass.

---

## Deploy to Testnet

```bash
# Generate and fund an identity
stellar keys generate --global my-key --network testnet
stellar keys fund my-key --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/padala_pay.wasm \
  --source my-key \
  --network testnet
```

Copy the `C...` contract ID from the output.

---

## Sample CLI Invocation

### Send a remittance

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source my-key \
  --network testnet \
  -- send \
  --sender GABCD...SENDER \
  --recipient GXYZ...RECIPIENT \
  --amount 5000000 \
  --code_hash aabbcc...32bytehex
```

### Claim a remittance

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source my-key \
  --network testnet \
  -- claim \
  --sender GABCD...SENDER \
  --recipient GXYZ...RECIPIENT \
  --release_code "secret123"
```

---

## License

MIT