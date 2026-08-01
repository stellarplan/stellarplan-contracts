use soroban_sdk::{contracttype, Address, String};

/// Ledger time is ~5s. One day ≈ 17280 ledgers.
pub const DAY_IN_LEDGERS: u32 = 17_280;
/// Bump contract storage on every interaction so data never expires.
/// ~30 days.
pub const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;

/// Instance storage keys.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Owner,
    Token,
    Initialized,
    PlanCount,
    Plan(u32),
    EarlyWithdrawRequest(u32),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PlanStatus {
    Locked = 0,
    Released = 1,
    EarlyWithdrawn = 2,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PlanType {
    /// Unlocks automatically on a set date.      e.g. Rent, Electricity.
    Bill = 0,
    /// Locked permanently until the user breaks it.
    Emergency = 1,
    /// Locked until a target date or target amount. Behaves exactly like Bill
    /// on-chain; only the UI treats them differently for presentation.
    Savings = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub id: u32,
    pub name: String,
    pub amount: i128,
    pub plan_type: PlanType,
    pub status: PlanStatus,
    /// 0 = "always locked" (Emergency plans).
    pub unlock_date: u64,
    pub created_at: u64,
}

/// Manual From impls so plan types could be passed as `u32` from Soroban clients
/// without needing a custom resolver on the calling side.
impl From<u32> for PlanType {
    fn from(v: u32) -> Self {
        match v {
            1 => PlanType::Emergency,
            2 => PlanType::Savings,
            _ => PlanType::Bill,
        }
    }
}

impl From<PlanType> for u32 {
    fn from(p: PlanType) -> Self {
        p as u32
    }
}

impl From<u32> for PlanStatus {
    fn from(v: u32) -> Self {
        match v {
            1 => PlanStatus::Released,
            2 => PlanStatus::EarlyWithdrawn,
            _ => PlanStatus::Locked,
        }
    }
}

impl From<PlanStatus> for u32 {
    fn from(s: PlanStatus) -> Self {
        s as u32
    }
}

/// A tiny struct used only for nicely typed event data on init.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VaultInitData {
    pub owner: Address,
    pub token: Address,
}
