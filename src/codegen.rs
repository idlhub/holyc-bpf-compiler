use crate::ast::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// BPF register allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpfReg {
    R0 = 0,  // Return value
    R1 = 1,  // 1st argument
    R2 = 2,  // 2nd argument
    R3 = 3,  // 3rd argument
    R4 = 4,  // 4th argument
    R5 = 5,  // 5th argument
    R6 = 6,  // Callee-saved
    R7 = 7,  // Callee-saved
    R8 = 8,  // Callee-saved
    R9 = 9,  // Callee-saved
    R10 = 10, // Stack pointer (read-only)
}

/// BPF instruction opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BpfOpcode {
    // Load/Store
    LdXDW = 0x79,    // Load double word
    StXDW = 0x7b,    // Store double word
    LdXW = 0x61,     // Load word
    StXW = 0x63,     // Store word
    LdXH = 0x69,     // Load half word
    StXH = 0x6b,     // Store half word
    LdXB = 0x71,     // Load byte
    StXB = 0x73,     // Store byte

    // ALU64
    Add64Imm = 0x07,  // Add immediate
    Add64Reg = 0x0f,  // Add register
    Sub64Imm = 0x17,  // Subtract immediate
    Sub64Reg = 0x1f,  // Subtract register
    Mul64Imm = 0x27,  // Multiply immediate
    Mul64Reg = 0x2f,  // Multiply register
    Div64Imm = 0x37,  // Divide immediate
    Div64Reg = 0x3f,  // Divide register
    Or64Imm = 0x47,   // OR immediate
    Or64Reg = 0x4f,   // OR register
    And64Imm = 0x57,  // AND immediate
    And64Reg = 0x5f,  // AND register
    Lsh64Imm = 0x77,  // Left shift immediate
    Lsh64Reg = 0x7f,  // Left shift register
    Rsh64Imm = 0x87,  // Right shift immediate (logical)
    Rsh64Reg = 0x8f,  // Right shift register (logical)
    Neg64 = 0x97,     // Negate
    Mod64Imm = 0xa7,  // Modulo immediate
    Mod64Reg = 0xaf,  // Modulo register
    Xor64Imm = 0xb7,  // XOR immediate
    Xor64Reg = 0xbf,  // XOR register
    Mov64Imm = 0xd7,  // Move immediate
    Mov64Reg = 0xdf,  // Move register
    Arsh64Imm = 0xe7, // Arithmetic right shift immediate
    Arsh64Reg = 0xef, // Arithmetic right shift register

    // ALU32 (same pattern with 0x04 base instead of 0x07)
    Add32Imm = 0x04,
    Add32Reg = 0x0c,
    Sub32Imm = 0x14,
    Sub32Reg = 0x1c,
    Mul32Imm = 0x24,
    Mul32Reg = 0x2c,
    Div32Imm = 0x34,
    Div32Reg = 0x3c,

    // Jumps
    Ja = 0x05,        // Jump always
    JeqImm = 0x15,    // Jump if equal immediate
    JeqReg = 0x1d,    // Jump if equal register
    JgtImm = 0x25,    // Jump if greater than immediate
    JgtReg = 0x2d,    // Jump if greater than register
    JgeImm = 0x35,    // Jump if greater or equal immediate
    JgeReg = 0x3d,    // Jump if greater or equal register
    JneImm = 0x55,    // Jump if not equal immediate
    JneReg = 0x5d,    // Jump if not equal register
    JsgtImm = 0x65,   // Jump if signed greater than immediate
    JsgtReg = 0x6d,   // Jump if signed greater than register
    JsgeImm = 0x75,   // Jump if signed greater or equal immediate
    JsgeReg = 0x7d,   // Jump if signed greater or equal register
    JltImm = 0xa5,    // Jump if less than immediate
    JltReg = 0xad,    // Jump if less than register
    JleImm = 0xb5,    // Jump if less or equal immediate
    JleReg = 0xbd,    // Jump if less or equal register
    JsltImm = 0xc5,   // Jump if signed less than immediate
    JsltReg = 0xcd,   // Jump if signed less than register
    JsleImm = 0xd5,   // Jump if signed less or equal immediate
    JsleReg = 0xdd,   // Jump if signed less or equal register

    // Call/Exit
    Call = 0x85,      // Function call
    Exit = 0x95,      // Exit program
}

