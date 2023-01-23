use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

pub mod constants;
pub mod utils;

use crate::instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod big_list {

    use super::*;

    pub fn initialize<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, InitializeBigList<'info>>,
        id: String,
        depth: u8,
    ) -> Result<()> {
        initialize::process(ctx, id, depth)
    }

    pub fn append(ctx: Context<Append>, id: String, addresses: Vec<Pubkey>) -> Result<()> {
        append::process(ctx, id, addresses)
    }

    pub fn append_rollover_k(ctx: Context<AppendRolloverK>, id: String, addresses: Vec<Pubkey>) -> Result<()> {
        append_rollover_k::process(ctx, id, addresses)
    }
}
