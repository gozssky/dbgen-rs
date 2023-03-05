use anyhow::Result;
use dbgen::{
    cli::{run, Args},
    span::Registry,
};
use futures::future;
use hyper::server::Server;
use hyper::service::make_service_fn;
use std::net::TcpListener;
use std::process::exit;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let mut registry = Registry::default();
    let args = Args::from_args();
    let listen_addr = args.s3_listen_addr.clone();

    let s3_service = match run(args, &mut registry) {
        Ok(s3_service) => s3_service,
        Err(e) => {
            eprintln!("{}", registry.describe(&e));
            exit(1);
        }
    };
    if s3_service.is_none() {
        return Ok(());
    }
    let s3_service = s3_service.unwrap();

    let server = {
        let service = s3_service.into_shared();
        let listener = TcpListener::bind(listen_addr.clone())?;
        let make_service: _ = make_service_fn(move |_| future::ready(Ok::<_, anyhow::Error>(service.clone())));
        Server::from_tcp(listener)?.serve(make_service)
    };

    println!("S3 server is running at http://{}/", listen_addr);
    server.await?;

    Ok(())
}
