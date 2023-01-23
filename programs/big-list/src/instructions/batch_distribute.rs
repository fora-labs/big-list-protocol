use anchor_lang::{
    prelude::*,
    solana_program::{self, instruction::Instruction, system_program},
};
use clockwork_sdk::{
    self,
    state::{Thread, ThreadResponse, Trigger},
    ThreadProgram,
};

use crate::{
    state::{BatchProcess, BatchProcessStatus, BigList},
    utils::{get_current_indices, get_j, get_k},
};

#[derive(Accounts)]
// #[instruction(id: String)]
pub struct BatchDistribute<'info> {
    #[account(seeds = [b"batch_process".as_ref(), b"my_big_list".as_ref(), authority.key().as_ref()], bump)]
    pub batch_process: Account<'info, BatchProcess>,
    #[account(
        has_one = authority,
        seeds = [b"my_big_list".as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list: Account<'info, BigList>,
    #[account(
        seeds = [get_j(0).to_string().as_ref(), b"my_big_list".as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list_j: Account<'info, BigList>,
    #[account(
        seeds = [get_k(0).to_string().as_ref(), get_j(0).to_string().as_ref(), b"my_big_list".as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list_k: Account<'info, BigList>,
    // #[account(mut)]
    /// CHECK: can be anyone
    pub authority: AccountInfo<'info>,
    #[account(mut, address = Thread::pubkey(batch_process.key(), "my_big_list".to_string()))]
    pub batch_processor_thread: Account<'info, Thread>,
    pub thread_program: Program<'info, ThreadProgram>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}


pub fn process<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, BatchDistribute<'info>>,
) -> Result<(ThreadResponse)> {
    let batch_process = &mut ctx.accounts.batch_process;
    let big_list = &ctx.accounts.big_list;
    let big_list_j = &ctx.accounts.big_list_j;
    let big_list_k = &ctx.accounts.big_list_k;
    let thread_program = &ctx.accounts.thread_program;
    let system_program = &ctx.accounts.system_program;
    let batch_processor_thread = &ctx.accounts.batch_processor_thread;
    let authority = &ctx.accounts.authority;


    batch_process.status = BatchProcessStatus::Processing;

    let next_total = batch_process.total_processed + ctx.remaining_accounts.len() as u32;
    let next_position = get_current_indices(next_total);
    batch_process.total_processed = next_total;
    batch_process.position = [next_position.0, next_position.1, next_position.2];

    msg!("Batch Total: {}", batch_process.total_processed);

    for account_info in ctx.remaining_accounts {
        // Add 5 SOL to thread to cover fees
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                &ctx.accounts.batch_processor_thread.key(),
                &account_info.key(),
                100_000,
            ),
            &[
                ctx.accounts.batch_processor_thread.to_account_info(),
                account_info.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        msg!("Payed {} 100,000 Lamports", account_info.key());
    }

    let mut accounts = vec![
        AccountMeta::new(batch_process.key(), false),
        AccountMeta::new_readonly(big_list.key(), false),
        AccountMeta::new_readonly(big_list_j.key(), false),
        AccountMeta::new_readonly(big_list_k.key(), false),
        AccountMeta::new_readonly(authority.key(), false),
        AccountMeta::new(batch_processor_thread.key(), true),
        AccountMeta::new_readonly(thread_program.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
    ];
    
    for i in batch_process.total_processed..next_total {
        let pubkey = big_list_k.elements[i as usize];
        let meta =  AccountMeta::new(pubkey, false);
        accounts.push(meta);
    }

    let create_batch_distribution_ix = Instruction {
        program_id: crate::ID,
        accounts: accounts,
        data: clockwork_sdk::utils::anchor_sighash("batch_distribute").into(),
    };

    Ok(ThreadResponse {
        kickoff_instruction: None,
        next_instruction: Some(create_batch_distribution_ix.into())
    })
}
