use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_lang::solana_program::program::invoke;

declare_id!("3psmgxa88NqcgHyzvkks5DuHYgPgNHvRWSYxuYFJKaB6");

#[program]
pub mod task_1 {
    use super::*;

    pub fn propose_swap(ctx: Context<ProposeSwap>, proposer_amount: u64, accepter_amount: u64) -> Result<()> {
        let swap = &mut ctx.accounts.swap;
        swap.proposer = ctx.accounts.proposer.key();
        swap.accepter = ctx.accounts.accepter.key();
        swap.proposer_amount = proposer_amount; 
        swap.accepter_amount = accepter_amount; 
        swap.accepted = false; 
        swap.executed = false; 
        Ok(())
    }

    pub fn accept_swap(ctx: Context<AcceptSwap>) -> Result<()> {
        let swap = &mut ctx.accounts.swap;
        swap.accepted = true; 
        Ok(())
    }

    pub fn add_to_treasury(ctx: Context<AddToTreasury>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.signer;
        let treasury: &mut Account<'_, Treasury> = &mut ctx.accounts.treasury;
        treasury.owner = user.key();

        let cpi_ix = transfer(&user.key(), &treasury.key(), amount);
        invoke(&cpi_ix, 
        &[
            user.to_account_info(),
            treasury.to_account_info(),
        ])?;

        Ok(())
    }

    pub fn execute_swap(ctx: Context<ExecuteSwap>) -> Result<()> {
        let swap = &mut ctx.accounts.swap;

        require!(swap.accepted, SwapError::SwapNotAccepted);
        require!(!swap.executed, SwapError::SwapAlreadyExecuted);

        let proposer = &mut ctx.accounts.proposer_treasury.to_account_info();
        let accepter = &mut ctx.accounts.accepter_treasury.to_account_info();

        require!(proposer.lamports() >= swap.proposer_amount, SwapError::InsufficientFunds);
        require!(accepter.lamports() >= swap.accepter_amount, SwapError::InsufficientFunds);

        proposer.sub_lamports(swap.proposer_amount)?;
        accepter.sub_lamports(swap.accepter_amount)?;

        proposer.add_lamports(swap.accepter_amount)?;
        accepter.add_lamports(swap.proposer_amount)?;

        swap.executed = true;
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct Swap {
    pub proposer: Pubkey,
    pub accepter: Pubkey,
    pub proposer_amount: u64,
    pub accepter_amount: u64,
    pub accepted: bool,
    pub executed: bool,
}

#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub owner: Pubkey
}

pub const SWAP: &[u8] = b"swap";
pub const TREASURY: &[u8] = b"treasury";

#[derive(Accounts)]
pub struct ProposeSwap<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    /// CHECK: manually checked
    pub accepter: UncheckedAccount<'info>,
    #[account(
        init, 
        payer=proposer,
        space = 8 + Swap::INIT_SPACE,
        seeds = [SWAP, proposer.key().as_ref(), accepter.key().as_ref()],
        bump,
    )]
    pub swap: Account<'info, Swap>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptSwap<'info> {
    #[account(
        mut, 
        seeds = [SWAP, swap.proposer.as_ref(), swap.accepter.as_ref()],
        bump,
    )]
    pub swap: Account<'info, Swap>,
    #[account(
        mut,
        constraint = swap.accepter == accepter.key()
    )]
    pub accepter: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct AddToTreasury<'info> {
    #[account(
        mut,
        constraint = signer.lamports() >= amount @ SwapError::InsufficientFunds
    )]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer=signer,
        space=8+Treasury::INIT_SPACE,
        seeds=[TREASURY, signer.key().as_ref()],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ExecuteSwap<'info> {
    #[account(
        mut, 
        seeds = [SWAP, swap.proposer.as_ref(), swap.accepter.as_ref()],
        bump,
    )]
    pub swap: Account<'info, Swap>,
    #[account(
        mut,
        seeds=[TREASURY, swap.proposer.key().as_ref()],
        bump
    )]
    pub proposer_treasury: Account<'info, Treasury>,
    #[account(
        mut,
        seeds=[TREASURY, swap.accepter.key().as_ref()],
        bump
    )]
    pub accepter_treasury: Account<'info, Treasury>,
    #[account(mut)]
    pub signer: Signer<'info>
}

#[error_code]
pub enum SwapError {
    #[msg("Invalid proposer")]
    InvalidProposer,
    #[msg("Invalid accepter")]
    InvalidAccepter,
    #[msg("Swap not accepted")]
    SwapNotAccepted,
    #[msg("Swap already executed")]
    SwapAlreadyExecuted,
    #[msg("Insufficient funds for swap")]
    InsufficientFunds,
}