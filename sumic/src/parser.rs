use crate::ast::{AstNode, BinaryOperator};
use crate::lexer::Token;

/// The Recursive Descent Parser
pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    // --- Helpers ---

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor + 1)
    }

    fn advance(&mut self) {
        if self.cursor < self.tokens.len() {
            self.cursor += 1;
        }
    }

    fn check(&self, token: &Token) -> bool {
        self.current() == Some(token)
    }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            let got = self.current().map(|t| format!("{:?}", t)).unwrap_or("EOF".to_string());
            Err(format!("Expected {:?}, got {}", expected, got))
        }
    }

    // --- Parsing Logic ---

    pub fn parse(&mut self) -> Result<AstNode, String> {
        let mut nodes = Vec::new();
        while self.current().is_some() {
            nodes.push(self.parse_top_level()?);
        }
        Ok(AstNode::Program(nodes))
    }

    fn parse_top_level(&mut self) -> Result<AstNode, String> {
        // 1. Handle Doc Comments
        let mut doc_string = None;
        if let Some(Token::DocComment(s)) = self.current() {
            doc_string = Some(s.clone());
            self.advance();
            while let Some(Token::DocComment(s2)) = self.current() {
                doc_string = Some(format!("{}\n{}", doc_string.unwrap(), s2));
                self.advance();
            }
        }

        // 2. Struct Declaration
        if self.check(&Token::Struct) {
            return self.parse_struct(doc_string);
        }

        // 3. Function Declaration (S2L Style): "fn Name(Args) Type { ... }"
        if self.check(&Token::Fn) {
            self.advance(); // Eat 'fn'
            
            let name = match self.current() {
                Some(Token::Identifier(s)) => s.clone(),
                _ => return Err("Expected Function Name after 'fn'".to_string()),
            };
            self.advance();

            self.consume(Token::LParen)?;
            let args = self.parse_args()?;
            self.consume(Token::RParen)?;

            // Parse Return Type (it comes AFTER args in S2L)
            // e.g. fn main(...) vec4 { ... }
            let return_type = match self.current() {
                Some(Token::Identifier(s)) => {
                    let t = s.clone();
                    self.advance();
                    t
                },
                Some(Token::LBrace) => "void".to_string(), // Implicit void if no type
                _ => return Err("Expected Return Type or Body".to_string()),
            };

            self.consume(Token::LBrace)?;
            let body = self.parse_block()?;

            return Ok(AstNode::FunctionDecl {
                return_type,
                name,
                args,
                body: Box::new(body),
                doc_string,
            });
        }

        // 4. Function Declaration (Legacy C-Style): "Type Name(Args) { ... }"
        // Fallback for compatibility or if user didn't write 'fn'
        let type_name = match self.current() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected Function Return Type or Struct".to_string()),
        };
        self.advance();

        let name = match self.current() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected Function Name".to_string()),
        };
        self.advance();

        self.consume(Token::LParen)?;
        let args = self.parse_args()?;
        self.consume(Token::RParen)?;
        
        self.consume(Token::LBrace)?;
        let body = self.parse_block()?;

        Ok(AstNode::FunctionDecl {
            return_type: type_name,
            name,
            args,
            body: Box::new(body),
            doc_string,
        })
    }

    fn parse_struct(&mut self, doc_string: Option<String>) -> Result<AstNode, String> {
        self.consume(Token::Struct)?;
        
        let name = match self.current() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected struct name".to_string()),
        };
        self.advance();

        self.consume(Token::LBrace)?;

        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && self.current().is_some() {
            // Field: Type Name ;
            let type_name = match self.current() {
                Some(Token::Identifier(s)) => s.clone(),
                _ => return Err("Expected field type".to_string()),
            };
            self.advance();

            let field_name = match self.current() {
                Some(Token::Identifier(s)) => s.clone(),
                _ => return Err("Expected field name".to_string()),
            };
            self.advance();

            self.consume(Token::Semicolon)?;
            fields.push((type_name, field_name));
        }

        self.consume(Token::RBrace)?;
        self.consume(Token::Semicolon)?; 

        Ok(AstNode::StructDecl {
            name,
            fields,
            doc_string,
        })
    }

