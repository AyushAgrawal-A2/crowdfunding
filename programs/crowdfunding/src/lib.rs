pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("EZ8K1FWpiYn8dDH9NLbAiz54ixg7pKEL8pCDMXTZwF7d");

#[program]
pub mod crowdfunding {
    use super::*;

    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        id: u64,
        goal: u64,
        deadline: i64,
    ) -> Result<()> {
        crate::instructions::create_campaign::handle_create_campaign(ctx, id, goal, deadline)
    }

    pub fn contribute(ctx: Context<Contribute>, id: u64, amount: u64) -> Result<()> {
        crate::instructions::contribute::handle_contribute(ctx, id, amount)
    }

    pub fn finalize(ctx: Context<Finalize>, id: u64) -> Result<()> {
        crate::instructions::finalize::handle_finalize(ctx, id)
    }

    pub fn refund(ctx: Context<Refund>, id: u64) -> Result<()> {
        crate::instructions::refund::handle_refund(ctx, id)
    }

    pub fn withdraw(ctx: Context<Withdraw>, id: u64) -> Result<()> {
        crate::instructions::withdraw::handle_withdraw(ctx, id)
    }
}
