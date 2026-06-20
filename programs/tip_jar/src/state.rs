use anchor_lang::prelude::*;
#[account]
pub struct TipJar {
    pub owner: Pubkey,
    pub total_tips: u64,
    pub bump: u8,
}
impl TipJar {
    // 8bytes discrimator +  32 owner + 8 total_tips + 1 bump
    pub const LEN: usize = 8 + 32 + 8 + 1;
}
