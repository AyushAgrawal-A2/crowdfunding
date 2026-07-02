use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{error::CrowdfundingError, Campaign, CampaignStatus, CAMPAIGN_SEED};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Withdraw<'info> {
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        has_one = maker,
        has_one = mint,
        seeds = [CAMPAIGN_SEED, maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump = campaign.bump
    )]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = campaign,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_withdraw(ctx: Context<Withdraw>, id: u64) -> Result<()> {
    require_eq!(
        ctx.accounts.campaign.status,
        CampaignStatus::GoalMet,
        CrowdfundingError::InvalidWithdraw
    );
    let maker_key = ctx.accounts.maker.key();
    let seeds = [
        CAMPAIGN_SEED,
        maker_key.as_ref(),
        &id.to_le_bytes(),
        &[ctx.accounts.campaign.bump],
    ];
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            token_interface::TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.maker_token_ata.to_account_info(),
                authority: ctx.accounts.campaign.to_account_info(),
            },
            &[&seeds[..]],
        ),
        ctx.accounts.vault.amount,
        ctx.accounts.mint.decimals,
    )?;
    token_interface::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        token_interface::CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(),
            authority: ctx.accounts.campaign.to_account_info(),
        },
        &[&seeds[..]],
    ))?;
    Ok(())
}
