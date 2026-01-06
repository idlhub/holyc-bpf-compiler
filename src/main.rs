use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod lexer;
mod ast;
mod parser;
mod codegen;
mod solana_wrapper;

use lexer::Lexer;
use parser::Parser as HolyCParser;
use codegen::CodeGen;

#[derive(Parser)]
#[command(name = "holycc")]
#[command(about = "HolyC to Solana BPF compiler", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile HolyC source to Solana BPF bytecode
    Compile {
        /// Input HolyC source file
        #[arg(short, long)]
        input: PathBuf,

        /// Output BPF bytecode file (.so)
        #[arg(short, long)]
        output: PathBuf,

        /// Emit assembly listing
        #[arg(short = 'S', long)]
        emit_asm: bool,

        /// Emit AST as JSON
        #[arg(long)]
        emit_ast: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Lex a HolyC file (tokenization only)
    Lex {
        /// Input HolyC source file
        #[arg(short, long)]
        input: PathBuf,

        /// Output tokens as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Parse a HolyC file (AST generation only)
    Parse {
        /// Input HolyC source file
        #[arg(short, long)]
        input: PathBuf,

        /// Output AST as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Show compiler information
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            emit_asm,
            emit_ast,
            verbose,
        } => compile(&input, &output, emit_asm, emit_ast, verbose),

        Commands::Lex { input, json } => lex_file(&input, json),

        Commands::Parse { input, json } => parse_file(&input, json),

        Commands::Info => show_info(),
    }
}

fn compile(
    input: &PathBuf,
    output: &PathBuf,
    emit_asm: bool,
    emit_ast: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("HolyC → Solana BPF Compiler");
        println!("Input:  {}", input.display());
        println!("Output: {}", output.display());
        println!();
    }

    // Read source file
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    if verbose {
        println!("[1/4] Lexing...");
    }

    // Lex
    let token_results = Lexer::collect_tokens(&source)
        .map_err(|e| anyhow!("Lexical analysis failed: {}", e))?;
    let tokens: Vec<_> = token_results.into_iter().map(|(t, _)| t).collect();

    if verbose {
        println!("      Found {} tokens", tokens.len());
    }

    if verbose {
        println!("[2/4] Parsing...");
    }

    // Parse
    let mut parser = HolyCParser::new(tokens);
    let program = parser.parse().context("Parsing failed")?;

    if verbose {
        println!("      Parsed {} top-level items", program.items.len());
    }

    // Emit AST if requested
    if emit_ast {
        let ast_path = output.with_extension("ast.json");
        let ast_json = serde_json::to_string_pretty(&program)
            .context("Failed to serialize AST")?;
        fs::write(&ast_path, ast_json)
            .with_context(|| format!("Failed to write AST to {}", ast_path.display()))?;

        if verbose {
            println!("      Wrote AST to {}", ast_path.display());
        }
    }

    if verbose {
        println!("[3/4] Generating BPF code...");
    }

    // Generate BPF bytecode
    let mut codegen = CodeGen::new();
    let bytecode = codegen.generate(&program)
        .context("Code generation failed")?;

    if verbose {
        println!("      Generated {} bytes of BPF bytecode", bytecode.len());
        println!("      {} instructions", bytecode.len() / 8);
    }

    if verbose {
        println!("[4/4] Writing output...");
    }

    // Write bytecode to output file
    fs::write(output, &bytecode)
        .with_context(|| format!("Failed to write output to {}", output.display()))?;

    // Emit assembly if requested
    if emit_asm {
        let asm_path = output.with_extension("asm");
        let asm = disassemble_bytecode(&bytecode);
        fs::write(&asm_path, asm)
            .with_context(|| format!("Failed to write assembly to {}", asm_path.display()))?;

        if verbose {
            println!("      Wrote assembly to {}", asm_path.display());
        }
    }

    if verbose {
        println!();
        println!("✓ Compilation successful!");
        println!("  Output: {}", output.display());
    } else {
        println!("Compiled {} → {}", input.display(), output.display());
    }

    Ok(())
}

fn lex_file(input: &PathBuf, json: bool) -> Result<()> {
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    let tokens = Lexer::collect_tokens(&source)
        .map_err(|e| anyhow!("Lexical analysis failed: {}", e))?;

    if json {
        let json = serde_json::to_string_pretty(&tokens)
            .context("Failed to serialize tokens")?;
        println!("{}", json);
    } else {
        println!("Tokens for {}:", input.display());
        println!("{:-<60}", "");
        for (idx, (token, range)) in tokens.iter().enumerate() {
            println!("{:4}: {:?} @ {}..{}", idx, token, range.start, range.end);
        }
        println!("{:-<60}", "");
        println!("Total: {} tokens", tokens.len());
    }

    Ok(())
}

