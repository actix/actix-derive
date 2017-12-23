#![cfg_attr(actix_nightly, feature(proc_macro,))]

extern crate actix;
extern crate futures;
#[macro_use] extern crate actix_derive;

use actix_derive::actor;

use actix::{msgs, Actor, Address, Arbiter, System};
use futures::{future, Future};

#[derive(Message)]
#[rtype(usize)]
struct Sum{a: usize, b: usize}

#[derive(Message)]
#[rtype(usize)]
struct Sum1{a: usize, b: usize}

struct SumActor;

#[cfg(actix_nightly)]
#[actor(Context<_>)]
impl SumActor {

    #[simple(Sum1)]
    fn sum1(&mut self, a: usize, b: usize) -> usize {
        a + b
    }

    #[handler(Sum)]
    fn sum(&mut self, a: usize, b: usize) -> Result<usize, ()> {
        Ok(a + b)
    }
}

#[cfg(actix_nightly)]
#[test]
fn test_handlers() {
    let system = System::new("test");
    let addr: Address<_> = SumActor.start();

    system.handle().spawn(addr.call_fut(Sum{a: 10, b: 5}).then(|res| {
        match res {
            Ok(Ok(result)) => assert!(result == 10 + 5),
            _ => panic!("Something went wrong"),
        }
        future::result(Ok(()))
    }));

    system.handle().spawn(addr.call_fut(Sum1{a: 10, b: 5}).then(|res| {
        match res {
            Ok(Ok(result)) => assert!(result == 10 + 5),
            _ => panic!("Something went wrong"),
        }

        Arbiter::system().send(msgs::SystemExit(0));
        future::result(Ok(()))
    }));

    system.run();
}
