use anchor_lang::prelude::*;
//use solana_transaction::InstructionError::InsufficientFunds;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the jar owner can perform this action.")]
    NotOwner,
    #[msg("Total tips would overflow u64")]
    OverFlow,
    #[msg("Tip Amount must be greater than zero")]
    ZeroTip,
    #[msg("Jar Does not have enough lamports for this withdraw")]
    InsufficientFunds,
}
