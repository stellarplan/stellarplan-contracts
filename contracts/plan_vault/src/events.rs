use soroban_sdk::{symbol_short, Address, Env, IntoVal, Val, Vec};

#[allow(deprecated)]
fn publish<T: IntoVal<Env, Val>, const N: usize>(
    env: &Env,
    topics_array: [T; N],
    data: impl IntoVal<Env, Val>,
) {
    let mut topics: Vec<Val> = Vec::new(env);
    for t in topics_array.into_iter() {
        topics.push_back(t.into_val(env));
    }
    env.events().publish(topics, data);
}

/// vault initialized
pub fn vault_initialized(env: &Env, owner: &Address, token: &Address) {
    publish(
        env,
        [symbol_short!("vault"), symbol_short!("init")],
        (owner.clone(), token.clone()),
    );
}

/// plan created
pub fn plan_created(env: &Env, plan_id: u32, amount: i128, unlock_date: u64) {
    publish(
        env,
        [symbol_short!("vault"), symbol_short!("create")],
        (plan_id, amount, unlock_date),
    );
}

/// plan released
pub fn plan_released(env: &Env, plan_id: u32, to: Address, amount: i128) {
    publish(
        env,
        [symbol_short!("vault"), symbol_short!("release")],
        (plan_id, to, amount),
    );
}

/// early withdraw requested
pub fn early_withdraw_requested(env: &Env, plan_id: u32) {
    publish(
        env,
        [symbol_short!("vault"), symbol_short!("ew_req")],
        plan_id,
    );
}

/// early withdraw completed
pub fn early_withdraw_completed(env: &Env, plan_id: u32, to: Address, amount: i128) {
    publish(
        env,
        [symbol_short!("vault"), symbol_short!("ew_done")],
        (plan_id, to, amount),
    );
}