fn parse_file(input: &PathBuf, json: bool) -> Result<()> {
    let source = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    let token_results = Lexer::collect_tokens(&source)
        .map_err(|e| anyhow!("Lexical analysis failed: {}", e))?;
    let tokens: Vec<_> = token_results.into_iter().map(|(t, _)| t).collect();

    let mut parser = HolyCParser::new(tokens);
    let program = parser.parse().context("Parsing failed")?;

    if json {
        let json = serde_json::to_string_pretty(&program)
            .context("Failed to serialize AST")?;
        println!("{}", json);
    } else {
        println!("AST for {}:", input.display());
        println!("{:-<60}", "");
        println!("{:#?}", program);
        println!("{:-<60}", "");
        println!("Total: {} top-level items", program.items.len());
    }

    Ok(())
}

fn show_info() -> Result<()> {
    println!("HolyC → Solana BPF Compiler (holycc)");
    println!();
    println!("Version:       {}", env!("CARGO_PKG_VERSION"));
    println!("Authors:       {}", env!("CARGO_PKG_AUTHORS"));
    println!();
    println!("Capabilities:");
    println!("  - Lexical analysis (HolyC tokenization)");
    println!("  - Syntax parsing (AST generation)");
    println!("  - BPF code generation (Solana-compatible)");
    println!("  - Direct .so output for Solana deployment");
    println!();
    println!("Supported HolyC features:");
    println!("  - Integer types: U8, U16, U32, U64, I8, I16, I32, I64");
    println!("  - Classes and structs");
    println!("  - Functions (up to 5 parameters)");
    println!("  - Arithmetic: +, -, *, /, %");
    println!("  - Bitwise: &, |, ^, ~, <<, >>");
    println!("  - Comparisons: ==, !=, <, <=, >, >=");
    println!("  - Control flow: if/else, while, for");
    println!("  - Pointers and arrays");
    println!();
    println!("BPF Target:");
    println!("  - eBPF extended instruction set");
    println!("  - Solana BPF VM compatibility");
    println!("  - Direct deployment to Solana");
    println!();
    println!("Usage examples:");
    println!("  holycc compile -i program.HC -o program.so");
    println!("  holycc compile -i program.HC -o program.so --emit-asm");
    println!("  holycc lex -i program.HC");
    println!("  holycc parse -i program.HC --json");

    Ok(())
}

fn disassemble_bytecode(bytecode: &[u8]) -> String {
    use codegen::BpfInstruction;

    let mut output = String::new();
    output.push_str("; HolyC-compiled BPF assembly\n");
    output.push_str("; Generated by holycc\n\n");

    for (idx, chunk) in bytecode.chunks(8).enumerate() {
        if chunk.len() == 8 {
            let inst = unsafe {
                std::ptr::read(chunk.as_ptr() as *const BpfInstruction)
            };

            output.push_str(&format!(
                "{:04x}: {:02x} {:02x} {:04x} {:08x}  ; ",
                idx * 8,
                inst.opcode,
                inst.dst_src,
                inst.offset as u16,
                inst.imm as u32
            ));

            // Decode instruction
            let dst = inst.dst_src & 0x0F;
            let src = (inst.dst_src >> 4) & 0x0F;

            match inst.opcode {
                0xd7 => output.push_str(&format!("mov r{}, {}\n", dst, inst.imm)),
                0xdf => output.push_str(&format!("mov r{}, r{}\n", dst, src)),
                0x07 => output.push_str(&format!("add r{}, {}\n", dst, inst.imm)),
                0x0f => output.push_str(&format!("add r{}, r{}\n", dst, src)),
                0x1f => output.push_str(&format!("sub r{}, r{}\n", dst, src)),
                0x2f => output.push_str(&format!("mul r{}, r{}\n", dst, src)),
                0x3f => output.push_str(&format!("div r{}, r{}\n", dst, src)),
                0xaf => output.push_str(&format!("mod r{}, r{}\n", dst, src)),
                0x5f => output.push_str(&format!("and r{}, r{}\n", dst, src)),
                0x4f => output.push_str(&format!("or r{}, r{}\n", dst, src)),
                0xbf => output.push_str(&format!("xor r{}, r{}\n", dst, src)),
                0x7f => output.push_str(&format!("lsh r{}, r{}\n", dst, src)),
                0x8f => output.push_str(&format!("rsh r{}, r{}\n", dst, src)),
                0x79 => output.push_str(&format!("ldxdw r{}, [r{}+{}]\n", dst, src, inst.offset)),
                0x7b => output.push_str(&format!("stxdw [r{}+{}], r{}\n", dst, inst.offset, src)),
                0x15 => output.push_str(&format!("jeq r{}, {}, {:+}\n", dst, inst.imm, inst.offset)),
                0x55 => output.push_str(&format!("jne r{}, {}, {:+}\n", dst, inst.imm, inst.offset)),
                0x05 => output.push_str(&format!("ja {:+}\n", inst.offset)),
                0x85 => output.push_str(&format!("call {}\n", inst.imm)),
                0x95 => output.push_str("exit\n"),
                _ => output.push_str(&format!("??? (0x{:02x})\n", inst.opcode)),
            }
        }
    }

    output.push_str(&format!("\nTotal: {} instructions\n", bytecode.len() / 8));
    output
}
