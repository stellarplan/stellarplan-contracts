#![cfg(test)]
//! Tests for the v1.1.0 hardening work: input validation, authorization
//! failures, the configurable early-withdraw delay, request cancellation,
//! overflow safety, and storage keep-alive.

use super::*;
use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger, MockAuth, MockAuthInvoke,
    },
    token, Address, Env, IntoVal, String,
};

fn create_vault<'a>(env: &Env, mint: i128) -> (Address, Address, PlanVaultContractClient<'a>) {
    env.mock_all_auths();

    let owner = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(owner.clone());
    let token_addr = sac.address();
    let contract_id = env.register(PlanVaultContract, (owner.clone(), token_addr.clone()));
    let client = PlanVaultContractClient::new(env, &contract_id);

    token::StellarAssetClient::new(env, &token_addr).mint(&owner, &mint);
    (owner, token_addr, client)
}

fn s(env: &Env, text: &str) -> String {
    String::from_str(env, text)
}

#[test]
fn version_reports_current_release() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 0);
    assert_eq!(client.version(), symbol_short!("v1_1_0"));
}

#[test]
fn empty_and_oversized_plan_names_are_rejected() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 1_000_000);
    let unlock = env.ledger().timestamp() + 1_000;

    let empty = client.try_create_plan(&s(&env, ""), &1_000, &PlanType::Bill, &unlock);
    assert_eq!(empty.err(), Some(Ok(ContractError::InvalidName)));

    let too_long = "x".repeat(65);
    let r = client.try_create_plan(&s(&env, &too_long), &1_000, &PlanType::Bill, &unlock);
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidName)));

    // Exactly the limit is accepted.
    let at_limit = "x".repeat(64);
    let id = client.create_plan(&s(&env, &at_limit), &1_000, &PlanType::Bill, &unlock);
    assert_eq!(id, 1);
}

#[test]
fn only_the_owner_can_create_a_plan() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 1_000_000);
    let stranger = Address::generate(&env);
    let unlock = env.ledger().timestamp() + 1_000;

    let res = client
        .mock_auths(&[MockAuth {
            address: &stranger,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "create_plan",
                args: (s(&env, "Rent"), 1_000i128, PlanType::Bill, unlock).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_create_plan(&s(&env, "Rent"), &1_000, &PlanType::Bill, &unlock);
    assert!(res.is_err(), "a non-owner signature must not create a plan");
    assert_eq!(client.list_plans().len(), 0);
}

#[test]
fn creating_a_plan_without_enough_balance_fails_and_stores_nothing() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 500);
    let res = client.try_create_plan(
        &s(&env, "Rent"),
        &1_000,
        &PlanType::Bill,
        &(env.ledger().timestamp() + 1_000),
    );
    assert!(res.is_err());
    assert_eq!(client.list_plans().len(), 0);
    assert_eq!(client.get_protected_total(), 0);
}

#[test]
fn savings_plans_are_stored_without_a_date_and_use_the_two_step_flow() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env, 100_000);
    let unlock = env.ledger().timestamp() + 500;
    let id = client.create_plan(&s(&env, "Holiday"), &60_000, &PlanType::Savings, &unlock);

    // Only Bill plans keep a date on-chain, so Savings can not auto-release.
    assert_eq!(client.get_plan(&id).unlock_date, 0);
    env.ledger().with_mut(|l| l.timestamp = unlock + 1);
    let res = client.try_release_plan(&id);
    assert_eq!(res.err(), Some(Ok(ContractError::PlanNotBill)));

    client.request_early_withdraw(&id);
    client.confirm_early_withdraw(&id);
    assert_eq!(token::Client::new(&env, &token).balance(&owner), 100_000);
}

#[test]
fn double_confirm_is_rejected() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    let id = client.create_plan(&s(&env, "Emergency"), &50_000, &PlanType::Emergency, &0);
    client.request_early_withdraw(&id);
    client.confirm_early_withdraw(&id);
    let res = client.try_confirm_early_withdraw(&id);
    assert_eq!(res.err(), Some(Ok(ContractError::PlanNotLocked)));
}

#[test]
fn early_withdraw_delay_defaults_to_zero_and_can_be_raised() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 1);
    assert_eq!(client.get_early_withdraw_delay(), 0);

    client.set_early_withdraw_delay(&3_600);
    assert_eq!(client.get_early_withdraw_delay(), 3_600);
}

#[test]
fn early_withdraw_waits_for_the_configured_delay() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env, 100_000);
    client.set_early_withdraw_delay(&3_600);
    let id = client.create_plan(&s(&env, "Emergency"), &50_000, &PlanType::Emergency, &0);

    client.request_early_withdraw(&id);

    // Immediately and one second early: refused.
    let r = client.try_confirm_early_withdraw(&id);
    assert_eq!(r.err(), Some(Ok(ContractError::EarlyWithdrawDelayNotMet)));
    env.ledger().with_mut(|l| l.timestamp += 3_599);
    let r = client.try_confirm_early_withdraw(&id);
    assert_eq!(r.err(), Some(Ok(ContractError::EarlyWithdrawDelayNotMet)));
    assert_eq!(client.get_plan(&id).status, PlanStatus::Locked);

    // Exactly at the boundary: accepted.
    env.ledger().with_mut(|l| l.timestamp += 1);
    client.confirm_early_withdraw(&id);
    assert_eq!(client.get_plan(&id).status, PlanStatus::EarlyWithdrawn);
    assert_eq!(token::Client::new(&env, &token).balance(&owner), 100_000);
}

#[test]
fn delay_above_the_cap_is_rejected() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 1);
    let cap = 30 * 24 * 60 * 60u64;
    client.set_early_withdraw_delay(&cap);
    let r = client.try_set_early_withdraw_delay(&(cap + 1));
    assert_eq!(r.err(), Some(Ok(ContractError::InvalidDelay)));
    assert_eq!(client.get_early_withdraw_delay(), cap);
}

