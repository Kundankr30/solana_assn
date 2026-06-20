use crate::error::ErrorCode;
use crate::state::TipJar;
use anchor_lang::prelude::*; // this gives you everything you need
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"tip_jar",owner.key().as_ref()],
        bump = jar.bump,
        has_one = owner@ ErrorCode::NotOwner,
    )]
    pub jar: Account<'info, TipJar>,
}
pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroTip);
    **ctx
        .accounts
        .jar
        .to_account_info()
        .try_borrow_mut_lamports()? -= amount;
    **ctx
        .accounts
        .owner
        .to_account_info()
        .try_borrow_mut_lamports()? += amount;
    msg!("Withdraw sucessfull of {} Lamports", amount);
    Ok(())
}
