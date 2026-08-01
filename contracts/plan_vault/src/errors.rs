use soroban_sdk::contracterror;

/// Errors the contract can return. Serializable across the WASM boundary.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidUnlockDate = 5,
    UnlockDateInPast = 6,
    PlanNotFound = 7,
    PlanNotLocked = 8,
    PlanNotBill = 9,
    NotYetUnlocked = 10,
    InsufficientBalance = 11,
    EarlyWithdrawDelayNotMet = 12,
    EarlyWithdrawNotRequested = 13,
    Overflow = 14,
}
