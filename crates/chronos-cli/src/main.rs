//! Chronos CLI - Command Line Interface

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use chronos_core::{ChronosCompiler, CompilerConfig, Target};

#[derive(Parser)]
#[command(name = "chronos")]
#[command(about = "The Self-Evolving Neural Compiler", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile from natural language description
    Compile {
        /// Natural language description
        #[arg(short, long)]
        description: String,

        /// Target architecture
        #[arg(short, long, default_value = "x86_64")]
        target: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "2")]
        optimization: u8,

        /// Verify with Z3
        #[arg(long, default_value = "true")]
        verify: bool,
    },

    /// Compile from a file
    CompileFile {
        /// Input file with description
        #[arg(short, long)]
        input: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start the REPL
    Repl,

    /// Show compiler info
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("debug")
        ).init();
    }

    match cli.command {
        Commands::Compile {
            description,
            target,
            output,
            optimization,
            verify,
        } => {
            let target = match target.as_str() {
                "x86_64" | "x86" => Target::X86_64,
                "arm64" | "aarch64" => Target::Arm64,
                "wasm" | "webassembly" => Target::WebAssembly,
                "cuda" | "gpu" => Target::Cuda,
                _ => {
                    eprintln!("Unknown target: {}", target);
                    std::process::exit(1);
                }
            };

            let config = CompilerConfig {
                target,
                optimization_level: optimization,
                verify,
                max_retries: 3,
            };

            println!("🌌 Chronos Compiler v0.1.0");
            println!("📝 Description: {}", description);
            println!("🎯 Target: {:?}", target);
            println!("⚡ Optimization: O{}", optimization);
            println!("🔍 Verification: {}", if verify { "enabled" } else { "disabled" });
            println!();

            let mut compiler = ChronosCompiler::new(config)?;
            let result = compiler.compile_from_nl(&description)?;

            println!("✅ Compilation successful!");
            println!("📊 IR nodes: {}", result.ir.node_count());
            println!("📄 Generated code:");
            println!("{}", "=".repeat(60));
            println!("{}", result.code);
            println!("{}", "=".repeat(60));

            if let Some(output_path) = output {
                std::fs::write(&output_path, &result.code)?;
                println!("💾 Written to: {}", output_path.display());
            }
        }

        Commands::CompileFile { input, output } => {
            let description = std::fs::read_to_string(&input)?;
            let config = CompilerConfig {
                target: Target::X86_64,
                optimization_level: 2,
                verify: true,
                max_retries: 3,
            };

            let mut compiler = ChronosCompiler::new(config)?;
            let result = compiler.compile_from_nl(&description)?;

            if let Some(output_path) = output {
                std::fs::write(&output_path, &result.code)?;
                println!("💾 Written to: {}", output_path.display());
            } else {
                println!("{}", result.code);
            }
        }

        Commands::Repl => {
            println!("🌌 Chronos REPL v0.1.0");
            println!("Type 'exit' to quit");
            println!();

            let mut compiler = ChronosCompiler::new(CompilerConfig {
                target: Target::X86_64,
                optimization_level: 2,
                verify: true,
                max_retries: 3,
            })?;

            loop {
                print!("chronos> ");
                use std::io::Write;
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim();

                if input == "exit" || input == "quit" {
                    break;
                }

                if input.is_empty() {
                    continue;
                }

                match compiler.compile_from_nl(input) {
                    Ok(result) => {
                        println!("{}", result.code);
                    }
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                    }
                }
            }
        }

        Commands::Info => {
            println!("🌌 Chronos - The Self-Evolving Neural Compiler");
            println!("Version: 0.1.0");
            println!("Author: Amir Hossein Nourzadeh");
            println!("License: MIT");
            println!();
            println!("Features:");
            println!("  ✅ Natural Language to Code");
            println!("  ✅ Formal Verification (Z3)");
            println!("  ✅ ML-based Optimization");
            println!("  ✅ Multi-target (x86, ARM, WASM, GPU)");
            println!("  ✅ Self-Hosting");
        }
    }

    Ok(())
}
