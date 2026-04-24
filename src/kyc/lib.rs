#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, Bytes, BytesN, xdr::ToXdr};

#[contracttype]
pub enum DataKey {
    Admin,
    VerifierPubKey,
    UserKyc(Address), // ExpiresAt (u64)
    Registry,
}

#[contract]
pub struct KycVerifier;

#[contractimpl]
impl KycVerifier {
    pub fn initialize(env: Env, admin: Address, verifier_pubkey: BytesN<32>) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VerifierPubKey, &verifier_pubkey);
    }

    pub fn set_registry(env: Env, registry: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        admin.require_auth();
        env.storage().instance().set(&DataKey::Registry, &registry);
    }

    pub fn set_kyc_status(env: Env, user: Address, signature: BytesN<64>, expires_at: u64) {
        Self::ensure_not_paused(&env);
        // KYC provider signs: user_addr + expires_at
        let current_time = env.ledger().timestamp();
        assert!(expires_at > current_time, "proof expired");

        let mut data = Bytes::new(&env);
        data.append(&user.clone().to_xdr(&env));
        data.append(&Bytes::from_slice(&env, &(expires_at as u128).to_be_bytes())); // Example message data

        // In real cases, we'd hash the data or use a specific format.
        // For simplicity, let's verify ed25519 signature.
        let pubkey: BytesN<32> = env.storage().instance().get(&DataKey::VerifierPubKey).unwrap();
        
        env.crypto().ed25519_verify(&pubkey, &data, &signature);

        env.storage().persistent().set(&DataKey::UserKyc(user.clone()), &expires_at);
        env.events().publish((symbol_short!("kyc_set"), user), expires_at);
    }

    pub fn is_kyc_valid(env: Env, user: Address) -> bool {
        let current_time = env.ledger().timestamp();
        match env.storage().persistent().get::<_, u64>(&DataKey::UserKyc(user)) {
            Some(expiry) => expiry > current_time,
            None => false,
        }
    }

    pub fn update_verifier(env: Env, new_pubkey: BytesN<32>) {
        Self::ensure_not_paused(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::VerifierPubKey, &new_pubkey);
    }

    fn ensure_not_paused(env: &Env) {
        if let Some(registry_addr) = env.storage().instance().get::<_, Address>(&DataKey::Registry) {
            let is_paused: bool = env.invoke_contract(&registry_addr, &soroban_sdk::symbol_short!("is_paused"), ().into_val(env));
            if is_paused {
                panic!("system is paused");
            }
        }
    }
}
