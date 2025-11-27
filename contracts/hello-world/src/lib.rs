#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Env, Symbol, String, symbol_short};

const ID_REGISTRY: Symbol = symbol_short!("IDREG");

#[contracttype]
#[derive(Clone)]
pub struct IdentityRecord {
    pub subject: Symbol,      // wallet / account being identified
    pub issuer: Symbol,       // trusted identity issuer
    pub country: String,      // example attribute
    pub kyc_level: u32,       // simple integer level for KYC tiering
    pub issued_at: u64,
    pub valid: bool,
}

#[contract]
pub struct ProofOfIdentityBox;

#[contractimpl]
impl ProofOfIdentityBox {
    // Issue or update an identity attestation for a subject (called by a trusted issuer/admin)
    pub fn issue_identity(
        env: Env,
        subject: Symbol,
        issuer: Symbol,
        country: String,
        kyc_level: u32,
    ) {
        let issued_at = env.ledger().timestamp();
        let record = IdentityRecord {
            subject: subject.clone(),
            issuer,
            country,
            kyc_level,
            issued_at,
            valid: true,
        };
        // Keyed by subject symbol; one active record per subject
        env.storage().instance().set(&subject, &record);
    }

    // Revoke an identity attestation (e.g., KYC expired or withdrawn)
    pub fn revoke_identity(env: Env, subject: Symbol) {
        let mut record: IdentityRecord =
            env.storage().instance().get(&subject).unwrap();
        record.valid = false;
        env.storage().instance().set(&subject, &record);
    }

    // Check if a subject has a valid identity attestation with at least a given KYC level
    pub fn has_valid_identity(env: Env, subject: Symbol, min_kyc_level: u32) -> bool {
        let record: Option<IdentityRecord> =
            env.storage().instance().get(&subject);

        match record {
            Some(r) => r.valid && r.kyc_level >= min_kyc_level,
            None => false,
        }
    }

    // Get full identity record for a subject
    pub fn get_identity(env: Env, subject: Symbol) -> Option<IdentityRecord> {
        env.storage().instance().get(&subject)
    }
}
