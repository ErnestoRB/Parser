use std::{fs, io::Write, path::Path};

use clap::{Args, Parser, Subcommand};
use parser::parse;
use scanner::tokenize_file;

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"), author = "Ernesto Ramírez (https://github.com/ErnestoRB)", about = "CLI parser for Vanilla Lang", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    /// Turn on/off processing output
    verbose: bool,
    #[arg(short, long)]
    /// Dont write tokens to files
    save: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Parse files and print tree to stdout
    Build(BuildArgs),
}

#[derive(Args, Clone)]
struct BuildArgs {
    file: String,
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Build(args) => {
            let file = args.file.clone();
            println!("{}", file);
            let tokenization = tokenize_file(&file);
            if let Ok(res) = tokenization {
                if cli.verbose {
                    println!(
                        "[VERBOSE] Tokenizing {}: {} Tokens, {} Errors",
                        file,
                        res.0.len(),
                        res.1.len()
                    );
                }
                if cli.save {
                    if let Some(filename) = Path::new(&file).file_name() {
                        let output_file = Path::new(&file)
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(filename.to_str().unwrap().to_owned() + ".lex");
                        if cli.verbose {
                            println!("[VERBOSE] Trying to save to {:?}", output_file.to_str());
                        }
                        if let Ok(mut file_handle) = fs::File::create(output_file.clone()) {
                            for token in res.0.iter() {
                                // Imprimir tokens
                                if let Err(_) = file_handle.write_fmt(format_args!(
                                    "{:?}, {}\n",
                                    token.token_type, token.lexemme
                                )) {
                                    eprintln!(
                                        "ERROR: Could not write to {}",
                                        output_file.to_str().unwrap()
                                    );
                                }
                            }
                        } else {
                            eprintln!("ERROR: Could not create file {}", file);
                        }
                    } else {
                        eprintln!("ERROR: Could not save file {} due invalid filename", file);
                    }
                }
                if cli.verbose {
                    for err in res.1.iter() {
                        eprintln!(
                            "ERROR: ({}, [({}, {})-({},{})]): {} ",
                            file,
                            err.start.col,
                            err.start.lin,
                            err.end.col,
                            err.end.lin,
                            err.message,
                        );
                    }
                }
                let parse_result = parse(res.0);
                match parse_result {
                    Ok(root_op) => {
                        if let Some(root) = root_op {
                            root.print();
                        }
                    }
                    Err(err) => {
                        eprintln!("ERROR: Parsing error: {:?}", err)
                    }
                }
            } else if let Err(e) = tokenization {
                eprintln!("ERROR: Could not generate output for {}: {}", file, e)
            }
        }
    }
}
