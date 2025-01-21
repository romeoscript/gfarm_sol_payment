use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("DuzSh3eZPciBtzVMDsvQDzbRqshuWX5s7PqjSj6RZMgc");

#[program]
pub mod payment_system {
    use super::*;

    pub fn make_payment(ctx: Context<MakePayment>, amount: u64) -> Result<()> {
        // Transfer SOL from payer to payee
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.payee.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!("Payment processed: {} lamports", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct MakePayment<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    /// CHECK: This is safe as we're just transferring SOL
    pub payee: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}