use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::CrowdfundingError, Campaign, CampaignStatus, CAMPAIGN_SEED};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Finalize<'info> {
    /// CHECK: part of campaign's seed and has_one check
    pub maker: UncheckedAccount<'info>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        has_one = maker,
        has_one = mint,
        seeds = [CAMPAIGN_SEED, maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump = campaign.bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        associated_token::mint = mint,
        associated_token::authority = campaign,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_finalize(ctx: Context<Finalize>, _id: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time > ctx.accounts.campaign.deadline,
        CrowdfundingError::CampaignOngoing
    );
    require_eq!(
        ctx.accounts.campaign.status,
        CampaignStatus::Ongoing,
        CrowdfundingError::CampaignEnded
    );
    let status = if ctx.accounts.vault.amount >= ctx.accounts.campaign.goal {
        CampaignStatus::GoalMet
    } else {
        CampaignStatus::GoalNotMet
    };
    ctx.accounts.campaign.status = status;
    Ok(())
}
