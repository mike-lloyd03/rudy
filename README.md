# Rudy
This is my attempt at a command-line-based HTTP proxy. Written in Rust of course.

## TLS
My first goal is to get this to work for TLS connections similar to how Burp or Zap work. The `cert/` directory contains a script to generate a CA and server certificate. Your browser will have to trust this CA cert in order for TLS requests to be established.

## Goals
- Allow interception and modification of HTTP requests
- Keep a request/response history
- Make a slick terminal UI using [TUI](https://docs.rs/tui/0.18.0/tui/)

## Usage
Run the `gen_ca` script in the `cert` directory to create a new Certificate Authority (CA). This CA will be used to sign all X509 certificates for HTTPS requests. You will have to direct your system or your browser to trust this CA in order for the proxy to work without getting TLS warnings.

Once done, simply run `rudy` and configure your browser to use `localhost:8080` as the proxy server. All requests will be shown in the terminal. For now, this tool only runs in a read-only mode but eventually you will be able to intercept and modify requests.
