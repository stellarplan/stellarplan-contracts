use soroban_sdk::{Address, Env};

use crate::{
    errors::ContractError,
    types::{DataKey, Plan, INSTANCE_BUMP_AMOUNT},
};

/// Early-withdraw cooling-off period in seconds.
///
/// Defaults to 0 (the intent only has to be recorded before it is confirmed) so
/// existing vaults and testnet demos behave as before. The owner can raise it
/// with `set_early_withdraw_delay`.
pub fn get_early_withdraw_delay(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::EarlyWithdrawDelay)
        .unwrap_or(0)
}

pub fn set_early_withdraw_delay(env: &Env, seconds: u64) {
    env.storage()
        .instance()
        .set(&DataKey::EarlyWithdrawDelay, &seconds);
}

/// Extend the instance entry (owner, token, counters, settings) so a vault that
/// is only used occasionally does not expire between interactions.
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_AMOUNT);
}

/// Record when the owner asked to break a plan, and keep that entry alive.
pub fn put_early_withdraw_request(env: &Env, plan_id: u32, requested_at: u64) {
    let key = DataKey::EarlyWithdrawRequest(plan_id);
    env.storage().persistent().set(&key, &requested_at);
    env.storage()
        .persistent()
        .extend_ttl(&key, INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_AMOUNT);
}

pub fn get_early_withdraw_request(env: &Env, plan_id: u32) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::EarlyWithdrawRequest(plan_id))
}

pub fn clear_early_withdraw_request(env: &Env, plan_id: u32) {
    env.storage()
        .persistent()
        .remove(&DataKey::EarlyWithdrawRequest(plan_id));
}

/// Extend one plan's persistent entry. Returns false if the plan is missing.
pub fn bump_plan(env: &Env, plan_id: u32) -> bool {
    let key = DataKey::Plan(plan_id);
    if !env.storage().persistent().has(&key) {
        return false;
    }
    env.storage()
        .persistent()
        .extend_ttl(&key, INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_AMOUNT);
    true
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
    env.storage()
        .persistent()
        .set(&DataKey::Plan(plan.id), plan);
    // Keep the entry alive for as long as the effort holds together.
    env.storage().persistent().extend_ttl(
        &DataKey::Plan(plan.id),
        INSTANCE_BUMP_AMOUNT,
        INSTANCE_BUMP_AMOUNT,
    );
}