#[test]
fn only_the_owner_can_change_the_delay() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 1);
    let stranger = Address::generate(&env);
    let res = client
        .mock_auths(&[MockAuth {
            address: &stranger,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_early_withdraw_delay",
                args: (10u64,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_set_early_withdraw_delay(&10);
    assert!(res.is_err());
    assert_eq!(client.get_early_withdraw_delay(), 0);
}

#[test]
fn delay_arithmetic_cannot_overflow() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    client.set_early_withdraw_delay(&100);
    let id = client.create_plan(&s(&env, "Emergency"), &10_000, &PlanType::Emergency, &0);

    env.ledger().with_mut(|l| l.timestamp = u64::MAX - 10);
    client.request_early_withdraw(&id);
    let r = client.try_confirm_early_withdraw(&id);
    assert_eq!(r.err(), Some(Ok(ContractError::Overflow)));
    assert_eq!(client.get_plan(&id).status, PlanStatus::Locked);
}

#[test]
fn cancelling_a_request_keeps_the_plan_locked_and_needs_a_fresh_request() {
    let env = Env::default();
    let (_owner, token, client) = create_vault(&env, 100_000);
    let id = client.create_plan(&s(&env, "Emergency"), &40_000, &PlanType::Emergency, &0);

    client.request_early_withdraw(&id);
    client.cancel_early_withdraw(&id);

    let r = client.try_confirm_early_withdraw(&id);
    assert_eq!(r.err(), Some(Ok(ContractError::EarlyWithdrawNotRequested)));
    assert_eq!(client.get_plan(&id).status, PlanStatus::Locked);
    assert_eq!(
        token::Client::new(&env, &token).balance(&client.address),
        40_000
    );

    // Cancelling again, or an unknown plan, is an error.
    let r = client.try_cancel_early_withdraw(&id);
    assert_eq!(r.err(), Some(Ok(ContractError::EarlyWithdrawNotRequested)));
    let r = client.try_cancel_early_withdraw(&99);
    assert_eq!(r.err(), Some(Ok(ContractError::PlanNotFound)));
}

#[test]
fn only_the_owner_can_cancel_a_request() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    let id = client.create_plan(&s(&env, "Emergency"), &40_000, &PlanType::Emergency, &0);
    client.request_early_withdraw(&id);

    let stranger = Address::generate(&env);
    let res = client
        .mock_auths(&[MockAuth {
            address: &stranger,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "cancel_early_withdraw",
                args: (id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_cancel_early_withdraw(&id);
    assert!(res.is_err());

    // The original request is intact and can still be confirmed.
    client.confirm_early_withdraw(&id);
}

#[test]
fn a_released_plan_no_longer_counts_as_protected() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    let unlock = env.ledger().timestamp() + 100;
    let bill = client.create_plan(&s(&env, "Rent"), &30_000, &PlanType::Bill, &unlock);
    client.create_plan(&s(&env, "Emergency"), &20_000, &PlanType::Emergency, &0);
    assert_eq!(client.get_protected_total(), 50_000);

    env.ledger().with_mut(|l| l.timestamp = unlock);
    client.release_plan(&bill);
    assert_eq!(client.get_protected_total(), 20_000);
}

#[test]
fn bump_ttl_is_open_to_anyone_and_changes_no_data() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    let id = client.create_plan(&s(&env, "Emergency"), &20_000, &PlanType::Emergency, &0);
    let before = client.get_plan(&id);

    // No mocked authorization at all: a keeper job needs no signature.
    let res = client.mock_auths(&[]).try_bump_ttl();
    assert!(res.is_ok());
    assert_eq!(client.get_plan(&id), before);
    assert_eq!(client.get_protected_total(), 20_000);
}

#[test]
fn bump_ttl_extends_the_instance_and_every_plan() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    let id = client.create_plan(&s(&env, "Emergency"), &20_000, &PlanType::Emergency, &0);

    let ttl_of_plan = || {
        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get_ttl(&types::DataKey::Plan(id))
        })
    };
    let ttl_of_instance =
        || env.as_contract(&client.address, || env.storage().instance().get_ttl());

    // Writes already extend both entries to the full lifetime.
    assert_eq!(ttl_of_plan(), types::INSTANCE_BUMP_AMOUNT);
    assert_eq!(ttl_of_instance(), types::INSTANCE_BUMP_AMOUNT);

    // 20 days pass: both have aged.
    let twenty_days = 20 * types::DAY_IN_LEDGERS;
    env.ledger().with_mut(|l| l.sequence_number += twenty_days);
    assert_eq!(ttl_of_plan(), types::INSTANCE_BUMP_AMOUNT - twenty_days);
    assert_eq!(ttl_of_instance(), types::INSTANCE_BUMP_AMOUNT - twenty_days);

    // A keeper bump, with no signature, restores the full lifetime.
    client.mock_auths(&[]).bump_ttl();
    assert_eq!(ttl_of_plan(), types::INSTANCE_BUMP_AMOUNT);
    assert_eq!(ttl_of_instance(), types::INSTANCE_BUMP_AMOUNT);
}

#[test]
fn a_pending_early_withdraw_request_is_kept_alive_when_recorded() {
    let env = Env::default();
    let (_owner, _token, client) = create_vault(&env, 100_000);
    let id = client.create_plan(&s(&env, "Emergency"), &20_000, &PlanType::Emergency, &0);
    client.request_early_withdraw(&id);

    let ttl = env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .get_ttl(&types::DataKey::EarlyWithdrawRequest(id))
    });
    assert_eq!(ttl, types::INSTANCE_BUMP_AMOUNT);
}
