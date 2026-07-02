use anchor_lang::prelude::*;

#[error_code]
pub enum CrowdfundingError {
    #[msg("Invalid goal")]
    InvalidGoal,
    #[msg("Invalid deadline")]
    InvalidDeadline,
    #[msg("Cannot ended")]
    CampaignEnded,
    #[msg("Campaign still ongoing")]
    CampaignOngoing,
    #[msg("Campaign must be finalized and goal met")]
    InvalidWithdraw,
    #[msg("Campaign must be finalized and goal not met")]
    InvalidRefund,
    #[msg("Overflow")]
    Overflow,
}
