use std::env;
use std::path::Path;
use std::process::ExitCode;

// Modules
mod model;
mod server;

use model::*;
use server::*;

fn usage(program: &str) {
    eprintln!("Usage: {program} [SubCommand] [Options]");
    eprintln!("Subcommands:");
    eprintln!("    index  <folder>        index the <folder> and save the files into a index.json");
    eprintln!(
        "    search <index-file>    check how many documents were indexed into the index file"
    );
    eprintln!("    serve                  start local HTTP server");
}

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");
    let subcommand = args.next().ok_or_else(|| {
        usage(&program);
        eprintln!("Error: no subcommand is provided");
    })?;

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                usage(&program);
                eprintln!("Error: no directory provided for indexing");
            })?;
            let mut model: Model = Default::default();
            let dir_path = Path::new(&dir_path);
            add_folder_to_model(&dir_path, &mut model);
            match save_model_as_json("index.json", &model) {
                Err(_) => {
                    eprintln!("Couldn't save index into index.json file");
                    return Err(());
                }
                _ => (),
            };
        }
        "serve" => {
            let index_file = args.next().unwrap();
            let index_file = Path::new(&index_file);
            let model: Model = load_index(&index_file).unwrap();

            let address = args.next().unwrap_or("127.0.0.1:8000".to_string());
            start(&address, &model)?;
        }
        _ => {
            usage(&program);
            eprintln!("Error: Unknown subcommand {subcommand}");
            return Err(());
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
