use crate::state::TipJar;
use anchor_lang::prelude::*;
#[derive(Accounts)]

pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = TipJar::LEN,
        seeds = [b"tip_jar",owner.key().as_ref()],
        bump,
    )]
    pub jar: Account<'info, TipJar>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let jar = &mut ctx.accounts.jar;
    jar.owner = ctx.accounts.owner.key();
    jar.total_tips = 0;
    jar.bump = ctx.bumps.jar;
    msg!("TipJar initalized for owner:{}", jar.owner);
    Ok(())
}
