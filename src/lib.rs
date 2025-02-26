// use anchor_lang::prelude::*;
// use anchor_spl::{
//     token::{self, Mint, Token, TokenAccount},
// };
// use num_integer::Roots;
// use std::cmp::min;
// declare_id!("75GJVCJNhaukaa2vCCqhreY31gaphv7XTScBChmr1ueR");

// #[program]
// pub mod amm {
//     use super::*;

//     pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
//         let pool_state = &mut ctx.accounts.pool_state;
//         pool_state.token_a_mint = ctx.accounts.token_a_mint.key();
//         pool_state.token_b_mint = ctx.accounts.token_b_mint.key();
//         pool_state.token_a_vault = ctx.accounts.token_a_vault.key();
//         pool_state.token_b_vault = ctx.accounts.token_b_vault.key();
//         pool_state.lp_token_mint = ctx.accounts.lp_token_mint.key();
//         pool_state.authority = ctx.accounts.pool_authority.key();
//         pool_state.bump = ctx.bumps.pool_authority; 
//         pool_state.fee_numerator = 3;
//         pool_state.fee_denominator = 1000;
//         pool_state.total_lp_supply = 0;
//         Ok(())
//     }

//     pub fn add_liquidity(
//         ctx: Context<AddLiquidity>,
//         amount_a: u64,
//         amount_b: u64,
//         min_lp_tokens: u64,
//     ) -> Result<()> {
//         let total_lp_supply = ctx.accounts.pool_state.total_lp_supply;
//         let lp_tokens_to_mint = if total_lp_supply == 0 {
//             (amount_a as u128 * amount_b as u128).sqrt() as u64
//         } else {
//             let reserve_a = ctx.accounts.pool_token_a.amount;
//             let reserve_b = ctx.accounts.pool_token_b.amount;
//             min(
//                 amount_a * total_lp_supply / reserve_a,
//                 amount_b * total_lp_supply / reserve_b,
//             )
//         };

//         require!(
//             lp_tokens_to_mint >= min_lp_tokens,
//             ErrorCode::SlippageExceeded
//         );

//         token::transfer(
//             CpiContext::new(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Transfer {
//                     from: ctx.accounts.user_token_a.to_account_info(),
//                     to: ctx.accounts.pool_token_a.to_account_info(),
//                     authority: ctx.accounts.user.to_account_info(),
//                 },
//             ),
//             amount_a,
//         )?;

//         token::transfer(
//             CpiContext::new(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Transfer {
//                     from: ctx.accounts.user_token_b.to_account_info(),
//                     to: ctx.accounts.pool_token_b.to_account_info(),
//                     authority: ctx.accounts.user.to_account_info(),
//                 },
//             ),
//             amount_b,
//         )?;

//         token::mint_to(
//             CpiContext::new_with_signer(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::MintTo {
//                     mint: ctx.accounts.lp_token_mint.to_account_info(),
//                     to: ctx.accounts.user_lp_token.to_account_info(),
//                     authority: ctx.accounts.pool_authority.to_account_info(),
//                 },
//                 &[&[b"pool_authority", &[ctx.accounts.pool_state.bump]]],
//             ),
//             lp_tokens_to_mint,
//         )?;

//         ctx.accounts.pool_state.total_lp_supply += lp_tokens_to_mint;
//         Ok(())
//     }

//     pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
//         let fee_numerator = ctx.accounts.pool_state.fee_numerator;
//         let fee_denominator = ctx.accounts.pool_state.fee_denominator;

//         let amount_in_with_fee = amount_in
//             .checked_mul(fee_denominator.checked_sub(fee_numerator).unwrap())
//             .unwrap()
//             .checked_div(fee_denominator)
//             .unwrap();

//         let reserve_in = ctx.accounts.pool_source_token.amount;
//         let reserve_out = ctx.accounts.pool_destination_token.amount;

//         let amount_out = calculate_output_amount(amount_in_with_fee, reserve_in, reserve_out)?;

//         require!(
//             amount_out >= minimum_amount_out,
//             ErrorCode::SlippageExceeded
//         );

//         token::transfer(
//             CpiContext::new(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Transfer {
//                     from: ctx.accounts.user_source_token.to_account_info(),
//                     to: ctx.accounts.pool_source_token.to_account_info(),
//                     authority: ctx.accounts.user.to_account_info(),
//                 },
//             ),
//             amount_in,
//         )?;

//         token::transfer(
//             CpiContext::new_with_signer(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Transfer {
//                     from: ctx.accounts.pool_destination_token.to_account_info(),
//                     to: ctx.accounts.user_destination_token.to_account_info(),
//                     authority: ctx.accounts.pool_authority.to_account_info(),
//                 },
//                 &[&[b"pool_authority", &[ctx.accounts.pool_state.bump]]],
//             ),
//             amount_out,
//         )?;

//         ctx.accounts.pool_state.last_update = Clock::get()?.unix_timestamp;
//         Ok(())
//     }

//     pub fn remove_liquidity(
//         ctx: Context<RemoveLiquidity>,
//         lp_amount: u64,
//         minimum_amount_a: u64,
//         minimum_amount_b: u64,
//     ) -> Result<()> {
//         let total_lp_supply = ctx.accounts.pool_state.total_lp_supply;
//         let reserve_a = ctx.accounts.pool_token_a.amount;
//         let reserve_b = ctx.accounts.pool_token_b.amount;

//         let amount_a = lp_amount
//             .checked_mul(reserve_a)
//             .unwrap()
//             .checked_div(total_lp_supply)
//             .unwrap();

