//! # Gradecoin
//!
//! ## Services
//! ### /register
//! - Student creates their own 2048 bit RSA `keypair`
//! - Downloads `Gradecoin`'s Public Key from Moodle
//! - Encrypts their JSON wrapped `Public Key` and `Student ID` using Gradecoin's Public Key
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

pub const PRIVATE_KEY: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAyGuqiCPGcguy+Y9TH7Bl7XlEsalyqb9bYlzpbV0dnqZ3lPkE
PkuOhkN+GcuiV6iXtSwyh7nB+xTRXKJFRUBO/jbN8jfcxVwBu0JxjF3v1YRBxbOH
hz2A295mbKD9xHQCKxkfYBNkUXxj8gd+GaDvQiSW5NdrX/lEkvqfGtdEX1m2+Hdc
G0+3YW24Xg0znhCwLr+sorLuJaDy9Xa0Uo+DPWGC5s001U/BxkCIWJ+eJQCb7Bv+
9vXb8BGRK/ecMb/fb6h5O+8fgB64RCHMgcc2v+Q/dPt8kHX1OJdMuYUrUJGACppM
QY3W6e1HdlRIBcZKL2LMZ2CrIB/2D5LiJhPThQIDAQABAoIBABbHrg1lS5QA4mnd
MYyDh0JTq0wqP18t4dwvRVTp5Yj30NW87A+MlPmLyFR0QdKG1h+Ak4m7wmGgfx9x
TkBNy+y3G/dxBAXmrEe1iKR0tOLm8nbfLgNgKTpUb/3e2pkuumRdqaRI7/kXE2Ea
Guoc0bUJ5aDDH3A8K+As3lK1rw7LNxwxZdmqmpO+EAldP6NaLnXNP5BegjLK50xP
NXTDNx6pw+I2ZHHwC/A6+QVksSA6zPipI1poANaO0frHffwKhcEZ/VucuXlJGGq/
aqXT/cc7IkKUVq8EZUwUqHi4SrnyDDq/mtuikSD0MazxumbeC6fBKRP98Kavy2rT
JItHSYECgYEA8H/yC9GDrR1bwBesD0pKdKBy18UMFQF3BrB04OjqdGzugdVafF4e
7azYQQTQ0ZddLDvgYl0QYvQaZfv26L7o4VrN5XEg8WjUWKuww8XUYOCfPn4gOFL1
ar8nQ0w3P65gYf/rw0rFMo3eB78rJMROYnG8nZ/3OdgQjVaYPJxFKmECgYEA1VZy
EQz8dHK3+F0EfQIFeXOSlYGUegmPZ9iYmh+yvW/zWKLYdXBEHNhAIRlBmfe7Yhj6
1FNluNGjFqZYuRnP0RuiBxt2RCd+AL90Lqq+O6jem4XNgr3cOKoaV0FbaU49sI4s
/B6iiYBFdVuPBiknz+Wf1KEF9lQ+w2VYSLucY6UCgYAWPe73ste3sehjWo0aGOfL
427bj6ivZKRKZRVaG5BbVhu0vDOTHu1DU+HoGXbqe1ItnhgBYNP8ItEyL1xFaCqH
dOtn1c+TI/vHe5FseaZLk1qG4AlAzENQLP+HlMvjQtA9H/sA47BbHY20L7TgwJrz
NcuY1Et7+QSG3cRUjqtC4QKBgGuP+VUVehfwW0dzBrdMlJwGpGqS+dyKA271awOS
ZdlTn5saCA82OnFcqwDFLilGGYk9VQJGxivoLtVVq7gwBnLE/u2ccAWu773KyfZZ
ii6kVxCM5vA7b9R2F2/U+RTgKQRiutWnUIYJUXv5XORbTcJpYSugwFPRaA+2gkux
pAktAoGABRyVs5LOhQ/oeXe2H2kvuaUq9c7f/dTtnyMNdNxK0uZcQn4jcB2eK9kB
PDYHM9dfQ8xn51U0fTeaXjy/8Km8fyX2Jtxntlm6puyhSTJ8AX+FEgJkC4ajNEvA
mJ1Gsy2fXKUyyZdI2b74MLqOpzr9cvS60tmTIScuiHFzg/SJgiA=
-----END RSA PRIVATE KEY-----";

pub const PUB_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyGuqiCPGcguy+Y9TH7Bl
7XlEsalyqb9bYlzpbV0dnqZ3lPkEPkuOhkN+GcuiV6iXtSwyh7nB+xTRXKJFRUBO
/jbN8jfcxVwBu0JxjF3v1YRBxbOHhz2A295mbKD9xHQCKxkfYBNkUXxj8gd+GaDv
QiSW5NdrX/lEkvqfGtdEX1m2+HdcG0+3YW24Xg0znhCwLr+sorLuJaDy9Xa0Uo+D
PWGC5s001U/BxkCIWJ+eJQCb7Bv+9vXb8BGRK/ecMb/fb6h5O+8fgB64RCHMgcc2
v+Q/dPt8kHX1OJdMuYUrUJGACppMQY3W6e1HdlRIBcZKL2LMZ2CrIB/2D5LiJhPT
hQIDAQAB
-----END PUBLIC KEY-----";
