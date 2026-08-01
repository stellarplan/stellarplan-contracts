#![no_std]
//! StellarPlan — Plan Vault contract.
//!
//! One contract instance is deployed per connected wallet. It holds
//! time-locked savings "plans" (internally: vaults) on behalf of a single
//! owner, in a single Stellar asset (e.g. USDC).
//!
//! Responsibilities (intentionally small, per the PRD):
//!   • create plans
//!   • lock funds behind a release date
//!   • release funds once the date is reached
//!   • allow early withdrawal with a friction delay
//! Business logic (allocation, salary detection, dashboards) lives in the API.

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use errors::ContractError;
pub use types::{Plan, PlanStatus, PlanType};

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol, Vec};
use storage::{
    get_owner, get_plan, get_plan_count, get_token, is_initialized, put_plan, set_initialized,
    set_owner, set_plan_count, set_token,
};
use types::{DataKey, INSTANCE_BUMP_AMOUNT};

#[contract]
pub struct PlanVaultContract;

#[contractimpl]
impl PlanVaultContract {
    /// Constructor — called once at deployment (`env.register(Contract, (owner, token))`)
    /// Registers the vault owner + asset and marks it initialized.
    pub fn __constructor(env: Env, owner: Address, token: Address) -> Result<(), ContractError> {
        if is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        owner.require_auth();

        set_owner(&env, &owner);
        set_token(&env, &token);
        set_initialized(&env);
        set_plan_count(&env, 0u32);

        env.storage().instance().extend_ttl(INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_AMOUNT);
        events::vault_initialized(&env, &owner, &token);
        Ok(())
    }

    /// Create a new time-locked plan and fund it.
    ///
    /// `amount`        — how many stroops of the vault's asset to lock.
    /// `unlock_date`   — unix timestamp; 0 means "always locked" until manually broken (Emergency/Savings).
    /// `plan_type`     — Bill | Emergency | Savings.
    ///
    /// Funds are transferred from the caller into the contract immediately.
    /// Returns the new plan id.
    pub fn create_plan(
        env: Env,
        name: String,
        amount: i128,
        plan_type: PlanType,
        unlock_date: u64,
    ) -> Result<u32, ContractError> {
        let owner = get_owner(&env)?;
        owner.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }
        if plan_type == PlanType::Bill && unlock_date == 0 {
            return Err(ContractError::InvalidUnlockDate);
        }
        let now = env.ledger().timestamp();
        if plan_type == PlanType::Bill && unlock_date <= now {
            return Err(ContractError::UnlockDateInPast);
        }

        let count = get_plan_count(&env)?;
        let plan_id = count.checked_add(1).ok_or(ContractError::Overflow)?;

        // Pull the funds into the contract.
        let _token = get_token(&env)?;
        transfer(&env, &owner, &env.current_contract_address(), amount)?;

        let plan = Plan {
            id: plan_id,
            name,
            amount,
            plan_type,
            status: PlanStatus::Locked,
            unlock_date: if plan_type == PlanType::Bill { unlock_date } else { 0 },
            created_at: now,
        };
        put_plan(&env, &plan);
        set_plan_count(&env, plan_id);

