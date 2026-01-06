/// HolyC to Solana BPF Compiler Library
///
/// This library provides a complete toolchain for compiling HolyC code
/// to Solana BPF bytecode that can be deployed on-chain.
///
/// # Architecture
///
/// The compilation pipeline consists of:
/// 1. **Lexer** - Tokenizes HolyC source code
/// 2. **Parser** - Builds Abstract Syntax Tree (AST)
/// 3. **CodeGen** - Generates Solana BPF bytecode
/// 4. **Wrapper** - Provides Solana program runtime interface
///
/// # Example
///
/// ```no_run
/// use holyc_bpf_compiler::{compile_source, CompilerOptions};
///
/// let source = r#"
///     U64 add(U64 a, U64 b) {
///         return a + b;
///     }
/// "#;
///
/// let options = CompilerOptions::default();
/// let bytecode = compile_source(source, options).unwrap();
/// ```

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod codegen;
pub mod solana_wrapper;

use anyhow::{anyhow, Context, Result};

/// Compiler options
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    /// Emit assembly listing
    pub emit_asm: bool,
    /// Emit AST as JSON
    pub emit_ast: bool,
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Verbose output
    pub verbose: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            emit_asm: false,
            emit_ast: false,
            opt_level: 0,
            verbose: false,
        }
    }
}

/// Compile HolyC source code to Solana BPF bytecode
pub fn compile_source(source: &str, options: CompilerOptions) -> Result<Vec<u8>> {
    // Lex
    let token_results = lexer::Lexer::collect_tokens(source)
        .map_err(|e| anyhow!("Lexical analysis failed: {}", e))?;
    let tokens: Vec<_> = token_results.into_iter().map(|(t, _)| t).collect();

    if options.verbose {
        println!("Lexed {} tokens", tokens.len());
    }

    // Parse
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse().context("Parsing failed")?;

    if options.verbose {
        println!("Parsed {} items", program.items.len());
    }

    // Generate bytecode
    let mut codegen = codegen::CodeGen::new();
    let bytecode = codegen.generate(&program)
        .context("Code generation failed")?;

    if options.verbose {
        println!("Generated {} bytes ({} instructions)", bytecode.len(), bytecode.len() / 8);
    }

    Ok(bytecode)
}

/// Compile HolyC source file to bytecode
pub fn compile_file(path: &str, options: CompilerOptions) -> Result<Vec<u8>> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path))?;

    compile_source(&source, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_function() {
        let source = r#"
            U64 add(U64 a, U64 b) {
                return a + b;
            }
        "#;

        let options = CompilerOptions::default();
        let result = compile_source(source, options);

        assert!(result.is_ok());
        let bytecode = result.unwrap();
        assert!(!bytecode.is_empty());
        assert_eq!(bytecode.len() % 8, 0); // Must be multiple of 8 (instruction size)
    }

    #[test]
    fn test_compile_class() {
        let source = r#"
            class Point {
                U64 x;
                U64 y;
            };

            U64 distance_squared(Point* p1, Point* p2) {
                U64 dx = p2->x - p1->x;
                U64 dy = p2->y - p1->y;
                return dx * dx + dy * dy;
            }
        "#;

        let options = CompilerOptions::default();
        let result = compile_source(source, options);

        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_xor_obfuscation() {
        let source = r#"
            U64 deobfuscate(U64 value, U64 key) {
                return value ^ key;
            }
        "#;

        let options = CompilerOptions::default();
        let result = compile_source(source, options);

        assert!(result.is_ok());
        let bytecode = result.unwrap();

        // Check that XOR instruction is present (0xbf opcode for XOR64Reg)
        assert!(bytecode.chunks(8).any(|chunk| chunk[0] == 0xbf));
    }
}
