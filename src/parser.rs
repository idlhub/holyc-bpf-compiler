use crate::ast::*;
use crate::lexer::Token;
use anyhow::{anyhow, Result};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item> {
        // Handle preprocessor directives
        if let Some(Token::Define(def)) = self.peek().cloned() {
            self.advance();
            return Ok(Item::Define(self.parse_define(&def)?));
        }

        if let Some(Token::Include(inc)) = self.peek().cloned() {
            self.advance();
            return Ok(Item::Include(inc));
        }

        // Check for class definition
        if self.match_token(&Token::Class) {
            return Ok(Item::ClassDef(self.parse_class()?));
        }

        // Parse function or global variable
        let return_type = self.parse_type()?;
        let name = self.expect_ident()?;

        if self.match_token(&Token::LeftParen) {
            // Function definition
            let params = self.parse_params()?;
            self.expect(&Token::RightParen)?;
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_contents()?;
            self.expect(&Token::RightBrace)?;

            Ok(Item::FunctionDef(FunctionDef {
                name,
                return_type,
                params,
                body,
                is_public: true,
            }))
        } else {
            // Global variable
            let init = if self.match_token(&Token::Assign) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&Token::Semicolon)?;

            Ok(Item::GlobalVar(VarDecl {
                name,
                var_type: return_type,
                init,
            }))
        }
    }

    fn parse_define(&self, def_str: &str) -> Result<Define> {
        // Parse #define NAME VALUE
        let parts: Vec<&str> = def_str.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid #define directive"));
        }

        Ok(Define {
            name: parts[1].to_string(),
            value: parts[2..].join(" "),
        })
    }

    fn parse_class(&mut self) -> Result<ClassDef> {
        let name = self.expect_ident()?;
        self.expect(&Token::LeftBrace)?;

        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let field_type = self.parse_type()?;
            let field_name = self.expect_ident()?;
            self.expect(&Token::Semicolon)?;

            fields.push(VarDecl {
                name: field_name,
                var_type: field_type,
                init: None,
            });
        }

        self.expect(&Token::RightBrace)?;
        self.expect(&Token::Semicolon)?;

        Ok(ClassDef { name, fields })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let base_type = match self.peek() {
            Some(Token::U8) => { self.advance(); Type::U8 }
            Some(Token::U16) => { self.advance(); Type::U16 }
            Some(Token::U32) => { self.advance(); Type::U32 }
            Some(Token::U64) => { self.advance(); Type::U64 }
            Some(Token::I8) => { self.advance(); Type::I8 }
            Some(Token::I16) => { self.advance(); Type::I16 }
            Some(Token::I32) => { self.advance(); Type::I32 }
            Some(Token::I64) => { self.advance(); Type::I64 }
            Some(Token::F64) => { self.advance(); Type::F64 }
            Some(Token::Bool) => { self.advance(); Type::Bool }
            Some(Token::Void) => { self.advance(); Type::Void }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Type::Custom(name)
            }
            _ => return Err(anyhow!("Expected type, got {:?}", self.peek())),
        };

        // Handle pointers
        let mut result = base_type;
        while self.match_token(&Token::Star) {
            result = Type::Pointer(Box::new(result));
        }

        // Handle arrays
        if self.match_token(&Token::LeftBracket) {
            let size = if !self.check(&Token::RightBracket) {
                if let Some(Token::IntLiteral(n)) = self.peek() {
                    let size = *n as usize;
                    self.advance();
                    Some(size)
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(&Token::RightBracket)?;
            result = Type::Array(Box::new(result), size);
        }

        Ok(result)
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(&Token::RightParen) {
            return Ok(params);
        }

        loop {
            let param_type = self.parse_type()?;
            let name = self.expect_ident()?;

            params.push(Param { name, param_type });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_block_contents(&mut self) -> Result<Block> {
        let mut stmts = Vec::new();

        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        // Return statement
        if self.match_token(&Token::Return) {
            let value = if !self.check(&Token::Semicolon) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&Token::Semicolon)?;
            return Ok(Stmt::Return(value));
        }

        // Break/Continue
        if self.match_token(&Token::Break) {
            self.expect(&Token::Semicolon)?;
            return Ok(Stmt::Break);
        }
        if self.match_token(&Token::Continue) {
            self.expect(&Token::Semicolon)?;
            return Ok(Stmt::Continue);
        }

        // If statement
        if self.match_token(&Token::If) {
            self.expect(&Token::LeftParen)?;
            let condition = self.parse_expr()?;
            self.expect(&Token::RightParen)?;
            self.expect(&Token::LeftBrace)?;
            let then_block = self.parse_block_contents()?;
            self.expect(&Token::RightBrace)?;

            let else_block = if self.match_token(&Token::Else) {
                self.expect(&Token::LeftBrace)?;
                let block = self.parse_block_contents()?;
                self.expect(&Token::RightBrace)?;
                Some(block)
            } else {
                None
            };

            return Ok(Stmt::If {
                condition,
                then_block,
                else_block,
            });
        }

        // While loop
        if self.match_token(&Token::While) {
            self.expect(&Token::LeftParen)?;
            let condition = self.parse_expr()?;
            self.expect(&Token::RightParen)?;
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_contents()?;
            self.expect(&Token::RightBrace)?;

            return Ok(Stmt::While { condition, body });
        }

        // For loop
        if self.match_token(&Token::For) {
            self.expect(&Token::LeftParen)?;

            let init = if !self.check(&Token::Semicolon) {
                Some(Box::new(self.parse_stmt()?))
            } else {
                self.advance(); // consume semicolon
                None
            };

            let condition = if !self.check(&Token::Semicolon) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&Token::Semicolon)?;

            let increment = if !self.check(&Token::RightParen) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&Token::RightParen)?;

            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_contents()?;
            self.expect(&Token::RightBrace)?;

            return Ok(Stmt::For {
                init,
                condition,
                increment,
                body,
            });
        }

        // Block
        if self.match_token(&Token::LeftBrace) {
            let block = self.parse_block_contents()?;
            self.expect(&Token::RightBrace)?;
            return Ok(Stmt::Block(block));
        }

        // Variable declaration or expression
        if self.is_type_token() {
            let var_type = self.parse_type()?;
            let name = self.expect_ident()?;

            let init = if self.match_token(&Token::Assign) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            self.expect(&Token::Semicolon)?;

            return Ok(Stmt::VarDecl(VarDecl {
                name,
                var_type,
                init,
            }));
        }

        // Expression statement
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_logical_or()?;

        if let Some(token) = self.peek() {
            let op = match token {
                Token::Assign => {
                    self.advance();
                    let value = self.parse_assignment()?;
                    return Ok(Expr::Assign {
                        target: Box::new(expr),
                        value: Box::new(value),
                    });
                }
                Token::PlusAssign => BinaryOp::AddAssign,
                Token::MinusAssign => BinaryOp::SubAssign,
                Token::StarAssign => BinaryOp::MulAssign,
                Token::SlashAssign => BinaryOp::DivAssign,
                Token::PercentAssign => BinaryOp::ModAssign,
                Token::AndAssign => BinaryOp::AndAssign,
                Token::OrAssign => BinaryOp::OrAssign,
                Token::XorAssign => BinaryOp::XorAssign,
                Token::LeftShiftAssign => BinaryOp::ShlAssign,
                Token::RightShiftAssign => BinaryOp::ShrAssign,
                _ => return Ok(expr),
            };

            self.advance();
            let right = self.parse_assignment()?;
            return Ok(Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&Token::LogicalOr) {
            let right = self.parse_logical_and()?;
            expr = Expr::Binary {
                op: BinaryOp::LogicalOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitwise_or()?;

        while self.match_token(&Token::LogicalAnd) {
            let right = self.parse_bitwise_or()?;
            expr = Expr::Binary {
                op: BinaryOp::LogicalAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(&Token::Pipe) {
            let right = self.parse_bitwise_xor()?;
            expr = Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(&Token::Caret) {
            let right = self.parse_bitwise_and()?;
            expr = Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&Token::Ampersand) {
            let right = self.parse_equality()?;
            expr = Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = if self.match_token(&Token::Equal) {
                BinaryOp::Eq
            } else if self.match_token(&Token::NotEqual) {
                BinaryOp::Ne
            } else {
                break;
            };

            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_shift()?;

        loop {
            let op = if self.match_token(&Token::Less) {
                BinaryOp::Lt
            } else if self.match_token(&Token::LessEqual) {
                BinaryOp::Le
            } else if self.match_token(&Token::Greater) {
                BinaryOp::Gt
            } else if self.match_token(&Token::GreaterEqual) {
                BinaryOp::Ge
            } else {
                break;
            };

            let right = self.parse_shift()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expr> {
        let mut expr = self.parse_additive()?;

        loop {
            let op = if self.match_token(&Token::LeftShift) {
                BinaryOp::Shl
            } else if self.match_token(&Token::RightShift) {
                BinaryOp::Shr
            } else {
                break;
            };

            let right = self.parse_additive()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut expr = self.parse_multiplicative()?;

        loop {
            let op = if self.match_token(&Token::Plus) {
                BinaryOp::Add
            } else if self.match_token(&Token::Minus) {
                BinaryOp::Sub
            } else {
                break;
            };

            let right = self.parse_multiplicative()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = if self.match_token(&Token::Star) {
                BinaryOp::Mul
            } else if self.match_token(&Token::Slash) {
                BinaryOp::Div
            } else if self.match_token(&Token::Percent) {
                BinaryOp::Mod
            } else {
                break;
            };

            let right = self.parse_unary()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        let op = if self.match_token(&Token::Minus) {
            Some(UnaryOp::Neg)
        } else if self.match_token(&Token::LogicalNot) {
            Some(UnaryOp::Not)
        } else if self.match_token(&Token::Tilde) {
            Some(UnaryOp::BitNot)
        } else if self.match_token(&Token::Increment) {
            Some(UnaryOp::PreIncrement)
        } else if self.match_token(&Token::Decrement) {
            Some(UnaryOp::PreDecrement)
        } else if self.match_token(&Token::Star) {
            Some(UnaryOp::Deref)
        } else if self.match_token(&Token::Ampersand) {
            Some(UnaryOp::AddressOf)
        } else {
            None
        };

        if let Some(op) = op {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(expr),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LeftParen) {
                // Function call
                let mut args = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightParen)?;
                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                };
            } else if self.match_token(&Token::LeftBracket) {
                // Array index
                let index = self.parse_expr()?;
                self.expect(&Token::RightBracket)?;
                expr = Expr::Index {
                    expr: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::Dot) {
                // Member access
                let member = self.expect_ident()?;
                expr = Expr::Member {
                    expr: Box::new(expr),
                    member,
                };
            } else if self.match_token(&Token::Arrow) {
                // Pointer member access
                let member = self.expect_ident()?;
                expr = Expr::Arrow {
                    expr: Box::new(expr),
                    member,
                };
            } else if self.match_token(&Token::Increment) {
                expr = Expr::Unary {
                    op: UnaryOp::PostIncrement,
                    expr: Box::new(expr),
                };
            } else if self.match_token(&Token::Decrement) {
                expr = Expr::Unary {
                    op: UnaryOp::PostDecrement,
                    expr: Box::new(expr),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek() {
            Some(Token::IntLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            Some(Token::HexLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            Some(Token::BinLiteral(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            Some(Token::FloatLiteral(f)) => {
                let f = *f;
                self.advance();
                Ok(Expr::FloatLiteral(f))
            }
            Some(Token::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            Some(Token::CharLiteral(c)) => {
                let c = *c;
                self.advance();
                Ok(Expr::CharLiteral(c))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr::Null)
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name))
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RightParen)?;
                Ok(expr)
            }
            Some(Token::Sizeof) => {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let typ = self.parse_type()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::Sizeof(typ))
            }
            _ => Err(anyhow!("Unexpected token in primary expression: {:?}", self.peek())),
        }
    }

    // Helper methods
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn check(&self, token: &Token) -> bool {
        if let Some(current) = self.peek() {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: &Token) -> Result<()> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(anyhow!(
                "Expected {:?}, got {:?}",
                token,
                self.peek()
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        if let Some(Token::Ident(name)) = self.peek() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(anyhow!("Expected identifier, got {:?}", self.peek()))
        }
    }

    fn is_type_token(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::U8)
                | Some(Token::U16)
                | Some(Token::U32)
                | Some(Token::U64)
                | Some(Token::I8)
                | Some(Token::I16)
                | Some(Token::I32)
                | Some(Token::I64)
                | Some(Token::F64)
                | Some(Token::Bool)
                | Some(Token::Void)
                | Some(Token::Ident(_))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_function() {
        let source = "U64 add(U64 a, U64 b) { return a + b; }";
        let tokens = Lexer::collect_tokens(source).unwrap();
        let tokens: Vec<Token> = tokens.into_iter().map(|(t, _)| t).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.items.len(), 1);
        if let Item::FunctionDef(func) = &program.items[0] {
            assert_eq!(func.name, "add");
            assert_eq!(func.params.len(), 2);
        } else {
            panic!("Expected function definition");
        }
    }

    #[test]
    fn test_parse_class() {
        let source = "class Point { U64 x; U64 y; };";
        let tokens = Lexer::collect_tokens(source).unwrap();
        let tokens: Vec<Token> = tokens.into_iter().map(|(t, _)| t).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.items.len(), 1);
        if let Item::ClassDef(class) = &program.items[0] {
            assert_eq!(class.name, "Point");
            assert_eq!(class.fields.len(), 2);
        } else {
            panic!("Expected class definition");
        }
    }

    #[test]
    fn test_parse_xor_expression() {
        let source = "U64 test() { return x ^ 0xdeadbeef; }";
        let tokens = Lexer::collect_tokens(source).unwrap();
        let tokens: Vec<Token> = tokens.into_iter().map(|(t, _)| t).collect();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        if let Item::FunctionDef(func) = &program.items[0] {
            if let Stmt::Return(Some(Expr::Binary { op, .. })) = &func.body[0] {
                assert_eq!(*op, BinaryOp::BitXor);
            } else {
                panic!("Expected XOR expression in return");
            }
        }
    }
}
