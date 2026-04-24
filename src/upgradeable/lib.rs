#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

/// Storage keys used by the upgradeable contract.
/// Instance storage is used for config (Admin, Version).
/// Persistent storage is used for user data.
#[derive(Clone)]
#[contracttype]
enum DataKey {
    /// The administrator address authorized to perform upgrades. (Instance)
    Admin,
    /// The current contract version number (incremented on each upgrade). (Instance)
    Version,
    /// Example persistent data key that might require migration. (Persistent)
    UserData(Address),
}

/// Old Data Schema (V1)
#[contracttype]
pub struct UserDataV1 {
    pub balance: u128,
}

/// New Data Schema (V2)
#[contracttype]
pub struct UserDataV2 {
    pub balance: u128,
    pub reputation: u32,
}

#[contract]
pub struct UpgradeableContract;

#[contractimpl]
impl UpgradeableContract {
    /// Initializes the contract with the given admin address.
    /// Admin should preferably be a Multisig or Governance contract.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &1u32);
    }

    /// Upgrades the contract to a new WASM binary identified by `new_wasm_hash`.
    /// Admin/Governance gate ensures only authorized multisigs/daos can upgrade.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let current_version: u32 = env.storage().instance().get(&DataKey::Version).unwrap_or(1);
        env.storage().instance().set(&DataKey::Version, &(current_version + 1));

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Migrates persistent data from V1 to V2 schema.
    /// This should be called immediately after an upgrade.
    pub fn migrate_user_data(env: Env, user: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let version: u32 = env.storage().instance().get(&DataKey::Version).unwrap_or(1);
        if version >= 2 {
            // Check if V1 data exists and migrate it
            if let Some(old_data) = env.storage().persistent().get::<_, UserDataV1>(&DataKey::UserData(user.clone())) {
                let new_data = UserDataV2 {
                    balance: old_data.balance,
                    reputation: 0, // default new field
                };
                env.storage().persistent().set(&DataKey::UserData(user), &new_data);
            }
        }
    }

    /// Transfers the admin role to a new address (e.g., from an EOA to a Governance contract).
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    pub fn version(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Version).unwrap_or(0)
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    fn setup_contract() -> (Env, UpgradeableContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(UpgradeableContract, ());
        let client = UpgradeableContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        (env, client, admin)
    }

    #[test]
    fn test_initialize() {
        let (_env, client, admin) = setup_contract();
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.version(), 1);
    }

    #[test]
    #[should_panic(expected = "contract already initialized")]
    fn test_initialize_twice_panics() {
        let (env, client, _admin) = setup_contract();
        let another_admin = Address::generate(&env);
        client.initialize(&another_admin);
    }

    #[test]
    fn test_set_admin() {
        let (env, client, _admin) = setup_contract();
        let new_admin = Address::generate(&env);
        client.set_admin(&new_admin);
        assert_eq!(client.get_admin(), new_admin);
    }

    #[test]
    fn test_version() {
        let (_env, client, _admin) = setup_contract();
        assert_eq!(client.version(), 1);
    }
}