//         let amount_b = lp_amount
//             .checked_mul(reserve_b)
//             .unwrap()
//             .checked_div(total_lp_supply)
//             .unwrap();

//         require!(
//             amount_a >= minimum_amount_a && amount_b >= minimum_amount_b,
//             ErrorCode::SlippageExceeded
//         );

//         token::burn(
//             CpiContext::new(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Burn {
//                     mint: ctx.accounts.lp_token_mint.to_account_info(),
//                     from: ctx.accounts.user_lp_token.to_account_info(),
//                     authority: ctx.accounts.user.to_account_info(),
//                 },
//             ),
//             lp_amount,
//         )?;

//         token::transfer(
//             CpiContext::new_with_signer(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Transfer {
//                     from: ctx.accounts.pool_token_a.to_account_info(),
//                     to: ctx.accounts.user_token_a.to_account_info(),
//                     authority: ctx.accounts.pool_authority.to_account_info(),
//                 },
//                 &[&[b"pool_authority", &[ctx.accounts.pool_state.bump]]],
//             ),
//             amount_a,
//         )?;

//         token::transfer(
//             CpiContext::new_with_signer(
//                 ctx.accounts.token_program.to_account_info(),
//                 token::Transfer {
//                     from: ctx.accounts.pool_token_b.to_account_info(),
//                     to: ctx.accounts.user_token_b.to_account_info(),
//                     authority: ctx.accounts.pool_authority.to_account_info(),
//                 },
//                 &[&[b"pool_authority", &[ctx.accounts.pool_state.bump]]],
//             ),
//             amount_b,
//         )?;

//         ctx.accounts.pool_state.total_lp_supply -= lp_amount;
//         Ok(())
//     }
// }

// #[derive(Accounts)]
// pub struct InitializePool<'info> {
//     #[account(mut)]
//     pub initializer: Signer<'info>,
//     #[account(
//         init,
//         payer = initializer,
//         space = 8 + size_of::<PoolState>()
//     )]
//     pub pool_state: Account<'info, PoolState>,
//     pub token_a_mint: Account<'info, Mint>,
//     pub token_b_mint: Account<'info, Mint>,
//     #[account(
//         init,
//         payer = initializer,
//         token::mint = token_a_mint,
//         token::authority = pool_authority,
//     )]
//     pub token_a_vault: Account<'info, TokenAccount>,
//     #[account(
//         init,
//         payer = initializer,
//         token::mint = token_b_mint,
//         token::authority = pool_authority,
//     )]
//     pub token_b_vault: Account<'info, TokenAccount>,
//     #[account(
//         init,
//         payer = initializer,
//         mint::decimals = 9,
//         mint::authority = pool_authority,
//     )]
//     pub lp_token_mint: Account<'info, Mint>,
//     /// CHECK: PDA account
//     #[account(seeds = [b"pool_authority"], bump)]
//     pub pool_authority: AccountInfo<'info>,
//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct AddLiquidity<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,
//     #[account(mut)]
//     pub pool_state: Account<'info, PoolState>,
//     #[account(mut)]
//     pub user_token_a: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub user_token_b: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub pool_token_a: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub pool_token_b: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub lp_token_mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub user_lp_token: Account<'info, TokenAccount>,
//     /// CHECK: PDA account
//     pub pool_authority: AccountInfo<'info>,
//     pub token_program: Program<'info, Token>,
// }

// #[derive(Accounts)]
// pub struct Swap<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,
//     #[account(mut)]
//     pub pool_state: Account<'info, PoolState>,
//     #[account(mut)]
//     pub user_source_token: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub user_destination_token: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub pool_source_token: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub pool_destination_token: Account<'info, TokenAccount>,
//     /// CHECK: PDA account
//     pub pool_authority: AccountInfo<'info>,
//     pub token_program: Program<'info, Token>,
// }

// #[derive(Accounts)]
// pub struct RemoveLiquidity<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,
//     #[account(mut)]
//     pub pool_state: Account<'info, PoolState>,
//     #[account(mut)]
//     pub user_token_a: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub user_token_b: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub pool_token_a: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub pool_token_b: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub lp_token_mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub user_lp_token: Account<'info, TokenAccount>,
//     /// CHECK: PDA account
//     pub pool_authority: AccountInfo<'info>,
//     pub token_program: Program<'info, Token>,
// }

// #[account]
// pub struct PoolState {
//     pub token_a_mint: Pubkey,
//     pub token_b_mint: Pubkey,
//     pub token_a_vault: Pubkey,
//     pub token_b_vault: Pubkey,
//     pub lp_token_mint: Pubkey,
//     pub authority: Pubkey,
//     pub bump: u8,
//     pub fee_numerator: u64,
//     pub fee_denominator: u64,
//     pub total_lp_supply: u64,
//     pub last_price_a: u64,
//     pub last_price_b: u64,
//     pub last_update: i64,
// }

// #[error_code]
// pub enum ErrorCode {
//     #[msg("Slippage tolerance exceeded")]
//     SlippageExceeded,
//     #[msg("Arithmetic overflow")]
//     Overflow,
// }

// // Helper functions
// fn calculate_output_amount(amount_in: u64, reserve_in: u64, reserve_out: u64) -> Result<u64> {
//     let numerator = amount_in
//         .checked_mul(reserve_out)
//         .ok_or(ErrorCode::Overflow)?;

//     let denominator = reserve_in
//         .checked_add(amount_in)
//         .ok_or(ErrorCode::Overflow)?;

//     Ok(numerator.checked_div(denominator).unwrap())
// }
