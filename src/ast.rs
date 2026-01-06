/// Abstract Syntax Tree for HolyC
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    FunctionDef(FunctionDef),
    ClassDef(ClassDef),
    GlobalVar(VarDecl),
    Define(Define),
    Include(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub return_type: Type,
    pub params: Vec<Param>,
    pub body: Block,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDef {
    pub name: String,
    pub fields: Vec<VarDecl>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarDecl {
    pub name: String,
    pub var_type: Type,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Define {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F64,
    Bool,
    Void,
    Pointer(Box<Type>),
    Array(Box<Type>, Option<usize>),
    Custom(String),
}

impl Type {
    pub fn size_bytes(&self) -> usize {
        match self {
            Type::U8 | Type::I8 | Type::Bool => 1,
            Type::U16 | Type::I16 => 2,
            Type::U32 | Type::I32 => 4,
            Type::U64 | Type::I64 | Type::F64 | Type::Pointer(_) => 8,
            Type::Void => 0,
            Type::Array(inner, Some(len)) => inner.size_bytes() * len,
            Type::Array(_, None) => 8, // Pointer to array
            Type::Custom(_) => 8, // Assume pointer
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 |
            Type::I8 | Type::I16 | Type::I32 | Type::I64
        )
    }

    pub fn is_unsigned(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::U8 => write!(f, "U8"),
            Type::U16 => write!(f, "U16"),
            Type::U32 => write!(f, "U32"),
            Type::U64 => write!(f, "U64"),
            Type::I8 => write!(f, "I8"),
            Type::I16 => write!(f, "I16"),
            Type::I32 => write!(f, "I32"),
            Type::I64 => write!(f, "I64"),
            Type::F64 => write!(f, "F64"),
            Type::Bool => write!(f, "Bool"),
            Type::Void => write!(f, "Void"),
            Type::Pointer(inner) => write!(f, "{}*", inner),
            Type::Array(inner, Some(len)) => write!(f, "{}[{}]", inner, len),
            Type::Array(inner, None) => write!(f, "{}[]", inner),
            Type::Custom(name) => write!(f, "{}", name),
        }
    }
}

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    VarDecl(VarDecl),
    Expr(Expr),
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
    },
    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Block,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    IntLiteral(u64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(u8),
    BoolLiteral(bool),
    Null,

    Ident(String),

    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },

    Member {
        expr: Box<Expr>,
        member: String,
    },

    Arrow {
        expr: Box<Expr>,
        member: String,
    },

    Cast {
        expr: Box<Expr>,
        target_type: Type,
    },

    Sizeof(Type),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,

    // Logical
    LogicalAnd,
    LogicalOr,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Assignment
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
}

impl BinaryOp {
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
        )
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
        )
    }

    pub fn is_bitwise(&self) -> bool {
        matches!(
            self,
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr
        )
    }

    pub fn is_assignment(&self) -> bool {
        matches!(
            self,
            BinaryOp::AddAssign
                | BinaryOp::SubAssign
                | BinaryOp::MulAssign
                | BinaryOp::DivAssign
                | BinaryOp::ModAssign
                | BinaryOp::AndAssign
                | BinaryOp::OrAssign
                | BinaryOp::XorAssign
                | BinaryOp::ShlAssign
                | BinaryOp::ShrAssign
        )
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::AddAssign => "+=",
            BinaryOp::SubAssign => "-=",
            BinaryOp::MulAssign => "*=",
            BinaryOp::DivAssign => "/=",
            BinaryOp::ModAssign => "%=",
            BinaryOp::AndAssign => "&=",
            BinaryOp::OrAssign => "|=",
            BinaryOp::XorAssign => "^=",
            BinaryOp::ShlAssign => "<<=",
            BinaryOp::ShrAssign => ">>=",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    AddressOf,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
            UnaryOp::Deref => "*",
            UnaryOp::AddressOf => "&",
            UnaryOp::PreIncrement => "++",
            UnaryOp::PreDecrement => "--",
            UnaryOp::PostIncrement => "++",
            UnaryOp::PostDecrement => "--",
        };
        write!(f, "{}", s)
    }
}

// AST visitors for analysis and transformation
pub trait Visitor {
    fn visit_program(&mut self, program: &Program) {
        for item in &program.items {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::FunctionDef(f) => self.visit_function(f),
            Item::ClassDef(c) => self.visit_class(c),
            Item::GlobalVar(v) => self.visit_var_decl(v),
            Item::Define(_) => {}
            Item::Include(_) => {}
        }
    }

    fn visit_function(&mut self, _func: &FunctionDef) {}
    fn visit_class(&mut self, _class: &ClassDef) {}
    fn visit_var_decl(&mut self, _var: &VarDecl) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_sizes() {
        assert_eq!(Type::U8.size_bytes(), 1);
        assert_eq!(Type::U64.size_bytes(), 8);
        assert_eq!(Type::Pointer(Box::new(Type::U64)).size_bytes(), 8);
        assert_eq!(Type::Array(Box::new(Type::U32), Some(10)).size_bytes(), 40);
    }

    #[test]
    fn test_type_checks() {
        assert!(Type::U64.is_integer());
        assert!(Type::U64.is_unsigned());
        assert!(!Type::I64.is_unsigned());
        assert!(!Type::Bool.is_integer());
    }

    #[test]
    fn test_binary_op_categories() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(BinaryOp::Eq.is_comparison());
        assert!(BinaryOp::BitXor.is_bitwise());
        assert!(BinaryOp::AddAssign.is_assignment());
    }
}
