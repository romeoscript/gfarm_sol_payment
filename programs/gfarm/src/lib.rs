use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("DuzSh3eZPciBtzVMDsvQDzbRqshuWX5s7PqjSj6RZMgc");

#[program]
pub mod payment_system {
    use super::*;

    // Initialize a new payment
    pub fn initialize_payment(
        ctx: Context<InitializePayment>,
        amount: u64,
        payment_id: String,
        description: String,
    ) -> Result<()> {
        let payment = &mut ctx.accounts.payment;
        payment.amount = amount;
        payment.payer = ctx.accounts.payer.key();
        payment.payee = ctx.accounts.payee.key();
        payment.paid = false;
        payment.payment_id = payment_id;
        payment.description = description;
        payment.timestamp = Clock::get()?.unix_timestamp;

        msg!("Payment initialized: {} lamports", amount);
        Ok(())
    }

    // Process the payment
    pub fn process_payment(ctx: Context<ProcessPayment>) -> Result<()> {
        let payment = &mut ctx.accounts.payment;
        require!(!payment.paid, PaymentError::AlreadyPaid);

        // Transfer SOL from payer to payee
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.payee.to_account_info(),
                },
            ),
            payment.amount,
        )?;

        payment.paid = true;
        payment.paid_at = Some(Clock::get()?.unix_timestamp);

        msg!("Payment processed: {} lamports", payment.amount);
        Ok(())
    }

    // Cancel a payment
    pub fn cancel_payment(ctx: Context<CancelPayment>) -> Result<()> {
        let payment = &mut ctx.accounts.payment;
        require!(!payment.paid, PaymentError::AlreadyPaid);
        
        msg!("Payment cancelled: {}", payment.payment_id);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u64, payment_id: String, description: String)]
pub struct InitializePayment<'info> {
    #[account(
        init,
        payer = payer,
        space = Payment::LEN,
        seeds = [b"payment", payment_id.as_bytes()],
        bump
    )]
    pub payment: Account<'info, Payment>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: This is safe as it's just storing the payee's address
    pub payee: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(
        mut,
        constraint = payment.payer == payer.key(),
        constraint = payment.payee == payee.key(),
    )]
    pub payment: Account<'info, Payment>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    /// CHECK: This is safe as we're just transferring SOL
    pub payee: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelPayment<'info> {
    #[account(
        mut,
        constraint = payment.payer == payer.key(),
        close = payer
    )]
    pub payment: Account<'info, Payment>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[account]
pub struct Payment {
    pub amount: u64,
    pub payer: Pubkey,
    pub payee: Pubkey,
    pub paid: bool,
    pub payment_id: String,
    pub description: String,
    pub timestamp: i64,
    pub paid_at: Option<i64>,
}

impl Payment {
    const LEN: usize = 8 + // discriminator
        8 + // amount
        32 + // payer
        32 + // payee
        1 + // paid
        40 + // payment_id (max length)
        100 + // description (max length)
        8 + // timestamp
        9; // paid_at (1 + 8)
}

#[error_code]
pub enum PaymentError {
    #[msg("This payment has already been processed")]
    AlreadyPaid,
}