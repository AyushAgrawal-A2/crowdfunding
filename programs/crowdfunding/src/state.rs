use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Campaign {
    pub maker: Pubkey,
    pub id: u64,
    pub mint: Pubkey,
    pub goal: u64,
    pub deadline: i64,
    pub status: CampaignStatus,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignStatus {
    Ongoing,
    GoalMet,
    GoalNotMet,
}
impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CampaignStatus::Ongoing => "Ongoing",
            CampaignStatus::GoalMet => "Goal met",
            CampaignStatus::GoalNotMet => "Goal not met",
        };
        write!(f, "{s}")
    }
}

#[account]
#[derive(InitSpace)]
pub struct Donor {
    pub amount: u64,
}