/// BPF instruction (8 bytes)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BpfInstruction {
    pub opcode: u8,
    pub dst_src: u8,  // dst_reg:4, src_reg:4
    pub offset: i16,
    pub imm: i32,
}

impl BpfInstruction {
    pub fn new(opcode: BpfOpcode, dst: BpfReg, src: BpfReg, offset: i16, imm: i32) -> Self {
        Self {
            opcode: opcode as u8,
            dst_src: ((src as u8) << 4) | (dst as u8),
            offset,
            imm,
        }
    }

    pub fn mov_imm(dst: BpfReg, imm: i32) -> Self {
        Self::new(BpfOpcode::Mov64Imm, dst, BpfReg::R0, 0, imm)
    }

    pub fn mov_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Mov64Reg, dst, src, 0, 0)
    }

    pub fn add_imm(dst: BpfReg, imm: i32) -> Self {
        Self::new(BpfOpcode::Add64Imm, dst, BpfReg::R0, 0, imm)
    }

    pub fn add_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Add64Reg, dst, src, 0, 0)
    }

    pub fn sub_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Sub64Reg, dst, src, 0, 0)
    }

    pub fn mul_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Mul64Reg, dst, src, 0, 0)
    }

    pub fn div_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Div64Reg, dst, src, 0, 0)
    }

    pub fn mod_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Mod64Reg, dst, src, 0, 0)
    }

    pub fn and_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::And64Reg, dst, src, 0, 0)
    }

    pub fn or_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Or64Reg, dst, src, 0, 0)
    }

    pub fn xor_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Xor64Reg, dst, src, 0, 0)
    }

    pub fn lsh_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Lsh64Reg, dst, src, 0, 0)
    }

    pub fn rsh_reg(dst: BpfReg, src: BpfReg) -> Self {
        Self::new(BpfOpcode::Rsh64Reg, dst, src, 0, 0)
    }

    pub fn ldxdw(dst: BpfReg, src: BpfReg, offset: i16) -> Self {
        Self::new(BpfOpcode::LdXDW, dst, src, offset, 0)
    }

    pub fn stxdw(dst: BpfReg, src: BpfReg, offset: i16) -> Self {
        Self::new(BpfOpcode::StXDW, dst, src, offset, 0)
    }

    pub fn jeq_imm(dst: BpfReg, imm: i32, offset: i16) -> Self {
        Self::new(BpfOpcode::JeqImm, dst, BpfReg::R0, offset, imm)
    }

    pub fn jne_imm(dst: BpfReg, imm: i32, offset: i16) -> Self {
        Self::new(BpfOpcode::JneImm, dst, BpfReg::R0, offset, imm)
    }

    pub fn jgt_reg(dst: BpfReg, src: BpfReg, offset: i16) -> Self {
        Self::new(BpfOpcode::JgtReg, dst, src, offset, 0)
    }

    pub fn jge_reg(dst: BpfReg, src: BpfReg, offset: i16) -> Self {
        Self::new(BpfOpcode::JgeReg, dst, src, offset, 0)
    }

    pub fn jlt_reg(dst: BpfReg, src: BpfReg, offset: i16) -> Self {
        Self::new(BpfOpcode::JltReg, dst, src, offset, 0)
    }

    pub fn jle_reg(dst: BpfReg, src: BpfReg, offset: i16) -> Self {
        Self::new(BpfOpcode::JleReg, dst, src, offset, 0)
    }

    pub fn ja(offset: i16) -> Self {
        Self::new(BpfOpcode::Ja, BpfReg::R0, BpfReg::R0, offset, 0)
    }

    pub fn call(func_id: i32) -> Self {
        Self::new(BpfOpcode::Call, BpfReg::R0, BpfReg::R0, 0, func_id)
    }

    pub fn exit() -> Self {
        Self::new(BpfOpcode::Exit, BpfReg::R0, BpfReg::R0, 0, 0)
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0] = self.opcode;
        bytes[1] = self.dst_src;
        bytes[2..4].copy_from_slice(&self.offset.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.imm.to_le_bytes());
        bytes
    }
}

