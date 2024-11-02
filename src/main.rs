use clap::{Args, Parser, Subcommand};
use std::{fs, io::Write, path::Path};

use parser::{codegen, parse, utils::print_sym_table, Analyzer};
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
    #[arg(short, long)]
    /// Gode generation
    codegen: bool,
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
                            let analyzer = Analyzer::new();
                            let (errors, symbol_table) = analyzer.analyze(&mut root); // Hacer mutable la tabla de símbolos
                            println!("Arbol con anotaciones:");
                            root.print(); // imprimir a stdout
                            if !errors.is_empty() {
                                eprintln!("Errores al analizar semánticamente:");
                                for error in errors {
                                    eprintln!(
                                        "ERROR: {} en la posición {:?}",
                                        error.message, error.cursor
                                    );
                                }
                            } else {
                                if cli.codegen {
                                    // Generación de codigo
                                    let output_file =
                                        Path::new(&file).parent().unwrap_or(Path::new(".")).join(
                                            Path::new(&file)
                                                .file_name()
                                                .map(|f| f.to_str().unwrap_or("program"))
                                                .unwrap_or("program")
                                                .to_owned()
                                                + ".vm",
                                        );
                                    if cli.verbose {
                                        println!(
                                            "[VERBOSE] Trying to generate code to {:?}",
                                            output_file.to_str()
                                        );
                                    }

                                    if let Ok(mut file_handle) =
                                        fs::File::create(output_file.clone())
                                    {
                                        let text = codegen::CodeGen::new().generate(&root);
                                        match file_handle.write_fmt(format_args!("{}", text)) {
                                            Ok(_) => {}
                                            Err(e) => eprint!("Error al escribir en el archivo"),
                                        }
                                    } else {
                                        eprintln!("ERROR: Could not create file {}", file);
                                    }
                                }
                            }
                            if cli.symbols {
                                print_sym_table(&symbol_table);
                            }
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
