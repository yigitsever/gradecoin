# Drocoin

This is designed to sit behind nginx reverse proxy so running at 127.0.0.1:8080 or not using https is not a problem.

It's currently live over at https://gradecoin.xyz. It will be moving to https://drocoin.xyz soon.

```
# Test the project
$ cargo test

# Read the documentation
$ cargo doc --open
```

The executable `main` needs the `/templates`, `users` and `blocks` folders. It also expects a `secrets/drocoin.pem` file with the private key of the system.

# References
- https://github.com/blurbyte/restful-rust
- https://github.com/zupzup/warp-postgres-example
- https://blog.logrocket.com/create-an-async-crud-web-service-in-rust-with-warp/

# How to be a good server
- https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml
- https://tools.ietf.org/html/rfc7231