        events::plan_created(&env, plan_id, amount, plan.unlock_date);
        Ok(plan_id)
    }

    /// Release a plan whose unlock date has been reached.
    ///
    /// Funds are returned to the owner. Callable by ANYONE — this lets the
    /// backend auto-run it daily without needing the owner's signature, while
    /// still being safe because funds can only ever go to the contract owner.
    pub fn release_plan(env: Env, plan_id: u32) -> Result<(), ContractError> {
        let mut plan = get_plan(&env, plan_id)?;
        let owner = get_owner(&env)?;

        if plan.status != PlanStatus::Locked {
            return Err(ContractError::PlanNotLocked);
        }
        if plan.plan_type != PlanType::Bill {
            return Err(ContractError::PlanNotBill);
        }
        if env.ledger().timestamp() < plan.unlock_date {
            return Err(ContractError::NotYetUnlocked);
        }

        transfer(&env, &env.current_contract_address(), &owner, plan.amount)?;

        plan.status = PlanStatus::Released;
        put_plan(&env, &plan);
        events::plan_released(&env, plan_id, owner, plan.amount);
        Ok(())
    }

    /// Early withdrawal, step 1: record an intent to break the plan.
    ///
    /// Requires the owner's signature. Starts the friction countdown.
    pub fn request_early_withdraw(env: Env, plan_id: u32) -> Result<(), ContractError> {
        let owner = get_owner(&env)?;
        owner.require_auth();

        let plan = get_plan(&env, plan_id)?;
        if plan.status != PlanStatus::Locked {
            return Err(ContractError::PlanNotLocked);
        }

        env.storage().persistent().set(
            &DataKey::EarlyWithdrawRequest(plan_id),
            &env.ledger().timestamp(),
        );
        events::early_withdraw_requested(&env, plan_id);
        Ok(())
    }

    /// Early withdrawal, step 2: complete the withdrawal after the delay.
    ///
    /// Requires the owner's signature again. The delay is only enforced if a
    /// `EARLY_WITHDRAW_DELAY` (seconds) was configured at build time via
    /// `storage::get_early_withdraw_delay`; by default the intent must have been
    /// recorded in a prior ledger.
    pub fn confirm_early_withdraw(env: Env, plan_id: u32) -> Result<(), ContractError> {
        let owner = get_owner(&env)?;
        owner.require_auth();

        let mut plan = get_plan(&env, plan_id)?;
        if plan.status != PlanStatus::Locked {
            return Err(ContractError::PlanNotLocked);
        }

        let key = DataKey::EarlyWithdrawRequest(plan_id);
        let requested_at: u64 = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(ContractError::EarlyWithdrawNotRequested)?;

        let required_delay = storage::get_early_withdraw_delay(&env);
        let now = env.ledger().timestamp();
        if now < requested_at + required_delay {
            return Err(ContractError::EarlyWithdrawDelayNotMet);
        }

        transfer(&env, &env.current_contract_address(), &owner, plan.amount)?;

        plan.status = PlanStatus::EarlyWithdrawn;
        put_plan(&env, &plan);
        env.storage().persistent().remove(&key);
        events::early_withdraw_completed(&env, plan_id, owner, plan.amount);
        Ok(())
    }

    /// Read a single plan.
    pub fn get_plan(env: Env, plan_id: u32) -> Result<Plan, ContractError> {
        get_plan(&env, plan_id)
    }

    /// Read all plans. There will never be enough plans for gas to be an issue.
    pub fn list_plans(env: Env) -> Result<Vec<Plan>, ContractError> {
        let count = get_plan_count(&env)?;
        let mut plans = Vec::new(&env);
        for id in 1..=count {
            if let Ok(plan) = get_plan(&env, id) {
                plans.push_back(plan);
            }
        }
        Ok(plans)
    }

    /// Total amount currently protected across all Locked plans.
    pub fn get_protected_total(env: Env) -> Result<i128, ContractError> {
        let count = get_plan_count(&env)?;
        let mut total: i128 = 0;
        for id in 1..=count {
            if let Ok(plan) = get_plan(&env, id) {
                if plan.status == PlanStatus::Locked {
                    total = total.checked_add(plan.amount).ok_or(ContractError::Overflow)?;
                }
            }
        }
        Ok(total)
    }

    /// Contract version.
    pub fn version(env: Env) -> Symbol {
        let _ = env;
        symbol_short!("v1_0_0")
    }
}

/// Move `amount` of the vault's token from `from` to `to`.
/// `from` must have signed for anything other than the contract itself.
fn transfer(env: &Env, from: &Address, to: &Address, amount: i128) -> Result<(), ContractError> {
    let token_addr = get_token(env)?;
    let client = soroban_sdk::token::Client::new(env, &token_addr);
    client.transfer(from, to, &amount);
    Ok(())
}
