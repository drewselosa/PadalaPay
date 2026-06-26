#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::{Address as _, Events},
        Address, Bytes, BytesN, Env,
    };

    use crate::{PadalaPayContract, PadalaPayContractClient};

    /// Helper: compute a fake "SHA-256" hash for testing.
    /// In real tests sha256 is available through env.crypto().
    fn mock_hash(env: &Env, code: &str) -> BytesN<32> {
        let bytes = Bytes::from_slice(env, code.as_bytes());
        env.crypto().sha256(&bytes).into()
    }

    // ── Test 1: Happy path ────────────────────────────────────────────────────
    /// The full send → claim flow completes successfully and returns the amount.
    #[test]
    fn test_happy_path_send_and_claim() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaPayContract);
        let client = PadalaPayContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let amount: i128 = 5_000_000; // 5 USDC
        let secret = "secret123";
        let code_hash = mock_hash(&env, secret);

        // OFW locks funds.
        client.send(&sender, &recipient, &amount, &code_hash);

        // Recipient claims with the correct code.
        let raw_code = Bytes::from_slice(&env, secret.as_bytes());
        let claimed = client.claim(&sender, &recipient, &raw_code);

        assert_eq!(claimed, amount);
    }

    // ── Test 2: Edge case — wrong release code is rejected ───────────────────
    /// Claiming with an incorrect code must panic and not mark the record claimed.
    #[test]
    #[should_panic(expected = "invalid release code")]
    fn test_wrong_code_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaPayContract);
        let client = PadalaPayContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let code_hash = mock_hash(&env, "correct_code");

        client.send(&sender, &recipient, &1_000_000, &code_hash);

        // Attempt to claim with a wrong code — must panic.
        let wrong = Bytes::from_slice(&env, b"wrong_code");
        client.claim(&sender, &recipient, &wrong);
    }

    // ── Test 3: State verification ────────────────────────────────────────────
    /// After a successful claim, the on-chain record must have claimed = true.
    #[test]
    fn test_state_is_claimed_after_claim() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaPayContract);
        let client = PadalaPayContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let secret = "mysecret";
        let code_hash = mock_hash(&env, secret);

        client.send(&sender, &recipient, &2_000_000, &code_hash);

        let raw = Bytes::from_slice(&env, secret.as_bytes());
        client.claim(&sender, &recipient, &raw);

        let record = client.get_record(&sender, &recipient).unwrap();
        assert!(record.claimed, "record must be marked claimed");
        assert_eq!(record.amount, 2_000_000);
    }

    // ── Test 4: Double-claim is rejected ─────────────────────────────────────
    /// Attempting to claim an already-claimed remittance must panic.
    #[test]
    #[should_panic(expected = "already claimed")]
    fn test_double_claim_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaPayContract);
        let client = PadalaPayContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let secret = "once_only";
        let code_hash = mock_hash(&env, secret);
        let raw = Bytes::from_slice(&env, secret.as_bytes());

        client.send(&sender, &recipient, &1_000_000, &code_hash);
        client.claim(&sender, &recipient, &raw); // first claim OK
        client.claim(&sender, &recipient, &raw); // must panic
    }

    // ── Test 5: Zero-amount send is rejected ─────────────────────────────────
    /// Sending 0 units must be blocked at the contract level.
    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_zero_amount_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PadalaPayContract);
        let client = PadalaPayContractClient::new(&env, &contract_id);

        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);
        let code_hash = mock_hash(&env, "doesnt_matter");

        client.send(&sender, &recipient, &0, &code_hash); // must panic
    }
}