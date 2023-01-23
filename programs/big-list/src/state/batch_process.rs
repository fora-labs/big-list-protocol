use anchor_lang::prelude::*;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum BatchProcessStatus {
    Ready,
    Processing,
}

#[account]
pub struct BatchProcess {
    pub status: BatchProcessStatus,
    pub big_list: Pubkey,
    pub total_processed: u32,
    pub position: [u8; 3],
    pub authority: Pubkey,
}