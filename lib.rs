use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

declare_id!("JWX1rFFNxXQG5Mcqe6fjqFAJ4GHvyXHAfyxNeojpQEC");

#[program]
pub mod vote_program {
    use super::*;

    pub fn initialize_votes(ctx: Context<InitializeVotes>) -> Result<()> {
        let votes_account = &mut ctx.accounts.votes_account;
        votes_account.number_of_votes = 0;
        Ok(())
    }

    pub fn create_vote(
        ctx: Context<CreateVote>,
        topic: String,
        options: Vec<String>,
        voting_days: i32,
    ) -> Result<()> {
        let votes_account = &mut ctx.accounts.votes_account;
        votes_account.number_of_votes += 1;

        let vote_account = &mut ctx.accounts.vote_account;
        vote_account.topic = topic;
        vote_account.voting_deadline = Clock::get()?.unix_timestamp + (voting_days * 86400) as i64;
        vote_account.options = options
            .into_iter()
            .map(|opt| VoteOption {
                name: opt,
                votes: 0,
            }).collect();
        Ok(())
    }

    pub fn vote(ctx: Context<CastVote>, option_index: u8) -> Result<()> {
        let vote_account = &mut ctx.accounts.vote_account;
        require!(
            Clock::get()?.unix_timestamp < vote_account.voting_deadline,
            VotingErr::VotingIsOver
        );
        require!(
            option_index < vote_account.options.len() as u8,
            VotingErr::InvalidOption
        );

        let voter = &mut ctx.accounts.voter_account;
        require!(!voter.voted, VotingErr::AlreadyVoted);
        voter.voted = true;
        voter.option_index = option_index;

        vote_account.options[option_index as usize].votes += 1;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVotes<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub votes_account: Account<'info, Votes>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateVote<'info> {
    #[account(mut)]
    pub votes_account: Account<'info, Votes>,
    #[account(init, payer = user, space = 8 + 32 + 8 + (32 + 8) * 10, seeds = ["vote".as_bytes(), &(votes_account.number_of_votes+1).to_le_bytes()], bump)]
    pub vote_account: Account<'info, VoteAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub vote_account: Account<'info, VoteAccount>,
    #[account(init, payer = user, space = 8 + 1 + 1, seeds = [vote_account.key().as_ref(), user.key().as_ref()], bump)]
    pub voter_account: Account<'info, Voter>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Votes {
    pub number_of_votes: u64,
}

#[account]
pub struct VoteAccount {
    pub topic: String,
    pub voting_deadline: i64,
    pub options: Vec<VoteOption>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VoteOption {
    pub name: String,
    pub votes: u64,
}

#[account]
pub struct Voter {
    pub voted: bool,
    pub option_index: u8,
}

#[error_code]
pub enum VotingErr {
    #[msg("Voting period is over!")]
    VotingIsOver,
    #[msg("You have already voted!")]
    AlreadyVoted,
    #[msg("Invalid option!")]
    InvalidOption,
}