#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, Symbol, Val, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Call {
    pub contract: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Registry,
}

#[contract]
pub struct BatchExecutor;

#[contractimpl]
impl BatchExecutor {
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

    /// Executes a sequence of contract calls in a single transaction.
    /// Returns a list of the execution results.
    /// If any call fails, the entire transaction reverts.
    pub fn execute_batch(env: Env, calls: Vec<Call>) -> Vec<Val> {
        Self::ensure_not_paused(&env);
        let mut results = Vec::new(&env);
        for call in calls.iter() {
            let result: Val = env.invoke_contract(&call.contract, &call.function, call.args.clone());
            results.push_back(result);
        }
        results
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

mod test;
