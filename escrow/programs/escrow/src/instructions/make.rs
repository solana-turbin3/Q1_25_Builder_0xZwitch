use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::states::*;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
    mut,
    associated_token::mint = mint_a,
    associated_token::authority = maker,
  )]
    pub maker_mint_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
    init,
    payer = maker,
    space = 8 + EscrowState::INIT_SPACE,
    seeds = [b"escrow", maker.key.as_ref(), seed.to_le_bytes().as_ref()],
    bump,
  )]
    pub escrow: Account<'info, EscrowState>,
    #[account(
    init,
    payer = maker,
    associated_token::mint = mint_a,
    associated_token::authority = escrow,
  )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Make<'info> {
    pub fn init_escrow_state(
        &mut self,
        seed: u64,
        receive_amount: u64,
        bumps: MakeBumps,
    ) -> Result<()> {
        self.escrow.set_inner(EscrowState {
            seed,
            receive_amount,
            bump: bumps.escrow,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
        });
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // Transfer token a to escrow
        let cpi_program = self.token_program.to_account_info();
        let cpi_account = TransferChecked {
            mint: self.mint_a.to_account_info(),
            from: self.maker_mint_a_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);

        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;
        Ok(())
    }
}
