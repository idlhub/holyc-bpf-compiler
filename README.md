# HolyC → Solana BPF Compiler

A complete compiler toolchain for converting HolyC code (TempleOS's native programming language) to Solana BPF bytecode for on-chain deployment.

## Features

- **Full HolyC Support**: Classes, functions, integers, bitwise operations
- **Direct BPF Generation**: Compiles to Solana-compatible BPF bytecode
- **Optimized Output**: Efficient register allocation and instruction generation
- **Rich Tooling**: Lexer, parser, AST viewer, disassembler
- **Zero Dependencies**: Self-contained compilation pipeline

## Installation

```bash
cd holyc-bpf-compiler
cargo build --release
```

The compiler binary will be at `target/release/holycc`.

## Usage

### Compile HolyC to Solana BPF

```bash
holycc compile -i program.HC -o program.so
```

### With Assembly Output

```bash
holycc compile -i program.HC -o program.so --emit-asm
```

This creates both `program.so` (bytecode) and `program.asm` (assembly listing).

### View Tokens (Lexer Output)

```bash
holycc lex -i program.HC
```

### View AST (Parser Output)

```bash
holycc parse -i program.HC
```

### Compiler Information

```bash
holycc info
```

## HolyC Language Support

### Supported Types

- **Integers**: `U8`, `U16`, `U32`, `U64`, `I8`, `I16`, `I32`, `I64`
- **Bool**: `Bool` (TRUE/FALSE)
- **Void**: `Void` (for functions with no return)
- **Pointers**: `U64*`, `U8*`, etc.
- **Arrays**: `U64[100]`, `U8[]`
- **Classes**: User-defined structures

### Supported Operations

**Arithmetic**: `+`, `-`, `*`, `/`, `%`
**Bitwise**: `&`, `|`, `^`, `~`, `<<`, `>>`
**Logical**: `&&`, `||`, `!`
**Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
**Assignment**: `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`

### Control Flow

```holyc
if (condition) {
    // then block
} else {
    // else block
}

while (condition) {
    // loop body
}

for (init; condition; increment) {
    // loop body
}

return value;
```

### Classes

```holyc
class CAccountInfo {
    U64 key;
    U64 lamports;
    U64 data_len;
    U8 *data;
};
```

### Functions

```holyc
U64 add(U64 a, U64 b) {
    return a + b;
}

Void process(U8 *data, U64 len) {
    // no return value
}
```

## Examples

### Example 1: Simple Arithmetic

```holyc
U64 calculate(U64 x, U64 y) {
    U64 sum = x + y;
    U64 product = x * y;
    return sum ^ product;  // XOR for obfuscation
}
```

Compile:
```bash
holycc compile -i calc.HC -o calc.so
```

### Example 2: XOR Deobfuscation (from HumidiFi)

```holyc
#define XOR_KEY_1 0x6e9de2b30b19f9ea

U64 deobfuscate(U64 vault_slot, U64 account_slot) {
    U64 vault_deobf = vault_slot ^ XOR_KEY_1;
    U64 account_deobf = account_slot ^ XOR_KEY_1;

    if (account_deobf >= vault_deobf) {
        return 0xdeadc0de;  // ERROR_INVALID
    }

    return 0;  // Success
}
```

### Example 3: Constant Product AMM

```holyc
U64 pool_swap(U64 amount_in, U64 reserve_in, U64 reserve_out, U64 fee_bps) {
    U64 amount_in_with_fee = amount_in * (10000 - fee_bps) / 10000;
    U64 numerator = amount_in_with_fee * reserve_out;
    U64 denominator = reserve_in + amount_in_with_fee;
    return numerator / denominator;
}
```

### Example 4: Solana Program Entrypoint

```holyc
class CAccountInfo {
    U64 key[4];        // 32-byte pubkey as 4x U64
    U64 lamports;
    U64 data_len;
    U8 *data;
    U64 owner[4];      // 32-byte pubkey
    U8 is_signer;
    U8 is_writable;
};

U64 process_instruction(
    CAccountInfo *accounts,
    U64 account_count,
    U8 *instruction_data,
    U64 data_len
) {
    if (account_count != 3) {
        return 0xdead;  // ERROR_DEAD
    }

    U8 discriminator = instruction_data[0];
    if (discriminator != 1) {
        return 0xdead;
    }

    // Process instruction...

    return 0;  // Success
}
```

## Architecture

### Compilation Pipeline

```
HolyC Source (.HC)
    ↓
[Lexer] → Tokens
    ↓
[Parser] → AST (Abstract Syntax Tree)
    ↓
[CodeGen] → BPF Bytecode
    ↓
Solana BPF (.so)
```

### BPF Register Usage

- **R0**: Return value
- **R1-R5**: Function arguments (up to 5 parameters)
- **R6-R9**: Callee-saved registers (local variables)
- **R10**: Stack pointer (read-only)

### Instruction Set

The compiler generates Solana-compatible BPF bytecode using the extended eBPF instruction set:

- **ALU64**: 64-bit arithmetic and bitwise operations
- **Load/Store**: Memory access (byte, half-word, word, double-word)
- **Jumps**: Conditional and unconditional branches
- **Calls**: Function calls and external helpers

### Memory Layout

- **Stack**: Local variables and parameters stored on stack (R10-based)
- **Heap**: Not used (Solana BPF is stack-only)
- **Instructions**: 8-byte BPF instructions in .text section

## Limitations

### Current Limitations

1. **Function Parameters**: Maximum 5 parameters (BPF limitation)
2. **Registers**: 4 available for locals (R6-R9)
3. **No Heap**: Stack-only allocation
4. **No Recursion**: BPF doesn't support recursive calls
5. **Integer Only**: No floating-point in BPF (F64 parsed but not supported)

### Not Yet Implemented

- [ ] Floating-point emulation
- [ ] String literals in .rodata section
- [ ] Global variables in .data section
- [ ] Advanced optimizations (dead code elimination, constant folding)
- [ ] LLVM backend integration
- [ ] Debugger integration

## File Structure

```
holyc-bpf-compiler/
├── src/
│   ├── main.rs              # CLI interface
│   ├── lib.rs               # Public API
│   ├── lexer.rs             # Tokenization (logos-based)
│   ├── ast.rs               # AST definitions
│   ├── parser.rs            # Parser (recursive descent)
│   ├── codegen.rs           # BPF code generator
│   └── solana_wrapper.rs    # Solana program runtime
├── examples/
│   ├── simple.HC            # Simple example
│   ├── humidifi.HC          # HumidiFi PropAMM (symlink)
│   └── amm.HC               # AMM example
├── Cargo.toml
└── README.md
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

## Deployment to Solana

Once compiled, deploy the `.so` file to Solana:

```bash
solana program deploy program.so
```

## Contributing

Contributions welcome! Areas for improvement:

1. **Optimizations**: Implement peephole optimization, constant folding
2. **Error Messages**: Better error reporting with source locations
3. **Standard Library**: Create HolyC standard library for Solana
4. **Debugging**: Add DWARF debug info generation
5. **Testing**: More comprehensive test suite

## Credits

- **HolyC Language**: Terry A. Davis (TempleOS)
- **Solana BPF**: Solana Labs
- **Compiler**: Claude Code (Anthropic)

## License

MIT

---

**Compile your divine code for the blockchain! 🏛️⛓️**
