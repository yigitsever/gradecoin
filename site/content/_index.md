+++
title = "Gradecoin"
sort_by = "weight"
+++

- Don't know where to start? Gradecoin uses RESTful API, simple `curl` commands or even your browser will work! [This website can help as well](https://curl.trillworks.com/).
- [JWT Debugger](https://jwt.io) and the corresponding [RFC](https://tools.ietf.org/html/rfc7519)

# Services
## /register
- Student creates their own 2048 bit RSA `keypair`
- Downloads `Gradecoin`'s Public Key from [Moodle](https://odtuclass.metu.edu.tr/my/)
- Encrypts their JSON wrapped `Public Key`, `Student ID` and one time `passwd` using Gradecoin's Public Key
- Their public key is now in our database and can be used to sign their JWT's during requests

## /transaction
- You can offer a [Transaction](/transaction) - POST request
    - The request should have `Authorization`
    - The request header should be signed by the Public Key of the `by` field in the transaction
- fetch the list of `Transaction`s - GET request

## /block
- offer a [`schema::Block`] - POST request
    - The request should have `Authorization`
    - The [`schema::Block::transaction_list`] of the block should be a subset of [`schema::Db::pending_transactions`]
- fetch the last accepted [`schema::Block`] - GET request

`Authorization`: The request header should have Bearer JWT.Token signed with Student Public Key
