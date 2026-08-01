#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token, Address, Env, IntoVal, String,
};

fn create_vault<'a>(env: &Env) -> (Address, Address, PlanVaultContractClient<'a>) {
    env.mock_all_auths();

    let owner = Address::generate(env);
    let sac = env.register_stellar_asset_contract_v2(owner.clone());
    let token_addr = sac.address();

    let contract_id = env.register(PlanVaultContract, (owner.clone(), token_addr.clone()));
    let client = PlanVaultContractClient::new(env, &contract_id);

    (owner, token_addr, client)
}

fn str(s: &Env, t: &str) -> String {
    String::from_str(s, t)
}

#[test]
fn init_and_create_bill_plan_locks_funds() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let contract_addr = client.address.clone();

    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &1_000_000);

    let id = client.create_plan(
        &str(&env, "House Rent"),
        &900_000,
        &PlanType::Bill,
        &(env.ledger().timestamp() + 86_400),
    );
    assert_eq!(id, 1);

    let plan = client.get_plan(&1);
    assert_eq!(plan.amount, 900_000);
    assert_eq!(plan.status, PlanStatus::Locked);
    assert_eq!(plan.plan_type, PlanType::Bill);
    assert_eq!(plan.name, str(&env, "House Rent"));

    // Funds are inside the contract, off the owner's balance.
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&contract_addr), 900_000);
    assert_eq!(token_client.balance(&owner), 100_000);
    assert_eq!(client.get_protected_total(), 900_000);
}

#[test]
fn release_before_unlock_date_fails() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &500_000);
    let id = client.create_plan(&str(&env, "Rent"), &500_000, &PlanType::Bill, &(env.ledger().timestamp() + 1000));
    let res = client.try_release_plan(&id);
    assert_eq!(res.err(), Some(Ok(ContractError::NotYetUnlocked)));
}

#[test]
fn release_after_unlock_date_sends_funds_back() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &1_000_000);

    let unlock = env.ledger().timestamp() + 1000;
    let id = client.create_plan(&str(&env, "Rent"), &500_000, &PlanType::Bill, &unlock);

    env.ledger().with_mut(|l| l.timestamp = unlock + 1);

    client.release_plan(&id);

    let plan = client.get_plan(&id);
    assert_eq!(plan.status, PlanStatus::Released);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&client.address), 0);
    assert_eq!(token_client.balance(&owner), 1_000_000);
    assert_eq!(client.get_protected_total(), 0);
}

#[test]
fn release_before_time_even_without_auth_rejected() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &100_000);

    let id = client.create_plan(&str(&env, "Rent"), &50_000, &PlanType::Bill, &(env.ledger().timestamp() + 1000));

    let stranger = Address::generate(&env);
    let res = client
        .mock_auths(&[MockAuth {
            address: &stranger,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "release_plan",
                args: (id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_release_plan(&id);
    // Even a stranger call fails because funds can only ever go to the owner,
    // and the time hasn't been reached.
    assert_eq!(res.err(), Some(Ok(ContractError::NotYetUnlocked)));
}

#[test]
fn emergency_plan_cannot_auto_release_but_can_early_withdraw() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &300_000);

    let id = client.create_plan(&str(&env, "Emergency"), &300_000, &PlanType::Emergency, &0);
    let plan = client.get_plan(&id);
    assert_eq!(plan.plan_type, PlanType::Emergency);
    assert_eq!(plan.unlock_date, 0);

    // Time passes — still not auto-releasable.
    env.ledger().with_mut(|l| l.timestamp += 99_999_999);
    let res = client.try_release_plan(&id);
    assert_eq!(res.err(), Some(Ok(ContractError::PlanNotBill)));

    // Two-step early withdrawal works.
    client.request_early_withdraw(&id);
    client.confirm_early_withdraw(&id);

    let plan = client.get_plan(&id);
    assert_eq!(plan.status, PlanStatus::EarlyWithdrawn);
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&client.address), 0);
    assert_eq!(token_client.balance(&owner), 300_000);
}

#[test]
fn confirm_without_request_fails() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &100_000);

    let id = client.create_plan(&str(&env, "Rent"), &50_000, &PlanType::Bill, &(env.ledger().timestamp() + 86_400));
    let res = client.try_confirm_early_withdraw(&id);
    assert_eq!(res.err(), Some(Ok(ContractError::EarlyWithdrawNotRequested)));
}

#[test]
fn double_release_is_rejected() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &100_000);

    let unlock_at = env.ledger().timestamp() + 1000;
    let id = client.create_plan(&str(&env, "Rent"), &40_000, &PlanType::Bill, &unlock_at);
    env.ledger().with_mut(|l| l.timestamp = unlock_at + 1);

    client.release_plan(&id);
    let res = client.try_release_plan(&id);
    assert_eq!(res.err(), Some(Ok(ContractError::PlanNotLocked)));
}

#[test]
fn init_can_only_be_called_once() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    // Re-invoke the constructor symbol on the already-deployed contract.
    let err = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &client.address,
        &soroban_sdk::Symbol::new(&env, "__constructor"),
        (owner, token).into_val(&env),
    );
    assert!(err.is_err());
}

#[test]
fn creating_a_bill_plan_with_zero_or_past_unlock_date_fails() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &100_000);

    let r1 = client.try_create_plan(&str(&env, "Rent"), &10_000, &PlanType::Bill, &0);
    assert_eq!(r1.err(), Some(Ok(ContractError::InvalidUnlockDate)));

    // Ensure ledger timestamp is non-zero before subtracting.
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let r2 = client.try_create_plan(
        &str(&env, "Rent"),
        &10_000,
        &PlanType::Bill,
        &(env.ledger().timestamp() - 1),
    );
    assert_eq!(r2.err(), Some(Ok(ContractError::UnlockDateInPast)));

    let r3 = client.try_create_plan(&str(&env, "Rent"), &0, &PlanType::Bill, &(env.ledger().timestamp() + 100));
    assert_eq!(r3.err(), Some(Ok(ContractError::InvalidAmount)));
}

#[test]
fn list_plans_and_protected_total() {
    let env = Env::default();
    let (owner, token, client) = create_vault(&env);
    let sac = token::StellarAssetClient::new(&env, &token);
    sac.mint(&owner, &1_000_000);

    client.create_plan(&str(&env, "Rent"), &900_000, &PlanType::Bill, &(env.ledger().timestamp() + 86_400));
    client.create_plan(&str(&env, "Emergency"), &50_000, &PlanType::Emergency, &0);

    let plans = client.list_plans();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans.get(0).unwrap().name, str(&env, "Rent"));
    assert_eq!(client.get_protected_total(), 950_000);
}
