+++
title = "Register"
description = "Register Documentation"
weight = 3
+++

POST request to /register endpoint
Lets a [`User`] (=student) to authenticate themselves to the system
This `request` can be rejected if the payload is malformed (=not authenticated properly) or if
the [`AuthRequest.user_id`] of the `request` is not in the list of users that can hold a Gradecoin account

# Authentication Process
- Gradecoin's Public Key (`gradecoin_public_key`) is listed on moodle.
- Gradecoin's Private Key (`gradecoin_private_key`) is loaded here

- Student picks a short temporary key (`k_temp`)
- Creates a JSON object (`auth_plaintext`) with their `metu_id` and `public key` in base64 (PEM) format (`S_PK`):
{
    student_id: "e12345",
    passwd: "15 char secret"
    public_key: "---BEGIN PUBLIC KEY..."
}

- Encrypts the serialized string of `auth_plaintext` with 128 bit block AES in CBC mode with Pkcs7 padding using the temporary key (`k_temp`), the result is `auth_ciphertext` TODO should this be base64'd?
- The temporary key student has picked `k_temp` is encrypted using RSA with OAEP padding scheme
using sha256 with `gradecoin_public_key` (TODO base64? same as above), giving us `key_ciphertext`
- The payload JSON object (`auth_request`) can be JSON serialized now:
{
    c: "auth_ciphertext"
    key: "key_ciphertext"
}

## Gradecoin Side

- Upon receiving, we first RSA decrypt with OAEP padding scheme using SHA256 with `gradecoin_private_key` as the key and auth_request.key `key` as the ciphertext, receiving `temp_key` (this is the temporary key chosen by stu
- With `temp_key`, we can AES 128 Cbc Pkcs7 decrypt the `auth_request.c`, giving us
auth_plaintext
- The `auth_plaintext` String can be deserialized to [`AuthRequest`]
- We then verify the payload and calculate the User fingerprint
- Finally, create the new [`User`] object, insert to users HashMap `<fingerprint, User>`


