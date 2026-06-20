use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use instructions::initialize::*;
use instructions::tip::*;
use instructions::withdraw::*;

declare_id!("FSrnXoxwum2k6FnvSPfabjaK883ziQ9sW8FcpepJpKEc");

#[program]
pub mod tip_jar {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }
    pub fn tip(ctx: Context<Tip>, tip_amount: u64) -> Result<()> {
        instructions::tip::handler(ctx, tip_amount)
    }
    pub fn withdraw(ctx: Context<Withdraw>, withdraw_amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, withdraw_amount)
    }
}
