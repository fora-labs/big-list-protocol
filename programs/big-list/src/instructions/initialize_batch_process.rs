use anchor_lang::{
    prelude::*,
    solana_program::{self, instruction::Instruction, system_program, pubkey},
};
use clockwork_sdk::{
    self,
    state::{Thread, Trigger},
    ThreadProgram,
};

use crate::{
    state::{BatchProcess, BatchProcessStatus, BigList},
    utils::{get_j, get_k},
};

#[derive(Accounts)]
#[instruction(id: String)]
pub struct InitializeBatchProcess<'info> {
    #[account(init, seeds = [b"batch_process".as_ref(), id.as_ref(), authority.key().as_ref()], bump, payer = authority, space = 8 + 1 + 3 + 32 + 32 + 8)]
    pub batch_process: Account<'info, BatchProcess>,
    #[account(
        has_one = authority,
        seeds = [id.as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list: Account<'info, BigList>,
    #[account(
        seeds = [get_j(0).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list_j: Account<'info, BigList>,
    #[account(
        seeds = [get_k(0).to_string().as_ref(), get_j(0).to_string().as_ref(), id.as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub big_list_k: Account<'info, BigList>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, address = Thread::pubkey(batch_process.key(), id.to_string()))]
    pub batch_processor_thread: SystemAccount<'info>,
    pub thread_program: Program<'info, ThreadProgram>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn process(ctx: Context<InitializeBatchProcess>, id: String) -> Result<()> {
    let batch_process = &mut ctx.accounts.batch_process;
    let big_list = &ctx.accounts.big_list;
    let big_list_j = &ctx.accounts.big_list_j;
    let big_list_k = &ctx.accounts.big_list_k;
    let thread_program = &ctx.accounts.thread_program;
    let system_program = &ctx.accounts.system_program;
    let authority = &ctx.accounts.authority;
    let batch_processor_thread = &ctx.accounts.batch_processor_thread;

    batch_process.status = BatchProcessStatus::Processing;
    batch_process.big_list = big_list.key();
    batch_process.position = [0 as u8, 0 as u8, 0 as u8];
    batch_process.total_processed = 0;
    batch_process.authority = authority.key();

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

    let first_20 = big_list_k.elements.iter().take(20);

    for account in first_20 {
        let meta =  AccountMeta::new(account.key(), false);
        accounts.push(meta);
    }

    let create_batch_distribution_ix = Instruction {
        program_id: crate::ID,
        accounts: accounts,
        data: clockwork_sdk::utils::anchor_sighash("batch_distribute").into(),
    };
    // let trigger_insant = Trigger::Immediate,


    // Add 5 SOL to thread to cover fees
    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key,
            &ctx.accounts.batch_processor_thread.key(),
            5_000_000_000,
        ),
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.batch_processor_thread.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;


    msg!("YOO");

    let authority_key = authority.clone().key();

    let batch_process_signer_seeds: &[&[u8]] = &[
        b"batch_process",
        id.as_ref(),
        authority_key.as_ref(),
        &[*ctx.bumps.get("batch_process").unwrap()],
    ];

    let trigger = Trigger::Cron {
        schedule: "*/30 * * * * * *".into(), // 30 sec
        skippable: false
    };

    clockwork_sdk::cpi::thread_create(
        CpiContext::new_with_signer(
            thread_program.to_account_info(),
            clockwork_sdk::cpi::ThreadCreate {
                authority: batch_process.to_account_info(),
                payer: authority.to_account_info(),
                system_program: system_program.to_account_info(),
                thread: batch_processor_thread.to_account_info(),
            },
            &[batch_process_signer_seeds],
        ),
        id.clone().into(),
        create_batch_distribution_ix.into(),
        trigger,
    )?;
    Ok(())
}
