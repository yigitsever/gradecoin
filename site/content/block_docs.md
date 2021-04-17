+++
title = "Blocks"
description = "Block Documentation"
weight = 10
+++

A block that was proposed to commit Transactions in `transaction_list` to the
ledger with a nonce that made `hash` valid; 6 zeroes at the left hand side of the
hash (24 bytes).

We are _mining_ using [blake2s](https://www.blake2.net/) algorithm, which produces 256 bit hashes. Hash/second is roughly {{ exp(num="20x10", exponent="3") }} on my machine, a new block can be mined in around 4-6 minutes.

# Requests

## GET
A HTTP `GET` request to [/block](/block) endpoint will return the latest mined block.

## POST

A HTTP `POST` request with Authorization using JWT will allow you to propose your own blocks.

# Fields
```
transaction_list: [array of Fingerprints]
nonce: unsigned 32-bit integer
timestamp: ISO 8601 <date>T<time>
hash: String
```

# Mining
The _mining_ process for the hash involves;
- Creating a temporary JSON object with `transaction_list`, `timestamp` and `nonce` values
- Serializing it
- Calculating blake2s hash of the serialized string

If the resulting hash is valid, then you can create a `Block` JSON object with the found `nonce` and `hash`.

# Hash

```tha``` field in [jwt documentation](/jwt) in fact stands for "The Hash", in the case of a post request for a block, you need to use hash field of the block.


[ISO 8601 Reference](https://en.wikipedia.org/wiki/ISO_8601#Combined_date_and_time_representations)
