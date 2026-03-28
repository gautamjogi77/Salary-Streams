#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, Address, symbol_short};

// Tracks global salary stream statistics
#[contracttype]
#[derive(Clone)]
pub struct StreamStats {
    pub total_streams: u64,    // Total salary streams ever created
    pub active_streams: u64,   // Currently active/running streams
    pub completed_streams: u64,// Streams fully paid out and closed
    pub total_disbursed: u64,  // Total tokens disbursed across all streams (in smallest unit)
}

// Reference key for global stats
const ALL_STATS: Symbol = symbol_short!("ALL_STATS");

// Maps a stream_id to its SalaryStream data
#[contracttype]
pub enum Streambook {
    Stream(u64),
}

// Counter for generating unique stream IDs
const COUNT_STREAM: Symbol = symbol_short!("C_STREAM");

// Represents a single salary stream between an employer and employee
#[contracttype]
#[derive(Clone)]
pub struct SalaryStream {
    pub stream_id: u64,          // Unique ID for this stream
    pub employer: Address,       // Employer who funds the stream
    pub employee: Address,       // Employee receiving salary
    pub salary_per_second: u64,  // Token payout rate per ledger second
    pub total_amount: u64,       // Total tokens locked into the stream
    pub amount_claimed: u64,     // Tokens already withdrawn by employee
    pub start_time: u64,         // Ledger timestamp when stream started
    pub end_time: u64,           // Ledger timestamp when stream ends
    pub is_active: bool,         // Whether the stream is currently active
}

#[contract]
pub struct SalaryStreamsContract;

#[contractimpl]
impl SalaryStreamsContract {

    /// Create a new salary stream from employer to employee.
    /// `total_amount`      – total tokens to stream (in smallest unit, e.g. stroops)
    /// `duration_seconds`  – how long the stream runs (in seconds)
    /// Returns the new stream_id.
    pub fn create_stream(
        env: Env,
        employer: Address,
        employee: Address,
        total_amount: u64,
        duration_seconds: u64,
    ) -> u64 {
        employer.require_auth();

        assert!(total_amount > 0,       "total_amount must be > 0");
        assert!(duration_seconds > 0,   "duration_seconds must be > 0");
        assert!(employer != employee,   "employer and employee must differ");

        let mut count: u64 = env.storage().instance().get(&COUNT_STREAM).unwrap_or(0);
        count += 1;

        let now = env.ledger().timestamp();
        let salary_per_second = total_amount / duration_seconds; // integer division

        let stream = SalaryStream {
            stream_id: count,
            employer: employer.clone(),
            employee: employee.clone(),
            salary_per_second,
            total_amount,
            amount_claimed: 0,
            start_time: now,
            end_time: now + duration_seconds,
            is_active: true,
        };

        let mut stats = Self::view_stats(env.clone());
        stats.total_streams  += 1;
        stats.active_streams += 1;

        env.storage().instance().set(&Streambook::Stream(count), &stream);
        env.storage().instance().set(&ALL_STATS, &stats);
        env.storage().instance().set(&COUNT_STREAM, &count);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Stream created: id={}, employer={}, employee={}, total={}, rate={}/s",
            count, employer, employee, total_amount, salary_per_second);

        count
    }

    /// Employee calls this to withdraw all tokens earned up to the current moment.
    /// Returns the amount claimed in this transaction.
    pub fn claim_salary(env: Env, stream_id: u64) -> u64 {
        let mut stream = Self::view_stream(env.clone(), stream_id);
        stream.employee.require_auth();

        assert!(stream.is_active, "Stream is not active");

        let now = env.ledger().timestamp();
        // Cap earning calculation at the stream end time
        let effective_time = if now >= stream.end_time { stream.end_time } else { now };
        let elapsed = effective_time.saturating_sub(stream.start_time);
        let earned   = elapsed * stream.salary_per_second;
        let claimable = earned.saturating_sub(stream.amount_claimed);

        assert!(claimable > 0, "Nothing to claim yet");

        stream.amount_claimed += claimable;

        // Auto-complete stream if fully paid out
        let mut stats = Self::view_stats(env.clone());
        if stream.amount_claimed >= stream.total_amount || now >= stream.end_time {
            stream.is_active = false;
            stats.active_streams    = stats.active_streams.saturating_sub(1);
            stats.completed_streams += 1;
            log!(&env, "Stream {} fully completed", stream_id);
        }
        stats.total_disbursed += claimable;

        env.storage().instance().set(&Streambook::Stream(stream_id), &stream);
        env.storage().instance().set(&ALL_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Claimed {} tokens from stream {}", claimable, stream_id);
        claimable
    }

    /// Employer can cancel an active stream early.
    /// Remaining (unclaimed) tokens would be refunded to the employer off-chain
    /// or handled by a token contract integration in a full implementation.
    pub fn cancel_stream(env: Env, stream_id: u64) {
        let mut stream = Self::view_stream(env.clone(), stream_id);
        stream.employer.require_auth();

        assert!(stream.is_active, "Stream is already inactive");

        stream.is_active = false;

        let mut stats = Self::view_stats(env.clone());
        stats.active_streams    = stats.active_streams.saturating_sub(1);
        // Cancelled streams don't count as completed
        stats.completed_streams += 0;

        env.storage().instance().set(&Streambook::Stream(stream_id), &stream);
        env.storage().instance().set(&ALL_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        let refundable = stream.total_amount.saturating_sub(stream.amount_claimed);
        log!(&env, "Stream {} cancelled. Refundable amount: {}", stream_id, refundable);
    }

    // ── View helpers ──────────────────────────────────────────────────────────

    /// Returns details of a specific salary stream by its ID.
    pub fn view_stream(env: Env, stream_id: u64) -> SalaryStream {
        env.storage().instance().get(&Streambook::Stream(stream_id))
            .unwrap_or_else(|| panic!("Stream not found: {}", stream_id))
    }

    /// Returns global platform-wide salary stream statistics.
    pub fn view_stats(env: Env) -> StreamStats {
        env.storage().instance().get(&ALL_STATS).unwrap_or(StreamStats {
            total_streams:      0,
            active_streams:     0,
            completed_streams:  0,
            total_disbursed:    0,
        })
    }

    /// Returns how many tokens the employee can claim right now.
    pub fn view_claimable(env: Env, stream_id: u64) -> u64 {
        let stream = Self::view_stream(env.clone(), stream_id);
        if !stream.is_active {
            return 0;
        }
        let now = env.ledger().timestamp();
        let effective_time = if now >= stream.end_time { stream.end_time } else { now };
        let elapsed  = effective_time.saturating_sub(stream.start_time);
        let earned   = elapsed * stream.salary_per_second;
        earned.saturating_sub(stream.amount_claimed)
    }
}