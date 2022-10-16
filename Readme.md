# Rust Embed Responder for Actix Web

An Actix Web responder for serving files embedded with [rust embed](https://docs.rs/rust-embed/latest/rust_embed/index.html).

Using this crate, you can use files embedded with `rust_embed` as a response
from your handlers. This crate handles `Last-Modified` and `ETag` headers, and
responds to `If-None-Match` and `If-Modified-Since` conditional requests
appropriately. This skips sending the files again if the client has already
cached them (such as a browser).

This crate can also perform on-the-fly compression for embedded files if the
client requesting supports compression.

## Option: rust-embed-for-web

This crate also supports an alternative rust-embed fork [rust-embed-for-web](https://github.com/SeriousBug/rust-embed-for-web).

This rust-embed fork pre-computes `ETag` and `Last-Modified` headers, and
pre-compresses the files for compressed responses. This comes at the cost of a
larger executable size, but improves performance massively.
