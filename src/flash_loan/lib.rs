#![no_std]

use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, symbol_short, token, Address, Env, IntoVal,
};

/// Interface that a flash loan receiver must implement.
#[contractclient(name = "FlashLoanReceiverClient")]
pub trait FlashLoanReceiver {
    fn execute_loan(env: Env, token: Address, amount: i128, fee: i128);
}

#[contracttype]
pub enum DataKey {
    Admin,
    Registry,
}

#[contract]
pub struct FlashLoanProvider;

#[contractimpl]
impl FlashLoanProvider {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn set_registry(env: Env, registry: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("not initialized");
        admin.require_auth();
        env.storage().instance().set(&DataKey::Registry, &registry);
    }

    /// Executes a flash loan.
    /// 
    /// # Arguments
    /// * `receiver` - The address of the contract that will receive the loan and execute the logic.
    /// * `token` - The address of the token to be lent.
    /// * `amount` - The amount of tokens to lend.
    pub fn flash_loan(env: Env, receiver: Address, token: Address, amount: i128) {
        Self::ensure_not_paused(&env);
        // 1. Calculate the fee (5 basis points = 0.05%)
        // fee = amount * 5 / 10000
        let fee = amount * 5 / 10000;
        
        // 2. Initial balance check
        let token_client = token::Client::new(&env, &token);
        let balance_before = token_client.balance(&env.current_contract_address());
        
        // 3. Transfer tokens to the receiver
        token_client.transfer(&env.current_contract_address(), &receiver, &amount);
        
        // 4. Invoke the receiver's execution logic
        let receiver_client = FlashLoanReceiverClient::new(&env, &receiver);
        receiver_client.execute_loan(&token, &amount, &fee);
        
        // 5. Verify repayment
        let balance_after = token_client.balance(&env.current_contract_address());
        
        if balance_after < balance_before + fee {
            panic!("Flash loan not repaid with fee");
        }

        // 6. Emit event
        env.events().publish(
            (symbol_short!("flash_ln"), receiver, token),
            (amount, fee),
        );
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

mod tests;
