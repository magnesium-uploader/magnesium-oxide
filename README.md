# Magnesium Oxide

![GitHub release (latest by date)](https://img.shields.io/github/v/release/ChecksumDev/magnesium-oxide?label=Release) [![Build and Deploy](https://github.com/ChecksumDev/magnesium-oxide/actions/workflows/rust.yml/badge.svg)](https://github.com/ChecksumDev/magnesium-oxide/actions/workflows/rust.yml) ![Discord](https://img.shields.io/discord/984852897051312159?label=Discord&logo=DISCORD) ![coffee](https://img.shields.io/badge/Made%20with-Coffee-a27250?logo=CoffeeScript)

## ❔ What is this?

Magnesium-Oxide (MGO) is a secure file uploader for ShareX.

## 🌠 Features

* 🔥 Blazingly fast uploads and encryption.
* 💾 All files are encrypted with a random, secure key, and the key is never saved on the database.
* 🔒 Encryption on all files uploaded using [AES256-GCM-SIV](https://eprint.iacr.org/2017/168.pdf).
* 🦄 All functions are documented, and all code is written in Rust, no external linkages!
* ✨ Completely memory-safe, no need to worry about memory leaks using a global **`#![forbid(unsafe_code)]`** in [`src/main.rs`](https://github.com/magnesium-uploader/magnesium-oxide/blob/main/src/main.rs#L5).

## 🌌 Roadmap

Think of any features you'd like to see in the future? Let us know by opening an issue or creating a pull request!

* [ ] 📦 Compressed uploads
* [ ] 📦 Upload encrypted files to S3
* [ ] 💀 Zero-width-encoding for file names
* [ ] 🪢 Support for other databases other than MongoDB (e.g. PostgreSQL)
* [ ] ☢️ Support for other ShareX like software

## ➕ Contributing

Contributions, issues, and feature requests are welcome,

Ensure you read [CONTRIBUTING](CONTRIBUTING.md) before submitting a pull request.

## 🤝 Support

**Don't hesitate to give us a ⭐️ if you like what you see, it motivates us to keep working hard on it!**
