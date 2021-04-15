+++
title = "Register"
description = "Register Documentation"
weight = 3
+++

POST request to /register endpoint

Lets a user to authenticate themselves to the system.
Only people who are enrolled to the class can open Gradecoin accounts.
This is enforced with your Student ID and a one time password you will receive.

# Authentication Process
- Gradecoin's Public Key (`gradecoin_public_key`) is listed on our Moodle page.
- You pick a short temporary key (`k_temp`)
- Create a JSON object (`auth_plaintext`) with your `metu_id` and `public key` in base64 (PEM) format (`S_PK`) [reference](https://tls.mbed.org/kb/cryptography/asn1-key-structures-in-der-and-pem)
```json
{
    "student_id": "e12345",
    "passwd": "15 char secret",
    "public_key": "---BEGIN PUBLIC KEY..."
}
```

- Pick a random IV.
- Encrypt the serialized string of `auth_plaintext` with 128 bit block AES in CBC mode with Pkcs7 padding using the temporary key (`k_temp`), the result is `auth_ciphertext`. Encode this with base64.
- The temporary key you have picked `k_temp` is encrypted using RSA with OAEP padding scheme
using SHA-256 with `gradecoin_public_key`, giving us `key_ciphertext`. Encode this with base 64.
- The payload JSON object (`auth_request`) can be serialized now:

```json
{
    "c": "auth_ciphertext",
    "iv": "hexadecimal",
    "key": "key_ciphertext"
}
```

If your authentication process was valid, you will be given access and your public key fingerprint that is your address.
