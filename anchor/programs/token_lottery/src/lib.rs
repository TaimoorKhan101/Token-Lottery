use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenInterface,TokenAccount};
//{mint_to, Mint, MintTo, TokenAccount, TokenInterface}
use switchboard_on_demand::accounts::RandomnessAccountData;
use anchor_spl::metadata::{
    Metadata,
    MetadataAccount,
    CreateMetadataAccountsV3,
    CreateMasterEditionV3,
    SignMetadata,
    SetAndVerifySizedCollectionItem,
    create_master_edition_v3,
    create_metadata_accounts_v3,
    sign_metadata,
    set_and_verify_sized_collection_item,
    mpl_token_metadata::types::{
            CollectionDetails,
            Creator, 
            DataV2,
        },
};   

declare_id!("89Kmtovb3JhuYirpLn6xJFANvHmwzVASRQ68KQWahuvy");

#[constant]
pub const NAME: &str = "Token Lottery Ticket #";
#[constant]
pub const SYMBOL: &str = "TLT";
#[constant]
pub const URI: &str = "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQ7sl2d1WKcd9f69uOCgvK3QWN6P2eKeVs4Lg&s";

#[program]
pub mod token_lottery {
    use anchor_spl::{metadata::{create_master_edition_v3, create_metadata_accounts_v3, mpl_token_metadata::{instructions::CreateMasterEditionV3, types::DataV2}, CreateMetadataAccountsV3}, token::{mint_to, MintTo}, token_interface};

    use super::*;

    pub fn initialize_config(ctx: Context<Initialize>, start: u64, end: u64, price: u64) -> Result<()> {
        ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
        ctx.accounts.token_lottery.start_time = start;
        ctx.accounts.token_lottery.end_time = end;
        ctx.accounts.token_lottery.ticket_price = price;
        ctx.accounts.token_lottery.authority = *ctx.accounts.payer.key;
        ctx.accounts.token_lottery.lottery_pot_amount = 0;
        ctx.accounts.token_lottery.total_tickets = 0;
        ctx.accounts.token_lottery.randomness_account = Pubkey::default();
        ctx.accounts.token_lottery.winner_chosen = false;


        Ok(())
    }

    pub fn initialize_lottery(ctx: Context<InitializeLottery> )-> Result<()>{
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint]
        ]];
        msg!("Creating Mint Account");
        token_interface::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token_interface::MintTo{
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    to: ctx.accounts.collection_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
                signer_seeds,
            ),
            1
        )?;

        msg!("Creating Metadata Account");
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3{
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.payer.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
                ),
                DataV2{
                    name: NAME.to_string(),
                    symbol: SYMBOL.to_string(),
                    uri: URI.to_string(),
                    seller_fee_basis_points: 0,
                    creators: Some(
                        vec![
                            Creator{
                                address: ctx.accounts.payer.key(),
                                verified: false,
                                share: 100,
                            }
                        ]
                    ),
                    collection: None,
                    uses: None,
                },
                true,
                true,
                Some(CollectionDetails::V1 {size: 0} ),
            )?;

            msg!("Creating Master Edition Account");

            create_master_edition_v3(
                CpiContext::new_with_signer(
                    ctx.accounts.token_metadata_program.to_account_info(),
                    anchor_spl::metadata::CreateMasterEditionV3{
                        edition: ctx.accounts.master_edition.to_account_info(),
                        mint: ctx.accounts.collection_mint.to_account_info(),
                        update_authority: ctx.accounts.payer.to_account_info(),
                        mint_authority: ctx.accounts.payer.to_account_info(),
                        payer: ctx.accounts.payer.to_account_info(),
                        metadata: ctx.accounts.metadata.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                        rent: ctx.accounts.rent.to_account_info(),
                    },
                    &signer_seeds
                ),
                Some(0)
            )?;

            msg!("Verifying Collection");

            sign_metadata(
                CpiContext::new_with_signer(
                    ctx.accounts.token_metadata_program.to_account_info(),
                    SignMetadata{
                        creator: ctx.accounts.collection_mint.to_account_info(),
                        metadata: ctx.accounts.metadata.to_account_info(),
                    }, 
                signer_seeds,
            ))?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + TokenLottery::INIT_SPACE,
        seeds = [b"token_lottery".as_ref()],
        bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLottery<'info>{
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals =0,
        mint::authority = payer,
        mint::freeze_authority = payer,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        token::mint = collection_mint,
        token::authority = collection_token_account,
        seeds = [b"collection_associated_token".as_ref()],
        bump,
    )]
    pub collection_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint.key().as_ref(),
            ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            collection_mint.key().as_ref(),
            b"edition",
            ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub master_edition: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>,

    pub associate_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct TokenLottery{
    pub bump: u8,
    pub winner: u64,
    pub winner_chosen: bool,
    pub start_time: u64,
    pub end_time: u64,
    pub lottery_pot_amount: u64,
    pub total_tickets: u64,
    pub ticket_price: u64,
    pub authority: Pubkey,
    pub randomness_account: Pubkey,
}

