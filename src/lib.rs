use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
// use solana_program::pubkey;

//  Account Configuration after Initialization
//     Config: DLHpgJ7yJhTB28WkAeru7EexeSPsbxriWM6h86rTVRF3, https://explorer.solana.com/address/DLHpgJ7yJhTB28WkAeru7EexeSPsbxriWM6h86rTVRF3?cluster=devnet
//     Vault:  4cbC12KPYk7PcamVGNBAqSrvHoxSV7Y3wyAzt9DU9KcQ, https://explorer.solana.com/address/4cbC12KPYk7PcamVGNBAqSrvHoxSV7Y3wyAzt9DU9KcQ?cluster=devnet
//     Program: CozaAtw4wmRh1WT2zeLTu6jWYgrJotB7TRYdZZbusBNK, https://explorer.solana.com/address/CozaAtw4wmRh1WT2zeLTu6jWYgrJotB7TRYdZZbusBNK?cluster=devnet

// program_id
declare_id!("CozaAtw4wmRh1WT2zeLTu6jWYgrJotB7TRYdZZbusBNK");

// MIN_MAX
pub const MIN_VAULT_BALANCE: u64 = 1; //1_000_000_000 = 1 SOL
                                      // pub const MAX_TOP_UP_AMOUNT: u64 = 5_000_000_000; //5_000_000_000 = 5 SOL

// Admin = Deployer check, constraint
// pub const ADMIN_PUBKEY: Pubkey = pubkey!("GmkBUxqDFBfaTBUQ526GBoVq8HeSXPEDQHcMNcg1Cmdb");

#[program]
pub mod waylearn_faucet {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        max_topup_amount: u64,
        initial_funding: u64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.max_topup_amount = max_topup_amount;
        config.bump = ctx.bumps.config;

        // Deposit some initial SOL into the vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.admin.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        transfer(cpi_context, initial_funding)?;

        msg!("Faucet Initialized!");
        msg!("Vault Address: {}", ctx.accounts.vault.key());
        Ok(())
    }

    pub fn update_config(ctx: Context<UpdateConfig>, new_max_topup_amount: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;

        config.max_topup_amount = new_max_topup_amount;

        msg!(
            "Faucet max top up amount updated to: {}",
            new_max_topup_amount
        );

        Ok(())
    }

    // whitelist(address) - callable by admin

    // deposit? supporter NFT :)
    // track how much has been deposited

    // user registry
    // address_type: admin, vip, user
    // address, address_type, sol_in (deposit into vault), sol_out (transfer from vault)

    //pub fn sol_drop(ctx: Context<SolTransfer>, req_amount: u64) -> Result<()> {
    pub fn sol_drop(ctx: Context<SolDrop>) -> Result<()> {
        // vault balance check
        let vault = ctx.accounts.vault.to_account_info();
        let vault_balance = vault.get_lamports();
        require!(vault_balance > MIN_VAULT_BALANCE, FaucetError::VaultEmpty);

        // whitelist check
        // FaucetError::RecipientNotWhitelisted
        let recipient = ctx.accounts.recipient.to_account_info();
        // require!( TODO__WHITELIST__CHECK, FaucetError::RecipientNotWhitelisted);

        // recipient balance check
        let max_top_up = ctx.accounts.config.max_topup_amount;
        let recipient_balance = recipient.get_lamports();
        require!(
            recipient_balance < max_top_up,
            FaucetError::RecipientHasEnoughBalance
        );

        // top up check
        let topup_amount = max_top_up - recipient_balance;
        require!(
            topup_amount < vault_balance,
            FaucetError::VaultInsufficientBalance
        );

        // perform the SOL drop
        let seeds = &[b"faucet-vault".as_ref(), &[ctx.bumps.vault]];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: vault, // vault
                to: recipient,
            },
            signer_seeds,
        );

        // The system_program::transfer instruction only works if from account is owned by the System Program.
        transfer(cpi_context, topup_amount)?;

        // record topup for address (leaderboard?)

        Ok(())
    }
}

/** CONFIG */
#[account]
#[derive(InitSpace)]
pub struct FaucetConfig {
    pub admin: Pubkey,         // 32 bytes
    pub max_topup_amount: u64, // 8 bytes
    pub bump: u8,              // 1 byte
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    // use 'has_one' to ensure 'admin' field in FaucetConfig struct matches the public key's admin field in this struct
    #[account(
        mut, 
        has_one = admin @ FaucetError::Unauthorized // better than using constraints, simply matches fields
    )]
    pub config: Account<'info, FaucetConfig>,

    pub admin: Signer<'info>,
}

/** CTOR or INIT */
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin, // fee payer
        space = 8 + FaucetConfig::INIT_SPACE, // starting 8 is anchor's discriminator, then the FaucetConfig size
        seeds = [b"faucet-config"],
        bump
    )]
    pub config: Account<'info, FaucetConfig>,

    #[account(
        // init, -- we don't need this because this is a SystemAccount
        // payer = admin, -- since we're not using init, web don't need this
        // space = 0, // no data, only lamports! -- again, no init, no need for this
        mut, 
        seeds = [b"faucet-vault"],
        bump
    )]
    pub vault: SystemAccount<'info>, // space = 0 comment above, no data then we can use AccountInfo instead

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/** Operations or Instruction-related */
#[derive(Accounts)]
pub struct SolDrop<'info> {
    pub config: Account<'info, FaucetConfig>,

    #[account(
        mut,
        seeds = [b"faucet-vault"],
        bump
    )]
    // Since we are not storing data, we can use AccountInfo instead of Account (lighter-weight)
    // but that will give us ownership issues... so best practice is using SystemAccount
    // which lets us use the transfer account from sys prog
    pub vault: SystemAccount<'info>,

    #[account(mut)]
    pub recipient: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum FaucetError {
    #[msg("Unauthorized, are you sure you are the admin?")]
    Unauthorized,
    #[msg("Top up amount higher than vault balance")]
    VaultInsufficientBalance,
    #[msg("No SOL left in vault")]
    VaultEmpty,
    #[msg("Invalid deposit amount")]
    InvalidDepositAmount,
    #[msg("Recipient is not whitelisted")]
    RecipientNotWhitelisted,
    #[msg("Recipient already holds a sufficient SOL amount")]
    RecipientHasEnoughBalance,
}
