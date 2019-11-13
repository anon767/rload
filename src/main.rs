use std::fs::File;
use std::env;

use futures::future::{self, Future};
use hyper::{Body, Request, Response, Server};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Root {
    schema: String,
    version: String,
    debug: bool,
    listen: String,
    routes: Vec<Route>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Route {
    catch: String,
    nodes: Vec<String>,
}


type BoxFut = Box<dyn Future<Item=Response<Body>, Error=hyper::Error> + Send>;

fn debug_request(enabled: bool, req: Request<Body>) -> BoxFut {
    match enabled {
        true => {
            let body_str = format!("{:?}", req);
            let response = Response::new(Body::from(body_str));
            Box::new(future::ok(response))
        }
        false => Box::new(future::ok(Response::new(Body::from(""))))
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = match args.get(1) {
        Some(f) => f,
        None => panic!("please provide a config file as first parameter")
    };

    println!("reading config {}", config_path);
    let file_handle = std::fs::File::open(config_path);
    let f: File = match file_handle {
        Ok(file) => file,
        Err(_f) => panic!("couldnt open file")
    };
    let root: Root = match serde_yaml::from_reader(f) {
        Ok(r) => r,
        Err(_) => panic!("couldnt parse schema")
    };
    if root.schema != "rload" {
        panic!("schema has to be a valid rload schema");
    }

    root.routes.iter().for_each(|route| {
        println!("loaded {} route", route.catch);
    });
    if root.debug {
        println!("debugging enabled");
    }

    let addr = root.listen
        .parse()
        .expect("Unable to parse socket address");

    // A `Service` is needed for every connection.
    let make_svc = make_service_fn(move |socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        let routes = root.routes.clone();
        let debug_enabled = root.debug.clone();
        service_fn(move |req: Request<Body>| {
            let route = routes.iter().filter(|route|
                match req.headers().get("host") {
                    Some(host) => host
                        .to_str()
                        .unwrap_or("")
                        .starts_with(route.catch.as_str()),
                    None => false
                }
            ).nth(0);
            match route {
                Some(x) => hyper_reverse_proxy::call(remote_addr.ip(), x.nodes.choose(&mut rand::thread_rng()).unwrap().as_str(), req),
                None => debug_request(debug_enabled, req),
            }
        })
    });

    let server = Server::bind(&addr)
        .serve(make_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Running server on {:?}", addr);

    // Run this server for... forever!
    hyper::rt::run(server);
}