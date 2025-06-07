#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

declare_id!("HXznYGd1fHXpYX5kuAd4V7EHWCJszzA7NiWcaSJHgRDE");

pub const AUTHORITY: Pubkey = pubkey!("BPFra4hXKUbS9voZK5R3pvfk8WtDBVcdSUaLPEjxHfWD");
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

#[program]
pub mod account_address_program {
    use super::*;

    pub fn create_account_address(ctx: Context<CreateAccountAddress>, account_name: String, _account_number: String, _bank_code: String, _region: String) -> Result<()> {
        let account_info = &mut ctx.accounts.account_info;
        account_info.account_name = account_name;

        let rent = Rent::get()?;
        let rent_exception = rent.minimum_balance(0);

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.account_address.to_account_info()
                }
            ),
            rent_exception
        )?;

        Ok(())
    }

    pub fn transfer_sol_to_pool(ctx: Context<TransferSolToPool>, _account_number: String, _bank_code: String, _region: String) -> Result<()> {
        let system_program_id = ctx.accounts.system_program.to_account_info();
        let from_pubkey = ctx.accounts.account_address.to_account_info();
        let to_pubkey = ctx.accounts.tx_pool.to_account_info();
        let balance = from_pubkey.get_lamports();
        let bump = ctx.bumps.account_address;
        let signer_seeds: &[&[&[u8]]] = &[&[
            _account_number.as_bytes(),
            _bank_code.as_bytes(),
            _region.as_bytes(),
            &[bump]],
        ];

        let rent_exception = Rent::get()?.minimum_balance(0);
        let transferable_balace = balance.checked_sub(rent_exception)
            .ok_or_else(|| error!(ErrorCode::InvalidTransferAmount))?;

        let cpi_context = CpiContext::new(
            system_program_id,
            system_program::Transfer {
                from: from_pubkey,
                to: to_pubkey
            }
        ).with_signer(signer_seeds);
        system_program::transfer(cpi_context, transferable_balace)?;

        Ok(())
    }

    pub fn transfer_token_to_pool(ctx: Context<TransferTokenToPool>, _account_number: String, _bank_code: String, _region: String) -> Result<()> {
        // let token_interface = TokenInterface::new(ctx.accounts.account_token_address.to_account_info());
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let from_pubkey = ctx.accounts.account_token_address.to_account_info();
        let to_pubkey = ctx.accounts.tx_pool_token_account.to_account_info();
        let balance = ctx.accounts.account_token_address.amount;
        let decimals = ctx.accounts.mint.decimals;
        let bump = ctx.bumps.account_address;
        let signer_seeds: &[&[&[u8]]] = &[&[
            _account_number.as_bytes(),
            _bank_code.as_bytes(),
            _region.as_bytes(),
            &[bump]],
        ];

        let cpi_accounts = TransferChecked {
            mint: ctx.accounts.mint.to_account_info(),
            from: from_pubkey,
            to: to_pubkey,
            authority: ctx.accounts.account_address.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);
        token_interface::transfer_checked(cpi_context, balance, decimals)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(account_name: String, _account_number: String, _bank_code: String, _region: String)]
pub struct CreateAccountAddress<'info> {
    #[account(mut, constraint = payer.key() == AUTHORITY @ ErrorCode::InvalidAuthority)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [_account_number.as_bytes(), _bank_code.as_bytes(), _region.as_bytes()],
        bump,
        // space = ANCHOR_DISCRIMINATOR_SIZE,
        // payer = payer,
    )]
    pub account_address: SystemAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = ANCHOR_DISCRIMINATOR_SIZE  + std::mem::size_of::<AccountAddress>(),
        seeds = [b"info", _account_number.as_bytes(), _bank_code.as_bytes(), _region.as_bytes()],
        bump,
    )]
    pub account_info: Account<'info, AccountAddress>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_account_number: String, _bank_code: String, _region: String)]
pub struct TransferSolToPool<'info> {
    #[account(mut, constraint = payer.key() == AUTHORITY @ ErrorCode::InvalidAuthority)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [_account_number.as_bytes(), _bank_code.as_bytes(), _region.as_bytes()],
        bump,
    )]
    pub account_address: SystemAccount<'info>,

    #[account(mut)]
    pub tx_pool: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_account_number: String, _bank_code: String, _region: String)]
pub struct TransferTokenToPool<'info> {
    #[account(mut, constraint = payer.key() == AUTHORITY @ ErrorCode::InvalidAuthority)]
    pub payer: Signer<'info>,
    #[account()]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [_account_number.as_bytes(), _bank_code.as_bytes(), _region.as_bytes()],
        bump,
    )]
    pub account_address: SystemAccount<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = account_address,
        token::token_program = token_program
    )]
    pub account_token_address: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub tx_pool: SystemAccount<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = tx_pool,
        token::token_program = token_program
    )]
    pub tx_pool_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AccountAddress {
    pub account_name: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only the authority can create an account address")]
    InvalidAuthority,
    #[msg("Can't detaermint transfer amount due to rent exception")]
    InvalidTransferAmount,
}
