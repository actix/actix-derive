# actix-derive

[![crates.io](http://meritbadge.herokuapp.com/actix-derive)](https://crates.io/crates/actix-derive)

Actix is a rust actor system framework.

* [API Documentation (Development)](http://actix.github.io/actix/actix/)
* [API Documentation (Releases)](https://docs.rs/actix/)
* Cargo package: [actix](https://crates.io/crates/actix)

---

Actix is licensed under the [Apache-2.0 license](http://opensource.org/licenses/APACHE-2.0).

## Features

* `actix-derive` adds support for Rust Custom Derive / Macros 1.1 to `actix`

## Usage

```rust
extern crate actix;
#[macro_use] extern crate actix_derive;

use actix::ResponseType;

#[derive(Message)]
#[Message(usize)]
struct Sum(usize, usize);
```