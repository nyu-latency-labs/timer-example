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
struct HelloServer{
    counter: Arc<Mutex<i32>>
}

impl service::Service for HelloServer {
    type HelloFut = Ready<i32>;

    fn hello(self, _: context::Context) -> Self::HelloFut {
        let mut data = self.counter.lock().unwrap();
        *data += 1;
        future::ready(*data)
    }
}

async fn produce_many_events(d: Duration) -> io::Result<()> {
    loop {
        Delay::new(d)
            .map(|_| println!("{:?}", d))
            .await;
    }
}

async fn run(server_addr: SocketAddr) -> io::Result<()> {
    let t = produce_many_events(Duration::from_secs(1));
    println!("Going to listen on {:?}", server_addr);
    let transport = bincode_transport::listen(&server_addr)?
        .filter_map(|r| future::ready(r.ok()));
    let server = Server::default()
        .incoming(transport)
        .respond_with(service::serve(HelloServer {
            counter: Arc::new(Mutex::new(0))}));
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
