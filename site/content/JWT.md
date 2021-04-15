+++
title = "JWT"
description = "JSON Web Token Documentation"
weight = 4
+++

> JSON Web Tokens are representations of claims, or authorization proofs that fit into the `Header` of HTTP requests.

# How?

JWTs are used as the [MAC](https://en.wikipedia.org/wiki/Message_authentication_code) of operations that require authorization:
- block proposal
- transaction proposal.

They are send alongside the JSON request body in the `Header`;

```html
Authorization: Bearer aaaaaa.bbbbbb.ccccc
```

Gradecoin uses 3 fields for the JWTs;

```json
{
"tha": "Hash of the payload, check invididual references",
"iat": "Issued At, Unix Time",
"exp": "Expiration Time, epoch"
}
```

- `tha` is explained in [blocks](@/block_docs.md) and [transactions](@/transaction_docs.md) documentations.
- `iat` when the JWT was created in [Unix Time](https://en.wikipedia.org/wiki/Unix_time) format
- `exp` when the JWT will expire & be rejected in [Unix Time](https://en.wikipedia.org/wiki/Unix_time)

# Algorithm
We are using [RS256](https://www.rfc-editor.org/rfc/rfc7518.html#section-3.1), `RSASSA-PKCS1-v1_5 using SHA-256`. The JWTs you encode with your private RSA key will be decoded using the public key you have authenticated with. You can see how the process works [here](https://jwt.io/).

# References
- [RFC, the ultimate reference](https://tools.ietf.org/html/rfc7519)
- [JWT Debugger](https://jwt.io/)

