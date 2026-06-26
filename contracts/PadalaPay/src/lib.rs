//! Padala Pay — OFW Remittance Smart Contract
//! Allows an OFW sender to lock USDC for a named recipient,
//! who claims it via a one-time release code stored as a hash.

#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Bytes, BytesN, Env, Symbol,
};

// ── Storage key types ────────────────────────────────────────────────────────

/// Key used to look up a remittance record by sender + recipient pair.
#[contracttype]
#[derive(Clone)]
pub struct RemittanceKey {
    pub sender: Address,
    pub recipient: Address,
}

/// The on-chain remittance record.
#[contracttype]
#[derive(Clone)]
pub struct Remittance {
    /// Amount of stroops-equivalent USDC units locked
    pub amount: i128,
    /// SHA-256 hash of the release code (never store raw codes on-chain)
    pub code_hash: BytesN<32>,
    /// Whether the remittance has already been claimed
    pub claimed: bool,
}

// ── Storage key enum ─────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    /// Maps RemittanceKey → Remittance
    Record(RemittanceKey),
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct PadalaPayContract;

#[contractimpl]
impl PadalaPayContract {
    /// Called by the OFW sender to lock funds for a recipient.
    ///
    /// In a real deployment, fund movement happens via a Stellar
    /// payment before invoking this; here we record the locked amount
    /// and store the hashed release code.
    ///
    /// # Arguments
    /// * `sender`    – the OFW's wallet address (must authorise this call)
    /// * `recipient` – the beneficiary's wallet address in the Philippines
    /// * `amount`    – amount in stroops-equivalent units (e.g. 1_000_000 = 1 USDC)
    /// * `code_hash` – SHA-256 of a secret release code the sender shares OOB
    pub fn send(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        code_hash: BytesN<32>,
    ) {
        // Require the sender to authorise this contract call.
        sender.require_auth();

        // Amount must be positive.
        assert!(amount > 0, "amount must be positive");

        let key = DataKey::Record(RemittanceKey {
            sender: sender.clone(),
            recipient: recipient.clone(),
        });

        // Prevent overwriting an unclaimed remittance.
        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<DataKey, Remittance>(&key)
        {
            assert!(existing.claimed, "an unclaimed remittance already exists");
        }

        // Persist the remittance record.
        env.storage().persistent().set(
            &key,
            &Remittance {
                amount,
                code_hash,
                claimed: false,
            },
        );

        // Emit an event so frontends can track status.
        env.events().publish(
            (symbol_short!("padala"), symbol_short!("sent")),
            (sender, recipient, amount),
        );
    }

    /// Called by the recipient to claim the locked funds using the release code.
    ///
    /// The contract verifies the SHA-256 hash of the provided code matches
    /// what the sender registered.  Actual token transfer must be triggered
    /// by the frontend after successful invocation (or via a token contract
    /// integration added in production).
    ///
    /// # Arguments
    /// * `sender`       – original sender address (needed to locate the record)
    /// * `recipient`    – must authorise this call
    /// * `release_code` – the raw secret code; hashed on-chain for comparison
    pub fn claim(
        env: Env,
        sender: Address,
        recipient: Address,
        release_code: Bytes,
    ) -> i128 {
        // The recipient must sign this transaction.
        recipient.require_auth();

        let key = DataKey::Record(RemittanceKey {
            sender: sender.clone(),
            recipient: recipient.clone(),
        });

        let mut record = env
            .storage()
            .persistent()
            .get::<DataKey, Remittance>(&key)
            .expect("no remittance found");

        assert!(!record.claimed, "already claimed");

        // Hash the provided code and compare.
        let computed: BytesN<32> = env.crypto().sha256(&release_code).into();
        assert!(computed == record.code_hash, "invalid release code");

        // Mark as claimed — actual fund release is handled by the anchor/frontend.
        record.claimed = true;
        env.storage().persistent().set(&key, &record);

        env.events().publish(
            (symbol_short!("padala"), symbol_short!("claimed")),
            (sender, recipient, record.amount),
        );

        record.amount
    }

    /// Returns the remittance record for a given sender–recipient pair.
    pub fn get_record(
        env: Env,
        sender: Address,
        recipient: Address,
    ) -> Option<Remittance> {
        env.storage()
            .persistent()
            .get::<DataKey, Remittance>(&DataKey::Record(RemittanceKey {
                sender,
                recipient,
            }))
    }
}