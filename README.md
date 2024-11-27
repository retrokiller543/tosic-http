# Tosic-HTTP

A powerful and simple HTTP server implementation that relies on [tower](https://github.com/tower-rs/tower)
to handle middleware.
The main inspiration for this crate is [actix-web](https://github.com/actix/actix-web)
and also made to learn more about creating robust and developer friendly code in Rust.

## Important notes

- This crate is not production ready!
- Currently, this crate will only run on the `nightly` version of Rust since it relies on some experimental features. I'm currently working on removing this requirement, but I can't guarantee that it will be done
- Breaking changes may happen at anytime since this is still far from a production ready crate

## Examples

See [examples](https://github.com/retrokiller543/tosic-http/tree/master/examples) folder
for a few examples that were used during development of this crate. If there are any
questions that are not answered in the examples or on the [documentation page](https://docs.rs/crate/tosic-http/),
feel free to submit an issue to GitHub.

## Installation

To install this crate run
`cargo add tosic-http`
or add it to you `Cargo.toml`

```toml
[dependencies]
tosic-http = "0.0.*"
```