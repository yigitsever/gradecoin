+++
title = "Transactions"
description = "Transaction documentation"
weight = 6
+++

A transaction request between `source` and `target` to move `amount` Gradecoin.

# Requests

## GET
A HTTP `GET` request to [/transaction](/transaction) endpoint will return the current list of pending transactions.

## POST

A HTTP `POST` request with Authorization using JWT to [/transaction](/transactions) will allow you to propose your own transactions.


# Fields
```
by: Fingerprint
source: Fingerprint
target: Fingerprint
amount: unsigned 16 bit integer
timestamp: ISO 8601 <date>T<time>
```

# Hash

```tha``` field in [jwt documentation](/jwt) in fact stands for "The Hash", in the case of a post request for a transaction, you need the hash of the serialized JSON representation of transaction. 
