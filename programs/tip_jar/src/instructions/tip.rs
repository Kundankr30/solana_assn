use crate::error::ErrorCode;
use crate::state::TipJar;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::{prelude::*, system_program};

#[derive(Accounts)]
pub struct Tip<'info> {
    #[account(mut)]
    pub tipper: Signer<'info>,
    #[account(
        mut,
        seeds = [b"tip_jar", jar.owner.as_ref()],
        bump = jar.bump,
    )]
    pub jar: Account<'info, TipJar>,
    pub system_program: Program<'info, System>,
}
pub fn handler(ctx: Context<Tip>, amount: u64) -> Result<()> {
    transfer(
        CpiContext::new(
            system_program::ID,
            Transfer {
                from: ctx.accounts.tipper.to_account_info(),
                to: ctx.accounts.jar.to_account_info(),
            },
        ),
        amount,
    )?;
    let jar = &mut ctx.accounts.jar;
    jar.total_tips = jar
        .total_tips
        .checked_add(amount)
        .ok_or(ErrorCode::OverFlow)?;
    msg!(
        "Tip of {} lamports received. Total: {}",
        amount,
        jar.total_tips
    );
    Ok(())
}