/// Code generator state
pub struct CodeGen {
    instructions: Vec<BpfInstruction>,
    variables: HashMap<String, (usize, Type)>, // name -> (stack_offset, type)
    stack_offset: usize,
    next_reg: usize,
    label_counter: usize,
    functions: HashMap<String, usize>, // function name -> func_id
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            variables: HashMap::new(),
            stack_offset: 0,
            next_reg: 6, // R6-R9 are callee-saved
            label_counter: 0,
            functions: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<Vec<u8>> {
        // First pass: register all functions
        for (idx, item) in program.items.iter().enumerate() {
            if let Item::FunctionDef(func) = item {
                self.functions.insert(func.name.clone(), idx);
            }
        }

        // Generate entrypoint
        for item in &program.items {
            self.generate_item(item)?;
        }

        // Convert instructions to bytes
        let mut bytecode = Vec::new();
        for inst in &self.instructions {
            bytecode.extend_from_slice(&inst.to_bytes());
        }

        Ok(bytecode)
    }

    fn generate_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::FunctionDef(func) => self.generate_function(func),
            Item::ClassDef(_) => Ok(()), // Classes are just type information
            Item::GlobalVar(_) => Ok(()), // Global variables handled separately
            Item::Define(_) => Ok(()),    // Defines are preprocessor directives
            Item::Include(_) => Ok(()),   // Includes are preprocessor directives
        }
    }

    fn generate_function(&mut self, func: &FunctionDef) -> Result<()> {
        // Reset local state for new function
        self.variables.clear();
        self.stack_offset = 0;

        // Allocate stack space for parameters
        for (idx, param) in func.params.iter().enumerate() {
            let offset = self.stack_offset;
            self.variables.insert(param.name.clone(), (offset, param.param_type.clone()));
            self.stack_offset += param.param_type.size_bytes();

            // Store parameter from register to stack
            let reg = match idx {
                0 => BpfReg::R1,
                1 => BpfReg::R2,
                2 => BpfReg::R3,
                3 => BpfReg::R4,
                4 => BpfReg::R5,
                _ => return Err(anyhow!("Too many parameters (max 5)")),
            };

            self.emit(BpfInstruction::stxdw(BpfReg::R10, reg, -(offset as i16) - 8));
        }

        // Generate function body
        for stmt in &func.body {
            self.generate_stmt(stmt)?;
        }

        // Ensure function returns (even if no explicit return)
        if !matches!(func.body.last(), Some(Stmt::Return(_))) {
            if func.return_type != Type::Void {
                self.emit(BpfInstruction::mov_imm(BpfReg::R0, 0));
            }
            self.emit(BpfInstruction::exit());
        }

        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::VarDecl(var) => {
                let offset = self.stack_offset;
                self.variables.insert(var.name.clone(), (offset, var.var_type.clone()));
                self.stack_offset += var.var_type.size_bytes();

                if let Some(init) = &var.init {
                    let reg = self.generate_expr(init)?;
                    self.emit(BpfInstruction::stxdw(BpfReg::R10, reg, -(offset as i16) - 8));
                }
                Ok(())
            }

            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
                Ok(())
            }

            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    let reg = self.generate_expr(expr)?;
                    if reg != BpfReg::R0 {
                        self.emit(BpfInstruction::mov_reg(BpfReg::R0, reg));
                    }
                }
                self.emit(BpfInstruction::exit());
                Ok(())
            }

            Stmt::If { condition, then_block, else_block } => {
                let cond_reg = self.generate_expr(condition)?;

                let else_label = self.new_label();
                let end_label = self.new_label();

                // Jump to else if condition is false (== 0)
                self.emit(BpfInstruction::jeq_imm(cond_reg, 0, 0)); // Offset will be patched
                let if_jump_idx = self.instructions.len() - 1;

                // Then block
                for stmt in then_block {
                    self.generate_stmt(stmt)?;
                }

                let mut else_jump_idx = None;
                if else_block.is_some() {
                    // Jump to end after then block
                    self.emit(BpfInstruction::ja(0)); // Offset will be patched
                    else_jump_idx = Some(self.instructions.len() - 1);
                }

                // Patch if jump to else/end
                let else_target = self.instructions.len();
                self.instructions[if_jump_idx].offset = (else_target - if_jump_idx - 1) as i16;

                // Else block
                if let Some(else_block) = else_block {
                    for stmt in else_block {
                        self.generate_stmt(stmt)?;
                    }
                }

                // Patch else jump to end
                if let Some(idx) = else_jump_idx {
                    let end_target = self.instructions.len();
                    self.instructions[idx].offset = (end_target - idx - 1) as i16;
                }

                Ok(())
            }

            Stmt::While { condition, body } => {
                let start = self.instructions.len();

                let cond_reg = self.generate_expr(condition)?;

                // Jump to end if condition is false
                self.emit(BpfInstruction::jeq_imm(cond_reg, 0, 0));
                let exit_jump_idx = self.instructions.len() - 1;

                // Loop body
                for stmt in body {
                    self.generate_stmt(stmt)?;
                }

                // Jump back to condition
                let back_offset = -((self.instructions.len() - start) as i16) - 1;
                self.emit(BpfInstruction::ja(back_offset));

                // Patch exit jump
                let end_target = self.instructions.len();
                self.instructions[exit_jump_idx].offset = (end_target - exit_jump_idx - 1) as i16;

                Ok(())
            }

            Stmt::Block(block) => {
                for stmt in block {
                    self.generate_stmt(stmt)?;
                }
                Ok(())
            }

            _ => Err(anyhow!("Unsupported statement: {:?}", stmt)),
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<BpfReg> {
        match expr {
            Expr::IntLiteral(n) => {
                let reg = self.alloc_reg()?;
                if *n <= i32::MAX as u64 {
                    self.emit(BpfInstruction::mov_imm(reg, *n as i32));
                } else {
                    // For large numbers, need to load in two parts
                    self.emit(BpfInstruction::mov_imm(reg, (*n >> 32) as i32));
                    self.emit(BpfInstruction::new(
                        BpfOpcode::Lsh64Imm,
                        reg,
                        BpfReg::R0,
                        0,
                        32,
                    ));
                    self.emit(BpfInstruction::new(
                        BpfOpcode::Or64Imm,
                        reg,
                        BpfReg::R0,
                        0,
                        (*n & 0xFFFFFFFF) as i32,
                    ));
                }
                Ok(reg)
            }

            Expr::Ident(name) => {
                if let Some(&(offset, _)) = self.variables.get(name) {
                    let reg = self.alloc_reg()?;
                    self.emit(BpfInstruction::ldxdw(reg, BpfReg::R10, -(offset as i16) - 8));
                    Ok(reg)
                } else {
                    Err(anyhow!("Undefined variable: {}", name))
                }
            }

            Expr::Binary { op, left, right } => {
                let left_reg = self.generate_expr(left)?;
                let right_reg = self.generate_expr(right)?;

                let inst = match op {
                    BinaryOp::Add => BpfInstruction::add_reg(left_reg, right_reg),
                    BinaryOp::Sub => BpfInstruction::sub_reg(left_reg, right_reg),
                    BinaryOp::Mul => BpfInstruction::mul_reg(left_reg, right_reg),
                    BinaryOp::Div => BpfInstruction::div_reg(left_reg, right_reg),
                    BinaryOp::Mod => BpfInstruction::mod_reg(left_reg, right_reg),
                    BinaryOp::BitAnd => BpfInstruction::and_reg(left_reg, right_reg),
                    BinaryOp::BitOr => BpfInstruction::or_reg(left_reg, right_reg),
                    BinaryOp::BitXor => BpfInstruction::xor_reg(left_reg, right_reg),
                    BinaryOp::Shl => BpfInstruction::lsh_reg(left_reg, right_reg),
                    BinaryOp::Shr => BpfInstruction::rsh_reg(left_reg, right_reg),
                    _ => return Err(anyhow!("Unsupported binary operator: {}", op)),
                };

                self.emit(inst);
                Ok(left_reg)
            }

            Expr::Assign { target, value } => {
                if let Expr::Ident(name) = &**target {
                    let value_reg = self.generate_expr(value)?;

                    if let Some((offset, _)) = self.variables.get(name) {
                        self.emit(BpfInstruction::stxdw(BpfReg::R10, value_reg, -(*offset as i16) - 8));
                        Ok(value_reg)
                    } else {
                        Err(anyhow!("Undefined variable: {}", name))
                    }
                } else {
                    Err(anyhow!("Invalid assignment target"))
                }
            }

            Expr::Call { func, args } => {
                if let Expr::Ident(func_name) = &**func {
                    // Load arguments into R1-R5
                    for (idx, arg) in args.iter().enumerate() {
                        let arg_reg = self.generate_expr(arg)?;
                        let param_reg = match idx {
                            0 => BpfReg::R1,
                            1 => BpfReg::R2,
                            2 => BpfReg::R3,
                            3 => BpfReg::R4,
                            4 => BpfReg::R5,
                            _ => return Err(anyhow!("Too many arguments (max 5)")),
                        };
                        if arg_reg != param_reg {
                            self.emit(BpfInstruction::mov_reg(param_reg, arg_reg));
                        }
                    }

                    // Call function
                    if let Some(func_id) = self.functions.get(func_name) {
                        self.emit(BpfInstruction::call(*func_id as i32));
                        Ok(BpfReg::R0) // Result in R0
                    } else {
                        Err(anyhow!("Undefined function: {}", func_name))
                    }
                } else {
                    Err(anyhow!("Invalid function call"))
                }
            }

            _ => Err(anyhow!("Unsupported expression: {:?}", expr)),
        }
    }

    fn emit(&mut self, inst: BpfInstruction) {
        self.instructions.push(inst);
    }

    fn alloc_reg(&mut self) -> Result<BpfReg> {
        let reg = match self.next_reg {
            6 => BpfReg::R6,
            7 => BpfReg::R7,
            8 => BpfReg::R8,
            9 => BpfReg::R9,
            _ => return Err(anyhow!("Out of registers")),
        };
        self.next_reg = (self.next_reg % 4) + 6; // Rotate R6-R9
        Ok(reg)
    }

    fn new_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpf_instruction_encoding() {
        let inst = BpfInstruction::mov_imm(BpfReg::R0, 42);
        let bytes = inst.to_bytes();
        assert_eq!(bytes[0], BpfOpcode::Mov64Imm as u8);
        assert_eq!(bytes[1], 0x00); // dst=R0, src=R0
        assert_eq!(&bytes[4..8], &42i32.to_le_bytes());
    }

    #[test]
    fn test_xor_instruction() {
        let inst = BpfInstruction::xor_reg(BpfReg::R6, BpfReg::R7);
        let bytes = inst.to_bytes();
        assert_eq!(bytes[0], BpfOpcode::Xor64Reg as u8);
        assert_eq!(bytes[1], 0x76); // dst=R6, src=R7
    }
}
