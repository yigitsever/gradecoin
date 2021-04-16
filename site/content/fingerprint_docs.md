+++
title = "Fingerprint"
description = "Fingerprint documentation"
weight = 6
+++

# Definition

A fingerprint is a 256 bit 64 character hexadecimal user idenfitier for users. Fingerprints are used in defining users in [transactions](@/transaction_docs.md) and [blocks](@block_docs.md).

# Fingerprint Generation

A user's finger print is generated via applying SHA256 sum of the user's public RSA key.