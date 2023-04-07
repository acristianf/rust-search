use std::fs::File;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::model::*;

pub fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> Result<(), ()> {
    println!(
        "Info: received request! method: {:?}, url{:?}",
        request.method(),
        request.url()
    );
    let content_type_header = Header::from_bytes("Content-Type", content_type)
        .expect("Didn't make an error in the header");
    let file_path = File::open(file_path).map_err(|err| {
        eprintln!("Error: could not serve file {file_path}; {err}");
    })?;
    let response = Response::from_file(file_path).with_header(content_type_header);
    request.respond(response).map_err(|err| {
        eprintln!("Error: could not serve a request; {err}");
    })
}

pub fn serve_404(request: Request) -> Result<(), ()> {
    request
        .respond(Response::from_string("404").with_status_code(StatusCode(404)))
        .map_err(|err| {
            eprintln!("Error: could not serve request; {err}");
        })
}

pub fn serve_request(model: &Model, mut request: Request) -> Result<(), ()> {
    match request.method() {
        Method::Post => match request.url() {
            "/api/search" => {
                let mut query = String::new();
                request.as_reader().read_to_string(&mut query).unwrap();
                let query = query.chars().collect::<Vec<_>>();

                let rank = search_query(&query, &model);

                // TODO: Delete this print
                println!("\n\n");
                for (path, total_tf) in rank.iter().take(10) {
                    println!("{path:?} => {total_tf}");
                }
            }
            _ => serve_404(request)?,
        },
        Method::Get => match request.url() {
            "/" | "/index.html" => {
                serve_static_file(request, "index.html", "text/html; charset=utf-8")?;
            }
            "/index.js" => {
                serve_static_file(request, "src/index.js", "text/javascript; charset=utf-8")?;
            }
            _ => serve_404(request)?,
        },
        _ => serve_404(request)?,
    }
    Ok(())
}

pub fn start(address: &str, model: &Model) -> Result<(), ()> {
    let server = Server::http(&address).map_err(|err| {
        eprintln!("Error: Couldn't start HTTP server on {address}; {err}");
    })?;

    println!("Info: listening at HTTP://{address}");

    for request in server.incoming_requests() {
        serve_request(&model, request)?
    }
    Ok(())
}
