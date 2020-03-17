# actix-derive [![Build Status](https://travis-ci.org/actix/actix-derive.svg?branch=master)](https://travis-ci.org/actix/actix-derive) [![crates.io](https://img.shields.io/crates/v/actix-derive)](https://crates.io/crates/actix-derive)

Actix is a rust actor framework.

* [API Documentation (Development)](https://actix.github.io/actix/actix/)
* [API Documentation (Releases)](https://docs.rs/actix/)
* Cargo package: [actix](https://crates.io/crates/actix)

---

## Features

* `actix-derive` adds support for Rust Custom Derive / Macros 1.1 to `actix`

## Usage

```rust
use actix_derive::{Message, MessageResponse};

#[derive(MessageResponse)]
struct Added(usize);

#[derive(Message)]
#[rtype(result = "Added")]
struct Sum(usize, usize);

fn main() {}
```

This code expands into following code:

```rust
use actix::{Actor, Context, Handler, System};
use actix_derive::{Message, MessageResponse};

#[derive(MessageResponse)]
struct Added(usize);

#[derive(Message)]
#[rtype(result = "Added")]
struct Sum(usize, usize);

#[derive(Default)]
struct Adder;

impl Actor for Adder {
    type Context = Context<Self>;
}

impl Handler<Sum> for Adder {
    type Result = <Sum as actix::Message>::Result;
    fn handle(&mut self, msg: Sum, _: &mut Self::Context) -> Added {
        Added(msg.0 + msg.1)
    }
}

fn main() {}
```

## License

This project is licensed under either of

  * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
    https://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or
    https://opensource.org/licenses/MIT)

at your option.
