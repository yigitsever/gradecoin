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
> Uses /register endpoint
- Student creates their own 2048 bit RSA `keypair`
- Downloads Gradecoin's Public Key from Moodle
- Encrypts their JSON wrapped Public Key and Student ID using Gradecoin's Public Key
- Their public key is now in our database and can be used to sign their JWT's during requests

## transactions
> Uses /transaction endpoint
- offer **a transaction** - POST request
    - The request header should have Bearer [JWT.Token signed with Student Public Key]
    - The request header should be signed by the Public Key of the `by` field in the transaction
- fetch the list of pending transactions - GET request
    - All the pending transactions are returned in a JSON body
    - ❓ Does this need to be authenticated as well?

## blocks - [INCOMPLETE]
> Uses /block endpoint
- Blocks are proposed using `N` transactions - POST request
    - ❓ This can be an exact number (=20) or if the last block is *some time* old then small blocks can be proposed.

- Block proposal: `Block` + some `nonce` is hashed using a *simple* hash function, resulting hash should have some property that will require some computation time (~1 minute? 10 minutes?) to find (=guessing) Proof-of-work scheme.
First proposed valid block is accepted, if assertions hold.
(No consensus, we are the sole authority, there's no blockchain here, only a glorified database and busywork)
- Pending transactions get cleared out after a new block is accepted
    - ❓ All or only the used ones?

## payment
First transaction in the block is called *Coinbase*, the block reward is paid to the *output* (Bitcoin notation, different) of this transaction.
If we do this then the rest of the transactions are just make believe playing.
So banker + block reward approach seems better.

# Big Thank List
- https://github.com/blurbyte/restful-rust
- https://github.com/zupzup/warp-postgres-example
- https://blog.logrocket.com/create-an-async-crud-web-service-in-rust-with-warp/

# How to be a good server
- https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml
- https://tools.ietf.org/html/rfc7231
