//! # Services
//!
//! All of these endpoints will be served with the URL prefix of a given network.
//! For example, if URL prefix is `testnet`, `/register` is found at `/testnet/register`.
//!
//! ## /register
//! - Student creates their own 2048 bit RSA `keypair`
//! - Downloads `Gradecoin`'s Public Key from Moodle
//! - Encrypts their JSON wrapped `Public Key` and `Student ID` using Gradecoin's Public Key
//! - Their public key is now in our Db under [`block::User::public_key`] and can be used to sign their JWT's during requests
//!
//! ## /transaction
//! - offer a [`block::Transaction`] - POST request
//!     - The request should have `Authorization`
//!     - The request header should be signed by the Public Key of the `by` field in the transaction
//! - fetch the list of `Transaction`s - GET request
//!
//! ## /block
//! - offer a [`block::Block`] - POST request
//!     - The request should have `Authorization`
//!     - The [`block::Block::transaction_list`] of the block should be a subset of [`block::Db::pending_transactions`]
//! - fetch the last accepted [`block::Block`] - GET request
//!
//! `Authorization`: The request header should have Bearer JWT.Token signed with Student Public Key
//!
//! ## /config
//! - Get the current [`config::Config`] as JSON - GET request
//!
//! # Configuration
//!
//! The default configuration file if `config.yaml`, which will run if no command line arguments are given.
//!
//! You can give one or more configuration files as command line arguments.
//! This will run all of them at the same time.
//! Make sure that the names and URL prefixes don't clash, otherwise it will lead to undefined behavior.
//!
//! Example:
//! ```sh
//! # Run both the main network (at /) and testnet (at /testnet)
//! # For example, register for main network at `localhost:8080/register`,
//! # testnet network at `localhost:8080/testnet/register`
//! $ cargo run config.yaml testnet.yaml
//! ```
//!
//! See [`config::Config`] struct for more information about the configurable fields.
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unused_async)]

mod block;
mod config;
mod custom_filters;
mod db;
mod handlers;
mod routes;
mod student;

use crate::config::Config;
pub use block::{Fingerprint, Id};
use db::Db;
use lazy_static::lazy_static;
use log::error;
use std::fs;
use warp::Filter;

#[tokio::main]
async fn main() {
    log4rs::init_file("log.conf.yml", log4rs::config::Deserializers::default()).unwrap();

    let mut args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // config.yaml is the default configuration file
        args.push("config.yaml".to_string());
    }

    let combined_routes = args
        .into_iter()
        .skip(1) // Skip the program name
        .filter_map(|filename| {
            Config::read(&filename).map(|config| routes::network(Db::new(config)))
        })
        .reduce(|routes, route| routes.or(route).unify().boxed());

    let routes = match combined_routes {
        Some(r) => r,
        None => {
            // Exit the program if there's no successfully loaded config file.
            error!("Failed to load any config files!");
            return;
        }
    };

    // gradecoin-site (zola) outputs a public/, we serve it here
    let static_route = warp::any().and(warp::fs::dir("public"));

    let api = routes.or(static_route);

    // Start the server
    let point = ([127, 0, 0, 1], 8080);
    warp::serve(api).run(point).await;
}

lazy_static! {
    static ref PRIVATE_KEY: String =
        fs::read_to_string("secrets/gradecoin.pem").expect("error reading 'secrets/gradecoin.pem'");
}
