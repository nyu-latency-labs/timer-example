#![feature(arbitrary_self_types)]
extern crate time;

use futures::{
    prelude::*,
    compat::Executor01CompatExt,
};
use std::{io, net::SocketAddr};
use tarpc::{client, context};
use time::{PreciseTime};
use std::env;


async fn run(server_addr: SocketAddr) -> io::Result<()> {
    let transport = bincode_transport::connect(&server_addr).await?;
    let mut client = service::new_stub(client::Config::default(), transport).await?;

    for _ in 1..1000 {
        let start = PreciseTime::now();
        let hello = client.hello(context::current()).await?;
        let end = PreciseTime::now();

        let runtime = start.to(end).num_nanoseconds().expect("-");

        println!("{}\t{}", hello, runtime);
    }

    Ok(())
}

fn main() {
    let args:Vec<_> = env::args().collect();
    let server_addr = &args[1];
    let server_addr = server_addr
        .parse()
        .unwrap_or_else(|e| panic!(r#"--server_addr value "{}" invalid: {}"#, server_addr, e));
    println!("Going to connect on {}", server_addr);

    tarpc::init(tokio::executor::DefaultExecutor::current().compat());

    tokio::run(
        run(server_addr)
            .map_err(|e| eprintln!("Oh no: {}", e))
            .boxed()
            .compat(),
    );
}

