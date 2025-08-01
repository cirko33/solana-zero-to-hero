use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("AXvsRYsGWQvrtec9v3EqaVm78yvgSLgBSM1S9wWpvurp");

#[program]
mod database {
    use super::*;

    pub fn set_item(
        ctx: Context<SetItem>,
        id: u64,
        username: String,
        secret: String,
    ) -> Result<()> {
        let item = &mut ctx.accounts.item;
        item.id = id;
        item.username = username;
        item.secret = secret;

        Ok(())
    }
}

pub const ITEM_KEY: &[u8] = b"items";
pub const DISC: usize = 8;

// Derive account because we we'll store item as account
#[account]
// Derive init space so we don't need to calculate space
#[derive(InitSpace)]
pub struct Item {
    pub id: u64,
    // Max length of string so we know max size
    #[max_len(50)]
    pub username: String,
    #[max_len(50)]
    pub secret: String,
}

// Automatically generate account validation logic for this instruction using Anchor
#[derive(Accounts)]
// When it has instruction it needs to receive id to work as intended
#[instruction(id: u64)]
pub struct SetItem<'info> {
    // User that would be owner of this item
    // Needs to have `mut` so it's state could be changed
    #[account(mut)]
    pub owner: Signer<'info>,

    // Account that would store one item
    // If it has `init if needed` it doesn't need `mut` and it's changable
    #[account(
        init_if_needed,
        // Payer is owner of the item, can be changed
        payer=owner,
        // Space for Account needs to be predefined
        // It's 8 + 64 + 50 + 50
        space=DISC + Item::INIT_SPACE,
        // Setting seeds so every account have unique key out of the EC
        // In this case we have "item", owners address, id from instruction formatted to little endian bytes
        seeds=[ITEM_KEY, owner.key().as_ref(), &id.to_le_bytes()],
        // When using seeds needs bump so it knows it should create new address
        bump
    )]
    pub item: Account<'info, Item>,

    // System program, need it when creating new accounts
    // It's 1111....1111
    pub system_program: Program<'info, System>,
}