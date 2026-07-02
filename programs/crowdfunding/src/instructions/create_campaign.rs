use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::CrowdfundingError, Campaign, CAMPAIGN_SEED};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = maker,
        space = 8 + Campaign::INIT_SPACE,
        seeds = [CAMPAIGN_SEED, maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint,
        associated_token::authority = campaign,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_campaign(
    ctx: Context<CreateCampaign>,
    id: u64,
    goal: u64,
    deadline: i64,
) -> Result<()> {
    require!(goal > 0, CrowdfundingError::InvalidGoal);
    let current_time = Clock::get()?.unix_timestamp;
    require!(deadline > current_time, CrowdfundingError::InvalidDeadline);
    ctx.accounts.campaign.set_inner(Campaign {
        maker: ctx.accounts.maker.key(),
        id,
        mint: ctx.accounts.mint.key(),
        goal,
        deadline,
        status: crate::CampaignStatus::Ongoing,
        bump: ctx.bumps.campaign,
    });
    Ok(())
}
