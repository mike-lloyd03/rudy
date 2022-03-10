# Rust in the Middle
This is my attempt at a command-line-based HTTP proxy. Written in Rust of course.

Based upon [rust-http-proxy](https://github.com/linmx0130/rust-http-proxy).

## TLS
My first goal is to get this to work for TLS connections similar to how Burp or Zap work. The
`cert/` directory contains a script to generate a CA and server certificate. You can tell Firefox
to trust this CA but the server cert will only work for sites which are specified in the server
cert's DNS entries.

To fix this, the program will have to generate and sign certs for every site it intercepts.
Ideally, there would be a cache to speed up repeated requests to the same domain.

## Goals
- Allow interception and modification of HTTP requests
- Keep a request/response history
- Make a slick terminal UI using [Crossterm](https://github.com/crossterm-rs/crossterm)
