# Gradecoin
[![ci](https://github.com/yigitsever/gradecoin/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/yigitsever/gradecoin/actions/workflows/rust.yml)

Gradecoin is a undergraduate level cryptography simulation/game that masquerades as a cryptocurrency.

The project is live at https://gradecoin.xyz.

```sh
# Test the project
$ cargo test

# Read the documentation
$ cargo doc --open
```

The executable `main` needs the `/templates`, `users` and `blocks` folders. It also expects a `secrets/gradecoin.pem` file with the private key of the system.


# Running Locally

Create RSA keys:
```sh
$ mkdir secrets
$ cd secrets
$ openssl genrsa -out gradecoin.pem 2048
$ openssl rsa -in gradecoin.pem -outform PEM -pubout -out gradecoin.pub
```
Use `gradecoin.pub` file in your client program.

Create students list: `students.csv` should be in the following form:
```
User ID, Password
e123456,register_password
e123456,register_password
```
First line is ignored.

Run the server:
```sh
$ cargo run
```

The server should be up on `localhost:8080`.
We recommend using Nginx to reverse proxy Gradecoin so it can be served as HTTPS.

The default config file is `config.yaml`.
You can specify another config file with:
```sh
$ cargo run another_config.yaml
```

You can even run multiple networks at once by giving multiple configuration files:
```sh
# Run both the main network (at /) and testnet (at /testnet)
# For example, register for main network at `localhost:8080/register`,
# testnet network at `localhost:8080/testnet/register`
$ cargo run config.yaml testnet.yaml
```
In the config file, `url_prefix` field determines the address of the network.

You can clear the database for all networks by running:
```sh
$ rm -rf blocks users
```
Or you can delete the database for a particular network by removing `blocks/network_name` and `users/network_name`.
