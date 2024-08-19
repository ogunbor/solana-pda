use anchor_lang::prelude::*;

declare_id!("3VQF2PA8T5BBeHXZcGBLGoKRT56vMCjtn226gGVqkg3S");

#[program]
pub mod solana_pda {
    use super::*;

    // first step is to write the 'create' function
    pub fn create(ctx: Context<Create>, name: String) -> Result<()> {
        // create the bank: // the bank needs to be mutable so deposits can be made to it, hence the "mut"
        let bank = &mut ctx.accounts.bank;

        // Convert name to fixed-size array
        let name_bytes = name.as_bytes();
        let mut name_array = [0u8; 32];
        name_array[..name_bytes.len()].copy_from_slice(name_bytes);

        // Assign bank properties
        // properties the bank would have: name, balance, owner
        bank.name = name_array;
        bank.balance = 0;
        bank.owner = ctx.accounts.user.key();

        Ok(())
    }
    // end of first step

    // fourth step which is users depositing into the bank(transfer from user's account to our pda account)
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let txn = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.bank.key(),
            amount,
        );

        anchor_lang::solana_program::program::invoke(
            &txn,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.bank.to_account_info(),
            ],
        )?;

        // Update the bank's balance
        ctx.accounts.bank.balance += amount;

        Ok(())
    }
    // end of fourth step

    // sixth step: withdraw from the pda storing the deposit to the admin who created the account
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let bank = &mut ctx.accounts.bank;
        let user = &mut ctx.accounts.user;

        // Ensure the caller is the owner of the bank account
        if bank.owner != *user.key {
            return Err(ProgramError::IllegalOwner.into());
        }

        let rent = Rent::get()?.minimum_balance(bank.to_account_info().data_len());
        // Ensure the bank has sufficient funds
        if **bank.to_account_info().lamports.borrow() - rent < amount {
            return Err(ProgramError::InsufficientFunds.into());
        }

        // Perform the transfer
        **bank.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }
    // end of sixth step
}

// second step is the derive account below: seems this is the pda section
#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer = user, space = 8 + 32 + 8 + 32, seeds = [b"bankaccount", user.key().as_ref()], bump)]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
// end of second step

// third step is to outline the properties the bank would have: name, balance, owner
#[account]
pub struct Bank {
    pub name: [u8; 32],
    pub balance: u64,
    pub owner: Pubkey,
}
// end of third step

// fifth step: for the deposit function
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
// end of fifth step

// seventh step: derive macro for the withdraw
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub bank: Account<'info, Bank>,
    #[account(mut)]
    pub user: Signer<'info>,
}
