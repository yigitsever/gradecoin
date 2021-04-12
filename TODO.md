# TODO

## Proof-of-work

## Authentication
- [X] pick a user authentication scheme = [JWT](https://tools.ietf.org/html/rfc7519) Seems perfect
- [ ] implement JWT
    - https://blog.logrocket.com/jwt-authentication-in-rust/
    - https://crates.io/crates/jsonwebtoken
    - https://jwt.io/introduction/
    - https://jwt.io/#debugger-io
- [ ] users should be able to _sign_ their transactions

## Verbosity
- [ ] Verbose error messages (use error.rs?)

## Tests
- [ ] Schema Tests
- [ ] Route Tests
    - [ ] Malformed JSON bodies
    - [ ] Valid JSON with missing fields
    - [ ] Valid JSON with extra fields

## Done & Brag
- [x] Switch to RwLock (parking_lot) (done at 2021-04-07 03:43, two possible schemes to represent inner Db (ledger) in code)
- [x] We need our own representation of students and their grades, "there is no blockchain" (done at 2021-04-12 00:05)
- [x] pick a block proposal scheme (= pick hash function) [list of hash functions](https://en.bitcoinwiki.org/wiki/List_of_hash_functions) (done at 2021-04-12 05:30)
- [x] check the nonce for incoming blocks (done at 2021-04-12 05:30)
