# Salary-Streams

A decentralized salary streaming smart contract built on the **Stellar blockchain** using the **Soroban SDK**. Salary Streams enables employers to create continuous, real-time salary flows to employees — replacing traditional periodic payroll with programmable, trustless income streams.

---

## Table of Contents

- [Project Title](#-salary-streams)
- [Project Description](#project-description)
- [Project Vision](#project-vision)
- [Key Features](#key-features)
- [Contract Architecture](#contract-architecture)
- [Functions](#functions)
- [Data Structures](#data-structures)
- [Getting Started](#getting-started)
- [Future Scope](#future-scope)

---

## Project Description

**Salary Streams** is a Soroban smart contract that allows employers to lock a defined amount of tokens into a time-bound salary stream for an employee. The tokens become claimable by the employee continuously — second by second — as time progresses on the ledger.

Instead of waiting for a monthly or bi-weekly paycheck, employees can withdraw their earned salary at any point during the stream. Employers retain the ability to cancel a stream early, while the contract automatically marks streams as completed once the full amount has been disbursed or the stream duration ends.

This system eliminates the need for trust between employer and employee by enforcing payment rules transparently on-chain.

---

## Project Vision

> *"Your work earns you money every second — your salary should flow the same way."*

The vision behind **Salary Streams** is to reimagine payroll for the decentralized, on-chain economy. We believe:

- **Workers deserve real-time access** to wages they have already earned.
- **Employers deserve transparency and control** without relying on intermediaries like banks or payroll processors.
- **Smart contracts should replace paperwork** — employment agreements, pay cycles, and salary negotiations can all be encoded and enforced on-chain.

By building on Stellar's Soroban platform, Salary Streams aims to be fast, low-cost, and globally accessible — serving remote workers, freelancers, DAOs, and traditional businesses alike.

---

## Key Features

| Feature | Description |
|---|---|
| ⏱ **Real-Time Streaming** | Salary accrues every second based on a configurable per-second rate |
| 🔐 **Auth-Gated Transactions** | Employer authorization required to create/cancel; employee authorization required to claim |
| 📊 **Global Statistics** | Platform-wide tracking of total, active, and completed streams plus total tokens disbursed |
| 🛑 **Early Cancellation** | Employers can cancel streams at any time; unclaimed tokens are refundable |
| 👁 **Claimable Balance Query** | Employees can check their pending claimable amount without executing a transaction |
| ✅ **Auto-Completion** | Streams automatically deactivate once fully paid out or the end time passes |
| 🧱 **Soroban-Native** | Built entirely with `soroban-sdk`, `#![no_std]`, and on-chain ledger timestamps |

---

## Contract Architecture

```
SalaryStreamsContract
│
├── create_stream(employer, employee, total_amount, duration_seconds) → stream_id
│       Locks tokens into a new salary stream and returns a unique stream ID
│
├── claim_salary(stream_id) → amount_claimed
│       Employee withdraws all tokens earned up to the current ledger timestamp
│
├── cancel_stream(stream_id)
│       Employer cancels an active stream; marks it inactive for refund processing
│
├── view_stream(stream_id) → SalaryStream
│       Returns full details of a salary stream
│
├── view_stats() → StreamStats
│       Returns platform-wide aggregated statistics
│
└── view_claimable(stream_id) → u64
        Returns the currently claimable token balance for a stream (read-only)
```

---

## Functions

### `create_stream`
```rust
pub fn create_stream(
    env: Env,
    employer: Address,
    employee: Address,
    total_amount: u64,
    duration_seconds: u64,
) -> u64
```
Creates a new salary stream. The `salary_per_second` rate is derived as `total_amount / duration_seconds`. Requires employer authorization (`employer.require_auth()`).

---

### `claim_salary`
```rust
pub fn claim_salary(env: Env, stream_id: u64) -> u64
```
Allows the employee to withdraw all tokens earned up to the current ledger timestamp. Automatically closes the stream if the end time has passed or the full amount has been disbursed. Requires employee authorization.

---

### `cancel_stream`
```rust
pub fn cancel_stream(env: Env, stream_id: u64)
```
Allows the employer to prematurely cancel an active stream. Sets `is_active = false`. Remaining unclaimed tokens are made available for refund (token transfer handled by integration layer). Requires employer authorization.

---

### `view_stream`
```rust
pub fn view_stream(env: Env, stream_id: u64) -> SalaryStream
```
Returns the full `SalaryStream` struct for a given stream ID. Panics if the stream does not exist.

---

### `view_stats`
```rust
pub fn view_stats(env: Env) -> StreamStats
```
Returns the `StreamStats` struct with platform-wide totals.

---

### `view_claimable`
```rust
pub fn view_claimable(env: Env, stream_id: u64) -> u64
```
Returns the number of tokens currently claimable by the employee for a given stream. Returns `0` if the stream is inactive.

---

## Data Structures

### `SalaryStream`
```rust
pub struct SalaryStream {
    pub stream_id: u64,          // Unique ID
    pub employer: Address,       // Funding party
    pub employee: Address,       // Receiving party
    pub salary_per_second: u64,  // Payout rate (tokens/second)
    pub total_amount: u64,       // Total tokens locked
    pub amount_claimed: u64,     // Tokens already withdrawn
    pub start_time: u64,         // Stream start (ledger timestamp)
    pub end_time: u64,           // Stream end (ledger timestamp)
    pub is_active: bool,         // Active status flag
}
```

### `StreamStats`
```rust
pub struct StreamStats {
    pub total_streams: u64,      // All streams ever created
    pub active_streams: u64,     // Currently running streams
    pub completed_streams: u64,  // Fully paid-out streams
    pub total_disbursed: u64,    // Lifetime tokens paid out
}
```

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) with `wasm32-unknown-unknown` target
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)
- Stellar Testnet account with funded XLM

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Deploy

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/salary_streams.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

### Invoke — Create a Stream

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <EMPLOYER_SECRET_KEY> \
  --network testnet \
  -- create_stream \
  --employer <EMPLOYER_ADDRESS> \
  --employee <EMPLOYEE_ADDRESS> \
  --total_amount 3600 \
  --duration_seconds 3600
```

### Invoke — Claim Salary

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <EMPLOYEE_SECRET_KEY> \
  --network testnet \
  -- claim_salary \
  --stream_id 1
```

---

## Future Scope

The current implementation is a foundational version of Salary Streams. Below are planned enhancements for future releases:

| Milestone | Description |
|---|---|
| 🪙 **Native Token Integration** | Connect with Stellar's SAC (Stellar Asset Contract) to handle actual on-chain token transfers during `create_stream`, `claim_salary`, and `cancel_stream` |
| 👥 **Multi-Employee Payroll** | Allow a single employer to manage bulk salary streams for entire teams from one transaction |
| 📅 **Pause & Resume Streams** | Give employers the ability to pause a stream temporarily (e.g., unpaid leave) and resume it later |
| 💱 **Stable Token Support** | Support streaming in stablecoins (USDC, EURC) for inflation-resistant salary agreements |
| 🗳 **On-Chain Employment Agreements** | Store hashed employment terms alongside the stream, creating a tamper-proof payroll contract |
| 📱 **Frontend dApp** | A React-based dashboard for employers and employees to create, monitor, and manage salary streams visually |
| 🔔 **Claimable Threshold Alerts** | Emit Soroban events so off-chain indexers and wallets can notify employees when claimable balance reaches a threshold |
| 🌍 **Cross-Chain Bridging** | Bridge salary streams to other EVM-compatible chains for wider ecosystem adoption |
| 🤖 **Auto-Claim Bots** | Optional keeper/relayer integration so employees can auto-claim on a schedule without manual intervention |

---

## License

This project is open source and available under the [MIT License](LICENSE).


#contracts details
contract ID : CC5R2ZENAYINY6EM5Z6D4TBDSR57FUHS7EORIMTBEINZ47BVXOWJBSYG
<img width="1906" height="960" alt="image" src="https://github.com/user-attachments/assets/e648b45f-57e8-45f3-b56a-cabaa48e7fd0" />

