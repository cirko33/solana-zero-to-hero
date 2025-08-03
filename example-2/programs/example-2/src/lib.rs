use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("3aFp1SPtANLZemvm5ZfYRPh8Qexpxg3MF5Nj4RZvcrc9");

pub const DISC: usize = 8;
pub const WALLET: &[u8] = b"wallet";
pub const TRANSACTION: &[u8] = b"transaction";
pub const MAX_SIGNERS: usize = 30;

#[program]
mod multisig_wallet {
    use super::*;

    pub fn initialize_wallet(
        ctx: Context<InitializeWallet>,
        signers: Vec<Pubkey>,
        quorum: u8,
    ) -> Result<()> {
        require!(quorum > 0, CustomError::InvalidQuorum);
        require!(signers.len() <= MAX_SIGNERS, CustomError::TooManySigners);
        require!(
            signers.len() >= quorum as usize,
            CustomError::NotEnoughSigners
        );

        let wallet = &mut ctx.accounts.wallet;
        wallet.signers = signers;
        wallet.quorum = quorum;

        Ok(())
    }

    pub fn propose_transaction(
        ctx: Context<ProposeTransaction>,
        to: Pubkey,
        lamports: u64,
    ) -> Result<()> {
        let transaction = &mut ctx.accounts.transaction;
        let wallet = &ctx.accounts.wallet;
        transaction.to = to;
        transaction.lamports = lamports;
        transaction.signed = Vec::new();
        transaction.executed = false;
        transaction.wallet = wallet.key();

        Ok(())
    }

    pub fn approve_transaction(ctx: Context<ApproveTransaction>) -> Result<()> {
        let transaction = &mut ctx.accounts.transaction;
        let wallet = &ctx.accounts.wallet;
        let signer = ctx.accounts.signer.key();

        require!(!transaction.executed, CustomError::AlreadyExecuted);
        require!(
            wallet.key().eq(&transaction.wallet),
            CustomError::WrongWallet
        );
        require!(wallet.signers.contains(&signer), CustomError::NotSigner);
        require!(
            !transaction.signed.contains(&signer),
            CustomError::AlreadyApproved
        );

        transaction.signed.push(signer);

        Ok(())
    }

    pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
        let transaction = &mut ctx.accounts.transaction;
        let wallet = ctx.accounts.wallet.as_mut();
        let signer = ctx.accounts.signer.key();
        let to = &mut ctx.accounts.to;

        require!(!transaction.executed, CustomError::AlreadyExecuted);
        require!(
            wallet.key().eq(&transaction.wallet),
            CustomError::WrongWallet
        );
        require!(wallet.signers.contains(&signer), CustomError::NotSigner);
        require!(transaction.signed.contains(&signer), CustomError::NotSigner);
        require!(
            wallet.get_lamports() >= transaction.lamports,
            CustomError::NotEnoughLamports
        );
        require!(to.key().eq(&transaction.to), CustomError::NotSigner);
        require!(
            transaction.signed.len() as u8 >= wallet.quorum,
            CustomError::NotEnoughSigners
        );

        wallet
            .to_account_info()
            .sub_lamports(transaction.lamports)?;
        to.add_lamports(transaction.lamports)?;

        transaction.executed = true;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct Wallet {
    #[max_len(MAX_SIGNERS)]
    pub signers: Vec<Pubkey>,
    pub quorum: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Transaction {
    pub to: Pubkey,
    pub lamports: u64,
    #[max_len(MAX_SIGNERS)]
    pub signed: Vec<Pubkey>,
    pub executed: bool,
    pub wallet: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeWallet<'info> {
    #[account(
        init,
        payer=payer,
        space=DISC+Wallet::INIT_SPACE,
        seeds=[WALLET, payer.key().as_ref()],
        bump
    )]
    pub wallet: Account<'info, Wallet>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProposeTransaction<'info> {
    #[account(mut)]
    pub wallet: Account<'info, Wallet>,
    #[account(
        init,
        payer=proposer,
        space=DISC+Transaction::INIT_SPACE,
        seeds=[TRANSACTION, wallet.key().as_ref(), proposer.key().as_ref()],
        bump
    )]
    pub transaction: Account<'info, Transaction>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveTransaction<'info> {
    #[account(mut)]
    pub wallet: Account<'info, Wallet>,
    #[account(mut)]
    pub transaction: Account<'info, Transaction>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
    #[account(mut)]
    pub wallet: Box<Account<'info, Wallet>>,
    #[account(mut, has_one=wallet)]
    pub transaction: Account<'info, Transaction>,
    pub signer: Signer<'info>,
    /// CHECK: This account is validated by comparing its key to transaction.to, so manual checks are performed.
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum CustomError {
    #[msg("too many signers")]
    TooManySigners,
    #[msg("invalid quorum")]
    InvalidQuorum,
    #[msg("not signer")]
    NotSigner,
    #[msg("already approved")]
    AlreadyApproved,
    #[msg("already executed")]
    AlreadyExecuted,
    #[msg("not enough signers for quorum")]
    NotEnoughSigners,
    #[msg("wrong wallet")]
    WrongWallet,
    #[msg("not enough lamports for transaction")]
    NotEnoughLamports,
}