fn parse_args(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut args = Vec::new();
        while !self.check(&Token::RParen) {
            
            // 1. Skip GLSL Qualifiers
            if let Some(Token::Identifier(s)) = self.current() {
                if s == "in" || s == "out" || s == "inout" {
                    self.advance(); 
                }
            }

            // 2. Read First Identifier (Could be Type OR Name)
            let first_ident = match self.current() {
                Some(Token::Identifier(s)) => s.clone(),
                _ => return Err("Expected argument identifier".to_string()),
            };
            self.advance();

            // 3. Check for Colon to determine style
            if self.check(&Token::Colon) {
                // Style: Name: Type (S2L / Rust)
                self.advance(); // Eat ':'
                
                let type_name = match self.current() {
                    Some(Token::Identifier(s)) => s.clone(),
                    _ => return Err("Expected type after colon".to_string()),
                };
                self.advance();
                
                // Handle Array Syntax for S2L: name: type[]
                if self.check(&Token::LBracket) {
                     // ... (omitted for brevity, same logic as before if needed)
                }

                args.push((type_name, first_ident)); // (Type, Name)

            } else {
                // Style: Type Name (GLSL / C)
                // first_ident was the Type. Now read the Name.
                let param_name = match self.current() {
                    Some(Token::Identifier(s)) => s.clone(),
                    _ => return Err("Expected arg name".to_string()),
                };
                self.advance();

                // Handle Array Syntax for GLSL: type name[N]
                if self.check(&Token::LBracket) {
                    self.advance();
                    if let Some(Token::Number(_)) = self.current() { self.advance(); }
                    self.consume(Token::RBracket)?;
                    args.push((first_ident, format!("{}[]", param_name)));
                } else {
                    args.push((first_ident, param_name));
                }
            }

            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        Ok(args)
    }
    
    fn parse_block(&mut self) -> Result<AstNode, String> {
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && self.current().is_some() {
            statements.push(self.parse_statement()?);
        }
        self.consume(Token::RBrace)?;
        Ok(AstNode::Block(statements))
    }

    fn parse_statement(&mut self) -> Result<AstNode, String> {
        // 1. Nested Block
        if self.check(&Token::LBrace) {
            self.advance();
            return self.parse_block();
        }

        // 2. If
        if self.check(&Token::If) {
            self.advance();
            self.consume(Token::LParen)?;
            let condition = self.parse_expression()?;
            self.consume(Token::RParen)?;
            let then_branch = self.parse_statement()?;
            
            let mut else_branch = None;
            if self.check(&Token::Else) {
                self.advance();
                else_branch = Some(Box::new(self.parse_statement()?));
            }
            return Ok(AstNode::IfStmt {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            });
        }

        // 3. Return
        if self.check(&Token::Return) {
            self.advance();
            let expr = self.parse_expression()?;
            self.consume(Token::Semicolon)?;
            return Ok(AstNode::ReturnStmt(Box::new(expr)));
        }

        // 4. Variable Decl or Assignment or Expression
        if let Some(Token::Identifier(t_name)) = self.current() {
            if let Some(Token::Identifier(v_name)) = self.peek() {
                // It is a Variable Declaration: Type Name ...
                let type_name = t_name.clone();
                let var_name = v_name.clone();
                self.advance(); // eat type
                self.advance(); // eat name

                // Array Decl
                if self.check(&Token::LBracket) {
                    self.advance();
                    let size = match self.current() {
                        Some(Token::Number(n)) => n.parse::<usize>().unwrap_or(0),
                        _ => 0, 
                    };
                    self.advance();
                    self.consume(Token::RBracket)?;
                    
                    let mut values = None;
                    if self.check(&Token::Equals) {
                        self.advance();
                        self.consume(Token::LBrace)?;
                        let mut vals = Vec::new();
                        while !self.check(&Token::RBrace) {
                            vals.push(self.parse_expression()?);
                            if self.check(&Token::Comma) { self.advance(); }
                        }
                        self.consume(Token::RBrace)?;
                        values = Some(vals);
                    }
                    self.consume(Token::Semicolon)?;
                    return Ok(AstNode::ArrayDecl { type_name, name: var_name, size, values });
                }

                // Normal Var Decl
                let mut value = None;
                if self.check(&Token::Equals) {
                    self.advance();
                    value = Some(Box::new(self.parse_expression()?));
                }
                self.consume(Token::Semicolon)?;
                return Ok(AstNode::VarDecl { type_name, name: var_name, value });
            }
        }

        // Fallback: Expression or Assignment
        let expr = self.parse_expression()?;
        if self.check(&Token::Equals) {
            self.advance();
            let value = self.parse_expression()?;
            self.consume(Token::Semicolon)?;
            return Ok(AstNode::Assignment { target: Box::new(expr), value: Box::new(value) });
        }
        
        self.consume(Token::Semicolon)?;
        Ok(expr)
    }

    // --- Expression Parsing ---

    fn parse_expression(&mut self) -> Result<AstNode, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_math()?;
        loop {
            let op = match self.current() {
                Some(Token::Greater) => BinaryOperator::Greater,
                Some(Token::Less) => BinaryOperator::Less,
                Some(Token::DoubleEquals) => BinaryOperator::Equal,
                _ => break,
            };
            self.advance();
            let right = self.parse_math()?;
            left = AstNode::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_math(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.current() {
                Some(Token::Plus) => BinaryOperator::Add,
                Some(Token::Minus) => BinaryOperator::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = AstNode::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_postfix()?;
        loop {
            let op = match self.current() {
                Some(Token::Star) => BinaryOperator::Mul,
                Some(Token::Slash) => BinaryOperator::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_postfix()?;
            left = AstNode::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_postfix(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&Token::LParen) {
                // Call
                self.advance();
                let mut args = Vec::new();
                while !self.check(&Token::RParen) {
                    args.push(self.parse_expression()?);
                    if self.check(&Token::Comma) { self.advance(); }
                }
                self.consume(Token::RParen)?;
                if let AstNode::Variable(name) = expr {
                    expr = AstNode::Call { func_name: name, args };
                } else {
                    return Err("Expected identifier before call".to_string());
                }
            } else if self.check(&Token::LBracket) {
                // Subscript
                self.advance();
                let index = self.parse_expression()?;
                self.consume(Token::RBracket)?;
                expr = AstNode::SubscriptAccess { base: Box::new(expr), index: Box::new(index) };
            } else if self.check(&Token::Dot) {
                // Member
                self.advance();
                let member = match self.current() {
                    Some(Token::Identifier(s)) => s.clone(),
                    _ => return Err("Expected member name".to_string()),
                };
                self.advance();
                expr = AstNode::MemberAccess { base: Box::new(expr), member };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<AstNode, String> {
        match self.current() {
            Some(Token::Number(s)) => {
                let n = s.clone();
                self.advance();
                if n.contains('.') {
                    Ok(AstNode::LiteralFloat(n.parse().unwrap_or(0.0)))
                } else {
                    Ok(AstNode::LiteralInt(n.parse().unwrap_or(0)))
                }
            },
            Some(Token::Identifier(s)) => {
                let name = s.clone();
                self.advance();
                Ok(AstNode::Variable(name))
            },
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(Token::RParen)?;
                Ok(expr)
            },
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None => Err("Unexpected EOF".to_string()),
        }
    }
}
