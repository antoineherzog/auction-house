use anchor_lang::prelude::*;

use agnostic_orderbook::critbit::LeafNode;
use agnostic_orderbook::orderbook::OrderBookState;
use agnostic_orderbook::state::Side;

use account_contexts::*;
use account_data::*;
use error::*;

mod account_contexts;
mod account_data;
mod consts;
mod error;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod auction_house {
    use consts::{CALLBACK_ID_LEN, CALLBACK_INFO_LEN};

    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        let this = Side::Ask;
        match this {
            Side::Bid => {
                msg!("hey it's a bid");
            }
            Side::Ask => {
                msg!("hey it's an ask");
            }
        }
        Ok(())
    }

    #[access_control(InitAuction::validate_args(args))]
    pub fn init_auction(ctx: Context<InitAuction>, args: InitAuctionArgs) -> Result<()> {
        let auction: &mut Auction = &mut *ctx.accounts.auction;
        auction.start_time = args.start_time;

        // TODO update auction account with a bunch of deets

        // TODO initialize the orderbook accounts

        // Basically everything from create_market.rs in aob except
        // the account checks
        // Load market
        // load event queue
        // init slab

        Ok(())
    }

    pub fn init_open_orders(_ctx: Context<InitOpenOrders>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))

        // TODO Just update ctx.accounts.auction with relevant values

        // Ok(())
    }

    pub fn new_order(_ctx: Context<NewOrder>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))

        // TODO
        // load the orderbook
        // load the event queue
        // Put together new_order::params
        // Send the new order
        // Update relevant values on the open orders account
        // order id, quote token locked, base token locked

        // Ok(())
    }

    pub fn cancel_order(_ctx: Context<NewOrder>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))

        // TODO
        // Any non-aob accounts we already have loaded up
        // Check the order_id is in the vector, just do a loop over the order_ids vector, it's no big deal
        // Load the aob market state
        // Load the orderbook
        // Get the slab, remove order_id by key, get the order details from the node
        // Match the side of the order
        // Update user account quote/base tokens locked/free
        // Delete the order_id from vector of open orders

        // Ok(())
    }

    pub fn new_encrypted_order(_ctx: Context<NewEncryptedOrder>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))

        // TODO
        // Args
        // Public key
        // Encryption values - nonce + cipher text
        // quote / base token quantity
        // Access control
        // Check the public key is correct if has already been added
        // Function
        // Check if public key hasn't already been added and add it if not
        // Create an EncryptedOrder struct using the nonce, cipher text and token quantity
        // Check that the same Encrypted Order struct isn't already in the vector
        // Add it to the encrypted orders vector
        // Match the side of the account
        // Transfer over the token amount of currency to the base / quote vault
        // Add the values to the base / quote locked

        // Ok(())
    }

    pub fn cancel_encrypted_order(_ctx: Context<NewEncryptedOrder>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))

        // TODO
        // Args
        // cipher text + nonce of order to cancel
        // Access control
        // Bid / Ask time hasn't finished
        // Function
        // Loop over the encrypted orders to find the cipher text that matches the input
        // Error if the order isn't found. There's a special - end of loop call function option
        // Match the side of the account
        // Reduce the order's token_locked from base/quote token locked
        // Transfer token_locked quantity of tokens base /quote token vault

        // Ok(())
    }

    pub fn decrypt_order(_ctx: Context<DecryptOrder>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))

        // TODO
        // Args
        // Secret key
        // Access control
        // 	After order period has finished
        // 	Before decryption period has finished
        // Function
        // Iterate over all the encrypted orders
        // Decrypt the price and quantity of each order from the cipher text
        // 	Validate the decrypted values
        // 	Price lots
        // 	Quantity lots
        // 	Sufficient quote/base tokens locked
        // 	Anything else that is validated by unencrypted order
        // 	If this is a bid and price * quantity < locked tokens
        // 	reduce remaining amount from quote tokens locked
        // 	Increase the quote token free by remaining amount
        // 	Post the order to the AOB, same as in new uncencrypted order, and add the order id to orders

        // Ok(())
    }

    pub fn calculate_clearing_price(
        ctx: Context<CalculateClearingPrice>,
        limit: u16,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;

        let mut orderbook = OrderBookState::new_safe(
            &ctx.accounts.bid_queue.to_account_info(),
            &ctx.accounts.ask_queue.to_account_info(),
            CALLBACK_INFO_LEN,
            CALLBACK_ID_LEN,
        )?;

        let bid_slab = orderbook.get_tree(Side::Bid);
        let mut bid_iter = bid_slab.clone().into_iter(false);
        let mut current_bid: LeafNode;
        let ask_slab = orderbook.get_tree(Side::Ask);
        let mut ask_iter = ask_slab.clone().into_iter(true);
        let mut current_ask: LeafNode;

        if auction.current_ask_key == 0 && auction.current_bid_key == 0 {
            // TODO Add an access control that verifies there's orders on both orderbooks
            current_bid = bid_iter.next().unwrap();
            auction.current_bid_key = current_bid.key;
            current_ask = ask_iter.next().unwrap();
            auction.current_ask_key = current_ask.key;
        } else {
            // TODO add a fake serialization function that iterates over the iterators
            // until it reaches the current bid/ask key. And errors if it can't find them
            current_bid = bid_iter.next().unwrap();
            current_ask = ask_iter.next().unwrap();
        }

        for _ in 0..limit {
            let bid_quantity_remaining = current_bid
                .base_quantity
                .checked_sub(auction.current_bid_quantity_filled)
                .unwrap();
            let ask_quantity_remaining = current_ask
                .base_quantity
                .checked_sub(auction.current_ask_quantity_filled)
                .unwrap();
            let is_bid_gte_ask = bid_quantity_remaining >= ask_quantity_remaining;
            match is_bid_gte_ask {
                true => {
                    // Ask order is fully filled
                    auction.current_bid_quantity_filled = auction
                        .current_bid_quantity_filled
                        .checked_add(ask_quantity_remaining)
                        .unwrap();
                    auction.total_quantity_filled_so_far = auction
                        .total_quantity_filled_so_far
                        .checked_add(ask_quantity_remaining)
                        .unwrap();
                    let new_ask = ask_iter.next();
                    match new_ask {
                        Some(ask) => {
                            if ask.price() > current_bid.price() {
                                // price have crossed
                                auction.has_found_clearing_price = true;
                                break;
                            }
                            current_ask = ask;
                            auction.current_ask_key = ask.key;
                            auction.current_ask_quantity_filled = 0;
                        }
                        None => {
                            // No more asks
                            auction.has_found_clearing_price = true;
                            break;
                        }
                    }
                }
                false => {
                    // Bid order is fully filled
                    auction.current_ask_quantity_filled = auction
                        .current_ask_quantity_filled
                        .checked_add(bid_quantity_remaining)
                        .unwrap();
                    auction.total_quantity_filled_so_far = auction
                        .total_quantity_filled_so_far
                        .checked_add(bid_quantity_remaining)
                        .unwrap();
                    let new_bid = bid_iter.next();
                    match new_bid {
                        Some(bid) => {
                            if current_ask.price() > bid.price() {
                                // price have crossed
                                auction.has_found_clearing_price = true;
                                break;
                            }
                            current_bid = bid;
                            auction.current_bid_key = bid.key;
                            auction.current_bid_quantity_filled = 0;
                        }
                        None => {
                            // No more bids
                            auction.has_found_clearing_price = true;
                            break;
                        }
                    }
                }
            }
        }

        if auction.has_found_clearing_price {
            auction.total_quantity_matched = auction.total_quantity_filled_so_far;
            auction.final_bid_price = current_bid.price();
            auction.final_ask_price = current_ask.price();
            // For now clearing price defaults to lowest bid that fills the ask quantity
            auction.clearing_price = current_bid.price();
        }

        Err(error!(CustomErrors::NotImplemented))
        // Ok(())
    }

    pub fn match_orders(_ctx: Context<MatchOrders>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))
        // Ok(())
    }

    pub fn consume_events(_ctx: Context<ConsumeEvents>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))
        // Ok(())
    }

    pub fn prune_orders(_ctx: Context<MatchOrders>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))
        // Ok(())
    }

    pub fn settle_and_close_open_orders(_ctx: Context<SettleAndCloseOpenOrders>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))
        // Ok(())
    }

    pub fn close_aob_accounts(_ctx: Context<CloseAobAccounts>) -> Result<()> {
        Err(error!(CustomErrors::NotImplemented))
        // Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
