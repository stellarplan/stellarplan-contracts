use soroban_sdk::{Address, Env};

use crate::{
    errors::ContractError,
    types::{DataKey, Plan},
};

/// Early-withdraw friction delay, in seconds.
/// 0 means "intent must simply be recorded in a previous ledger" — we keep it
/// at 0 for MVP so testnet demos aren't blocked waiting real time, while the
/// two-step flow still forces the user to pause and confirm twice.
pub fn get_early_withdraw_delay(_env: &Env) -> u64 {
    0
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&DataKey::Initialized, &true);
}

pub fn get_owner(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Owner)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_owner(env: &Env, owner: &Address) {
    env.storage().instance().set(&DataKey::Owner, owner);
}

pub fn get_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Token)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

pub fn get_plan_count(env: &Env) -> Result<u32, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::PlanCount)
        .ok_or(ContractError::NotInitialized)
}

pub fn set_plan_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::PlanCount, &count);
}

pub fn get_plan(env: &Env, plan_id: u32) -> Result<Plan, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Plan(plan_id))
        .ok_or(ContractError::PlanNotFound)
}

pub fn put_plan(env: &Env, plan: &Plan) {
    env.storage().persistent().set(&DataKey::Plan(plan.id), plan);
    // Keep the entry alive for as long as the effort holds together.
    env.storage().persistent().extend_ttl(
        &DataKey::Plan(plan.id),
        crate::types::INSTANCE_BUMP_AMOUNT,
        crate::types::INSTANCE_BUMP_AMOUNT,
    );
}
