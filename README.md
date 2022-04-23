# Gradecoin

This is designed to sit behind nginx reverse proxy so running at 127.0.0.1:8080 or not using https is not a problem.

It's currently live over at https://gradecoin.xyz.

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
```sh
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


# References
- https://github.com/blurbyte/restful-rust
- https://github.com/zupzup/warp-postgres-example
- https://blog.logrocket.com/create-an-async-crud-web-service-in-rust-with-warp/

# How to be a good server
- https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml
- https://tools.ietf.org/html/rfc7231
