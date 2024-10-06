use std::{fs, io::Write, path::Path};

use clap::{Args, Parser, Subcommand};
use parser::{
    create_symbol_table, debug, evaluate_arithmetic_expressions, parse, utils::print_sym_table,
};
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
    /// Don't write tokens to files
    save: bool,
    #[arg(short, long)]
    /// Output file to a json
    json: bool,
    #[arg(short, long)]
    /// Semantic analysis
    analyze: bool,
    /// Output symboltable to stdout
    #[arg(long)]
    symbols: bool,
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
    let mut cli = Cli::parse();
    cli.analyze = true;

    match &cli.command {
        Commands::Build(args) => {
            let file = args.file.clone();
            println!("{}", file);
            let tokenization = tokenize_file(&file);
            match tokenization {
                Ok(res) => {
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
                            eprintln!(
                                "ERROR: Could not save file {} due to invalid filename",
                                file
                            );
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
                    let (root_op, errors) = parse(res.0);
                    if let Some(mut root) = root_op {
                        //hubo arbol
                        if let Some(filename) = Path::new(&file).file_name() {
                            if cli.json {
                                let json_file = Path::new(&file)
                                    .parent()
                                    .unwrap_or(Path::new("."))
                                    .join(filename.to_str().unwrap().to_owned() + ".json");
                                println!("[JSON] Trying to save to {:?}", json_file.to_str());
                                let json = serde_json::to_string_pretty(&root);
                                match json {
                                    Ok(str) => {
                                        if let Ok(mut file_handle) =
                                            fs::File::create(json_file.clone())
                                        {
                                            if let Err(_) =
                                                file_handle.write_fmt(format_args!("{}", str))
                                            {
                                                eprintln!(
                                                    "ERROR: Could not write to {}",
                                                    json_file.to_str().unwrap()
                                                );
                                            }
                                        } else {
                                            eprintln!(
                                                "ERROR: Could not create file {}",
                                                json_file.to_string_lossy()
                                            );
                                        }
                                    }
                                    Err(_) => todo!(),
                                }
                            } else {
                                eprintln!(
                                    "ERROR: Could not save file {} due to invalid filename",
                                    file
                                );
                            }
                        }
                        root.print(); // imprimir a stdout

                        if cli.analyze {
                            let (mut symbol_table, errors) = create_symbol_table(&root); // Hacer mutable la tabla de símbolos

                            if !errors.is_empty() {
                                eprintln!("Errores al construir tabla de simbolos:");
                                for error in errors {
                                    eprintln!(
                                        "ERROR: {} en la posición {:?}",
                                        error.message, error.cursor
                                    );
                                }
                            }
                            let eval_errors =
                                evaluate_arithmetic_expressions(&mut root, &mut symbol_table);
                            if !eval_errors.is_empty() {
                                eprintln!("Errores de evaluación:");
                                for error in eval_errors {
                                    eprintln!(
                                        "ERROR: {} en la posición {:?}",
                                        error.message, error.cursor
                                    );
                                }
                            }
                            if cli.symbols {
                                print_sym_table(&symbol_table);
                            }
                            root.print(); // imprimir a stdout
                        }
                    }
                    if !errors.is_empty() {
                        for err in errors {
                            eprintln!("ERROR: Parsing error: {:?}", err);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Could not generate output for {}: {}", file, e)
                }
            }
        }
    }
}
