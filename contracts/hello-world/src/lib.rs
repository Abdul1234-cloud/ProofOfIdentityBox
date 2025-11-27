#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Endpoint {
    pub id: u64,
    pub owner: Address,
    pub url_hash: String, // client stores full URL offchain, uses hash onchain
}

#[contracttype]
#[derive(Clone)]
pub struct Check {
    pub endpoint_id: u64,
    pub status: bool,
    pub timestamp: u64,
    pub oracle: Address,
}

#[contracttype]
pub enum EndpointKey {
    Count,
    Endpoint(u64),
}

#[contracttype]
pub enum CheckKey {
    Checks(u64), // endpoint_id -> Vec<Check>
}

#[contract]
pub struct WebPulseMonitor;

#[contractimpl]
impl WebPulseMonitor {
    // register an endpoint to be monitored; owner records url_hash
    pub fn register_endpoint(env: Env, owner: Address, url_hash: String) -> u64 {
        owner.require_auth();
        
        let mut count: u64 = env.storage().instance().get(&EndpointKey::Count).unwrap_or(0);
        count = count.saturating_add(1);
        env.storage().instance().set(&EndpointKey::Count, &count);

        let ep = Endpoint { id: count, owner: owner.clone(), url_hash };
        env.storage().instance().set(&EndpointKey::Endpoint(count), &ep);
        env.storage().instance().set(&CheckKey::Checks(count), &Vec::<Check>::new(&env));
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        count
    }

    // oracle submits a check result for an endpoint
    pub fn submit_check(env: Env, oracle: Address, endpoint_id: u64, status: bool) {
        oracle.require_auth();
        
        let _ep: Endpoint = env.storage().instance().get(&EndpointKey::Endpoint(endpoint_id)).expect("endpoint not found");
        let mut checks: Vec<Check> = env.storage().instance().get(&CheckKey::Checks(endpoint_id)).unwrap_or(Vec::new(&env));
        
        let rec = Check {
            endpoint_id,
            status,
            timestamp: env.ledger().timestamp(),
            oracle: oracle.clone(),
        };
        checks.push_back(rec);
        env.storage().instance().set(&CheckKey::Checks(endpoint_id), &checks);
        
        env.storage().instance().extend_ttl(5000, 5000);
    }

    // view latest check for an endpoint
    pub fn view_latest(env: Env, endpoint_id: u64) -> Check {
        let checks: Vec<Check> = env.storage().instance().get(&CheckKey::Checks(endpoint_id)).expect("no checks");
        
        if checks.len() == 0 {
            panic!("no checks available");
        }
        
        let last_idx = checks.len().saturating_sub(1);
        checks.get(last_idx).expect("no checks available")
    }

    // Get endpoint details
    pub fn get_endpoint(env: Env, endpoint_id: u64) -> Endpoint {
        env.storage().instance().get(&EndpointKey::Endpoint(endpoint_id)).expect("endpoint not found")
    }

    // Get total endpoint count
    pub fn get_endpoint_count(env: Env) -> u64 {
        env.storage().instance().get(&EndpointKey::Count).unwrap_or(0)
    }

    // Get check count for an endpoint
    pub fn get_check_count(env: Env, endpoint_id: u64) -> u32 {
        let checks: Vec<Check> = env.storage().instance().get(&CheckKey::Checks(endpoint_id)).unwrap_or(Vec::new(&env));
        checks.len()
    }

    // View a specific check by index (0-based)
    pub fn view_check(env: Env, endpoint_id: u64, check_index: u32) -> Check {
        let checks: Vec<Check> = env.storage().instance().get(&CheckKey::Checks(endpoint_id)).expect("no checks");
        checks.get(check_index).expect("check not found")
    }
}