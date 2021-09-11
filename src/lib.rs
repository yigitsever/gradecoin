//! # Drocoin
//!
//! ## Services
//! ### /register
//! - Student creates their own 2048 bit RSA `keypair`
//! - Downloads `Drocoin`'s Public Key from Moodle
//! - Encrypts their JSON wrapped `Public Key` and `Student ID` using Drocoin's Public Key
//! - Their public key is now in our Db under [`schema::User::public_key`] and can be used to sign their JWT's during requests
//!
//! ### /transaction
//! - offer a [`schema::Transaction`] - POST request
//!     - The request should have `Authorization`
//!     - The request header should be signed by the Public Key of the `by` field in the transaction
//! - fetch the list of `Transaction`s - GET request
//!
//! ### /block
//! - offer a [`schema::Block`] - POST request
//!     - The request should have `Authorization`
//!     - The [`schema::Block::transaction_list`] of the block should be a subset of [`schema::Db::pending_transactions`]
//! - fetch the last accepted [`schema::Block`] - GET request
//!
//! `Authorization`: The request header should have Bearer JWT.Token signed with Student Public Key

pub mod custom_filters;
pub mod handlers;
pub mod routes;
pub mod schema;

use lazy_static::lazy_static;
use std::fs;

lazy_static! {
    static ref PRIVATE_KEY: String =
        fs::read_to_string("secrets/drocoin.pem").expect("error reading 'secrets/drocoin.pem'");
}
