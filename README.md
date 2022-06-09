# Rudy
This is my attempt at a command-line-based HTTP proxy. Written in Rust of course.

## TLS
My first goal is to get this to work for TLS connections similar to how Burp or Zap work. The `cert/` directory contains a script to generate a CA and server certificate. Your browser will have to trust this CA cert in order for TLS requests to be established.

## Goals
- Allow interception and modification of HTTP requests
- Keep a request/response history
- Make a slick terminal UI using [TUI](https://docs.rs/tui/0.18.0/tui/)
