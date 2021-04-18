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

`tha` field in [jwt documentation](@/JWT.md) in fact stands for "The Hash", in the case of a post request for a transaction, you need the Md5 hash of the serialized JSON representation of transaction. The resulting JSON string should look something like;

```
{"by":"foo","source":"bar","target":"baz","amount":2,"timestamp":"2021-04-18T21:49:00"}
```

Or; without any whitespace, separated with `:` and `,`.

# Bank

There is a `bank` account with Fingerprint `31415926535897932384626433832795028841971693993751058209749445923`

{% tidbit() %}
First 64 digits of Pi
{% end %}

This is the only account that will let you _withdraw_ from them.

```
by: this has to be your Fingerprint
source: this can be either you or the bank
target: this can be a valid fingerprint or yourself if source is the bank
...
```
