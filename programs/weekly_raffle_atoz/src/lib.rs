use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

declare_id!("9JfpYC24wmPGSeoNdzQh2vbz2yqjwzcmFpQxpfQ8SFA9");

#[program]
pub mod weekly_raffle_atoz {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        start_time: i64,
        end_time: i64,
        total_supply: u64,
        ticket_price: u64,
        pool_size: u64,
        name: String,
        prize: Pubkey,
        winners: Vec<WinnerList>,
        description:String,
    
    ) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle_account;
        raffle.claimed = false;
        raffle.total_supply = total_supply;
        raffle.pool_size = pool_size;
        raffle.tickets_bought = 0;
        raffle.ticket_price = ticket_price;
        raffle.prize = prize;
        raffle.treasury = ctx.accounts.signer.key();
        raffle.start_time = start_time;
        raffle.end_time = end_time;
        raffle.name = name;
        raffle.open = true;
        raffle.winner_list = winners;
        raffle.is_active = true;
        raffle.description= description; 
        let transfer_instruction = Transfer {
            from: ctx.accounts.signer_token_account.to_account_info(),
            to: ctx.accounts.prize_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, transfer_instruction);
        anchor_spl::token::transfer(cpi_ctx, pool_size)?;
        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>, ticket_number: u64, amount: u64) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle_account;
        let participant_key = &ctx.accounts.signer.key();
        require!(ticket_number <= raffle.total_supply, RaffleError::SoldOut);
        let buy_instruction = Transfer {
            from: ctx.accounts.signer_token_account.to_account_info(),
            to: ctx.accounts.prize_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            buy_instruction,
        );
        anchor_spl::token::transfer(cpi_context, amount)?;

        ctx.accounts.ticket.owner = *participant_key;
        ctx.accounts.ticket.ticket_id = raffle.tickets_bought;
        raffle.tickets_bought += 1;
        if raffle.tickets_bought == raffle.total_supply {
            raffle.open = false;
        }

        Ok(())
    }
    pub fn change_open_state(ctx: Context<EditRaffle>, name: String) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle_account;
        let new_state: bool = !raffle.is_active;
        raffle.is_active = new_state;
        Ok(())
    }
    pub fn close_raffle(_ctx: Context<Close>) -> Result<()> {
        Ok(())
    }

    pub fn pick_winner(ctx: Context<PickWinner>, name: String) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle_account;
        let ticket_list = &raffle.tickets_bought;
        if *ticket_list == 0 {
            return Ok(());
        }
        let total_winners = raffle.winner_list.len();

        for i in 0..total_winners {
            let clock = Clock::get()?;
            let pseudo_random_number = (clock.unix_timestamp + i as i64) % raffle.tickets_bought as i64;
            let winner_id = pseudo_random_number as usize;
            let raffle_key = raffle.key();
            let binding = winner_id.to_string();
            let seeds = &[
                "ticket_atoz".as_bytes(),
                raffle_key.as_ref(),
                binding.as_bytes(),
            ];

            let (derived_address, _bump) = Pubkey::find_program_address(seeds, &ctx.program_id);
            raffle.winners.push(derived_address)
        }

        Ok(())
    }
    pub fn claim_prize(ctx: Context<ClaimPrize>, name: String) -> Result<()> {

        let raffle = &mut ctx.accounts.raffle_account;
        let ticket = &mut ctx.accounts.winning_ticket;
        let index = raffle.winners.iter().position(|key| key == &ticket.key());
    
        if let Some(index) = index {
            // Clone the winner_pos so that we can modify it without affecting raffle
            let mut winner_pos = raffle.winner_list[index].clone();
            require!(
                ticket.owner == ctx.accounts.signer.key(),
                RaffleError::NoWinner
            );
            let binding = raffle.treasury.key();
    
            let seeds = &["atoz".as_bytes(), binding.as_ref(), name.as_ref()];
            let (_derived_address, bump) = Pubkey::find_program_address(seeds, &ctx.program_id);
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: ctx.accounts.prize_token_account.to_account_info(),
                        to: ctx.accounts.signer_token_account.to_account_info(),
                        authority: raffle.to_account_info(),
                    },
                    &[&[
                        "atoz".as_bytes(),
                        raffle.treasury.key().as_ref(),
                        name.as_ref(),
                        &[bump],
                    ]],
                ),
                winner_pos.winner_prize_amount * u64::pow(10, 6),
            )?;
    
            // Update the cloned winner_pos and assign it back to raffle.winner_list
            winner_pos.is_claimed = true;
            raffle.winner_list[index] = winner_pos;
        } else {
            return Err(RaffleError::NoWinner.into());
        }
    
        Ok(())
    }
    pub fn claim_remaining(ctx: Context<ClaimRemaining>,name:String,amount: u64)->Result<()>{
        let raffle = &mut ctx.accounts.raffle_account ; 
        let binding = raffle.treasury.key();
    
        let seeds = &["atoz".as_bytes(), binding.as_ref(), name.as_ref()];
        let (_derived_address, bump) = Pubkey::find_program_address(seeds, &ctx.program_id);
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.prize_token_account.to_account_info(),
                    to: ctx.accounts.signer_token_account.to_account_info(),
                    authority: raffle.to_account_info(),
                },
                &[&[
                    "atoz".as_bytes(),
                    raffle.treasury.key().as_ref(),
                    name.as_ref(),
                    &[bump],
                ]],
            ),
            amount,
        )?;    

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction( name: String)]
pub struct EditRaffle<'info> {
    #[account(mut, seeds = ["atoz".as_bytes(), signer.key().as_ref(), name.as_bytes()], bump)]
    pub raffle_account: Account<'info, Raffle>,
    #[account(mut)]
    pub signer: Signer<'info>,
}
#[derive(Accounts)]
#[instruction(  start_time: i64,
    end_time: i64,
    total_supply: u64,
    ticket_price: u64,
    pool_size: u64,
    name: String,
    prize: Pubkey,
    winners: Vec<WinnerList>,
    description:String,)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = 800, seeds = ["atoz".as_bytes(), signer.key().as_ref(), name.as_bytes()], bump)]
    pub raffle_account: Account<'info, Raffle>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(init, seeds = [raffle_account.key().as_ref(), "proceeds".as_bytes()], bump, payer = signer, token::mint = prize_mint, token::authority = raffle_account)]
    pub prize_token_account: Account<'info, TokenAccount>,
    pub prize_mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut, close = destination)]
    account: Account<'info, Raffle>,
    #[account(mut)]
    /// CHECK:0
    destination: AccountInfo<'info>,
}
#[account]
#[derive(Default)]
pub struct Raffle {
    pub winners: Vec<Pubkey>,
    pub total_supply: u64,
    pub tickets_bought: u64,
    pub ticket_price: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub treasury: Pubkey,
    pub claimed: bool,
    pub name: String,
    pub pool_size: u64,
    pub winner_list: Vec<WinnerList>,
    pub prize: Pubkey,
    pub open: bool,
    pub is_active: bool,
    pub description:String
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WinnerList {
    pub winner_place: i64,
    pub winner_prize_amount: u64,
    pub is_claimed:bool
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TicketBought {
    owner: Pubkey,
    ticket_id: u64,
}

#[derive(Accounts)]
#[instruction(ticket_number: u64)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub raffle_account: Account<'info, Raffle>,
    #[account(init, payer = signer, seeds = ["ticket_atoz".as_bytes(), raffle_account.key().as_ref(), ticket_number.to_string().as_bytes()], bump, space = 60)]
    pub ticket: Account<'info, Ticket>,
    #[account(mut)]
    pub prize_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Ticket {
    owner: Pubkey,
    ticket_id: u64,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct PickWinner<'info> {
    #[account(mut, seeds = ["atoz".as_bytes(), creator.key().as_ref(), name.as_bytes()], bump)]
    pub raffle_account: Account<'info, Raffle>,
    #[account(mut)]
    pub prize_mint: Account<'info, Mint>,
    #[account(mut)]
    /// CHECK
    pub creator: UncheckedAccount<'info>,
    #[account(mut, seeds = ["atoz".as_bytes(), creator.key().as_ref(), name.as_bytes()], bump)]
    /// CHECK
    pub signer: UncheckedAccount<'info>,
}
#[derive(Accounts)]
#[instruction(name: String)]
pub struct ClaimPrize<'info> {
    #[account(mut, seeds = ["atoz".as_bytes(), creator.key().as_ref(), name.as_bytes()], bump)]
    pub raffle_account: Box<Account<'info, Raffle>>,
    #[account(mut)]
    pub winning_ticket: Box<Account<'info, Ticket>>,
    #[account(mut)]
    signer: Signer<'info>,
    #[account(mut)]
    pub prize_mint: Account<'info, Mint>,
    #[account(mut)]
    ///CHECK:
    pub creator: UncheckedAccount<'info>,
    #[account(mut)]
    pub prize_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer_token_account: Account<'info, TokenAccount>,
   
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
#[instruction(name: String)]
pub struct ClaimRemaining<'info> {
    #[account(mut, seeds = ["atoz".as_bytes(), creator.key().as_ref(), name.as_bytes()], bump)]
    pub raffle_account: Box<Account<'info, Raffle>>,
    #[account(mut)]
    signer: Signer<'info>,
    #[account(mut)]
    pub prize_mint: Account<'info, Mint>,
    #[account(mut)]
    ///CHECK:
    pub creator: UncheckedAccount<'info>,
    #[account(mut)]
    pub prize_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,


}
#[error_code]
pub enum RaffleError {
    WinnerAlreadyExists,
    NoTickets,
    WinnerNotChosen,
    InvalidWinner,
    AlreadyClaimed,
    NoParticipants,
    NoWinner,
    WrongTime,
    SoldOut,
}
