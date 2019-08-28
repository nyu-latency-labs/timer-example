use futures::{
    join,
    future::{self, Ready},
    compat::Executor01CompatExt,
    prelude::*,
};
use std::{io, 
    net::SocketAddr,
    time::Duration
};
use futures_timer::Delay;
use tarpc::{
    context,
    server::{Handler, Server},
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Data {
    counter: i32,
}

#[derive(Clone)]
struct HelloServer{
    data: Arc<Mutex<Data>>,
}

impl service::Service for HelloServer {
    type HelloFut = Ready<i32>;

    fn hello(self, _: context::Context) -> Self::HelloFut {
        let mut data = self.data.lock().unwrap();
        data.counter += 1;
        future::ready(data.counter)
    }
}

async fn produce_many_events(d: Duration, dat: Arc<Mutex<Data>>) -> io::Result<()> {
    loop {
        Delay::new(d)
            .map(|_| {
                let c = dat.lock().unwrap();
                println!("{:?} {:?}", d, c.counter);
            })
            .await;
    }
}

async fn run(server_addr: SocketAddr) -> io::Result<()> {
    let d = Arc::new(Mutex::new(Data{ counter: 0}));
    let t = produce_many_events(Duration::from_secs(1), d.clone());
    println!("Going to listen on {:?}", server_addr);
    let transport = bincode_transport::listen(&server_addr)?
        .filter_map(|r| future::ready(r.ok()));
    let server = Server::default()
        .incoming(transport)
        .respond_with(service::serve(HelloServer {data: d.clone()}));
    let _ = join!(t, server);
    //combo.await;
    Ok(())
}

fn main() {
    let port = "8900";
    let port = port
        .parse()
        .unwrap_or_else(|e| panic!(r#"--port value "{}" invalid: {}"#, port, e));

    tarpc::init(tokio::executor::DefaultExecutor::current().compat());

    tokio::run(
        run(([0, 0, 0, 0], port).into())
            .map_err(|e| eprintln!("Oh no: {}", e))
            .boxed()
            .compat(),
    );
}
