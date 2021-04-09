# TODO
## Process
- [ ] we need our own representation of students and their grades, "there is no blockchain"

## Proof-of-work
- [ ] pick a block proposal scheme (= pick hash function) [list of hash functions](https://en.bitcoinwiki.org/wiki/List_of_hash_functions)
- [ ] check the nonce for incoming blocks

## Authentication
- [X] pick a user authentication scheme = [JWT](https://tools.ietf.org/html/rfc7519) Seems perfect
- [ ] implement JWT
    - https://blog.logrocket.com/jwt-authentication-in-rust/
    - https://crates.io/crates/jsonwebtoken
    - https://jwt.io/introduction/
    - https://jwt.io/#debugger-io
- [ ] users should be able to _sign_ their transactions

## Done & Brag
- [x] Switch to RwLock (parking_lot) (done at 2021-04-07 03:43, two possible schemes to represent inner Db (ledger) in code)
