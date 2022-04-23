//! # Gradecoin
//!
//! ## Services
//! ### /register
//! - Student creates their own 2048 bit RSA `keypair`
//! - Downloads `Gradecoin`'s Public Key from Moodle
//! - Encrypts their JSON wrapped `Public Key` and `Student ID` using Gradecoin's Public Key
//! - Their public key is now in our Db under [`block::User::public_key`] and can be used to sign their JWT's during requests
//!
//! ### /transaction
//! - offer a [`block::Transaction`] - POST request
//!     - The request should have `Authorization`
//!     - The request header should be signed by the Public Key of the `by` field in the transaction
//! - fetch the list of `Transaction`s - GET request
//!
//! ### /block
//! - offer a [`block::Block`] - POST request
//!     - The request should have `Authorization`
//!     - The [`block::Block::transaction_list`] of the block should be a subset of [`block::Db::pending_transactions`]
//! - fetch the last accepted [`block::Block`] - GET request
//!
//! `Authorization`: The request header should have Bearer JWT.Token signed with Student Public Key
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unused_async)]

mod custom_filters;
mod db;
mod handlers;
mod routes;
mod block;
mod student;
mod config;

pub use block::{Fingerprint, Id};
use db::Db;
use lazy_static::lazy_static;
use std::fs;

#[tokio::main]
async fn main() {
    log4rs::init_file("log.conf.yml", log4rs::config::Deserializers::default()).unwrap();

    let api = routes::application(Db::new());

    // Start the server
    let point = ([127, 0, 0, 1], 8080);
    warp::serve(api).run(point).await;
}

lazy_static! {
    static ref PRIVATE_KEY: String =
        fs::read_to_string("secrets/gradecoin.pem").expect("error reading 'secrets/gradecoin.pem'");
}
