# Gradecoin

This will sit behind nginx reverse proxy so running at 127.0.0.1:8080 is no problem, or https.

```
$ cargo run

$ curl --location --request POST 'localhost:8080/transaction' --header 'Content-Type: application/json' --data-raw '{
  "source": "Myself Truly",
  "target": "Literally Anybody Else",
  "amount": 12,
  "timestamp": "2021-04-07T00:17:00"
}'
```

# how?

## authentication
Students generate their own `keypairs` and authenticate with their METU Student IDs.
Authenticated students propose transactions, between them and another node (=public keys) or between the grader (=bank) and themselves.

## transactions
Transactions are `signed` using the proposers private key.
(This whole public/private key + signing process will require some crypto dependency, **todo**)

## blocks
Blocks are proposed using `N` transactions, this can be an exact number (=20) or if the last block is *some time* old then small blocks can be proposed.
Block proposal: `Block` + some `nonce` is hashed using a *simple* hash function, resulting hash should have some property that will require some computation time (~1 minute? 10 minutes?) to find (=guessing) Proof-of-work scheme.
First proposed valid block is accepted, if assertions hold.
(No consensus, we are the sole authority, there's no blockchain here, only a glorified database and busywork)

## payment
First transaction in the block is called *Coinbase*, the block reward is paid to the *output* (Bitcoin notation, different) of this transaction.
If we do this then the rest of the transactions are just make believe playing.
So banker + block reward approach seems better.

## then
After the new block, stale transactions are cleared?

# Big Thank List
- https://github.com/blurbyte/restful-rust
- https://github.com/zupzup/warp-postgres-example
- https://blog.logrocket.com/create-an-async-crud-web-service-in-rust-with-warp/

# How to be a good server
- https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml
- https://tools.ietf.org/html/rfc7231
