use anchor_lang::prelude::*;

declare_id!("HXznYGd1fHXpYX5kuAd4V7EHWCJszzA7NiWcaSJHgRDE");

pub const AUTHORITY: Pubkey = pubkey!("CThXy1nb8YgSDjKpWRn4znasTbEdmXggJ9hoHEMdYfiQ");
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

#[program]
pub mod account_address_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn create_account_address(ctx: Context<CreateAccountAddress>, account_name: String, _account_number: String, _bank_code: String, _region: String) -> Result<()> {
        let account_address = &mut ctx.accounts.account_address;
        account_address.wallet_address = ctx.accounts.payer.key();
        account_address.account_name = account_name;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(_account_number: String, _bank_code: String, _region: String)]
pub struct CreateAccountAddress<'info> {
    #[account(mut, constraint = payer.key() == AUTHORITY @ ErrorCode::InvalidAuthority)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = ANCHOR_DISCRIMINATOR_SIZE  + std::mem::size_of::<AccountAddress>(),
        seeds = [_account_number.as_bytes(), _bank_code.as_bytes(), _region.as_bytes()],
        bump,
    )]
    pub account_address: Account<'info, AccountAddress>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AccountAddress {
    pub wallet_address: Pubkey,
    pub account_name: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only the authority can create an account address")]
    InvalidAuthority,
}
