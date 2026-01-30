use crate::ast::{AstNode, BinaryOperator, UnaryOperator};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn current(&self) -> Option<&Token> { self.tokens.get(self.cursor) }
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.cursor + 1) }
    fn advance(&mut self) { if self.cursor < self.tokens.len() { self.cursor += 1; } }
    
    fn check(&self, token: &Token) -> bool { self.current() == Some(token) }

    fn consume(&mut self, expected: Token) -> Result<(), String> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            let got = self.current().map(|t| format!("{:?}", t)).unwrap_or("EOF".to_string());
            Err(format!("Expected {:?}, got {}", expected, got))
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, String> {
        let mut nodes = Vec::new();
        while self.current().is_some() {
            nodes.push(self.parse_top_level()?);
        }
        Ok(AstNode::Program(nodes))
    }

    fn parse_top_level(&mut self) -> Result<AstNode, String> {
        // Handle Doc Comments
        let mut doc_string = None;
        if let Some(Token::DocComment(s)) = self.current() {
            doc_string = Some(s.clone());
            self.advance();
            while let Some(Token::DocComment(s2)) = self.current() {
                doc_string = Some(format!("{}\n{}", doc_string.unwrap(), s2));
                self.advance();
            }
        }

        if self.check(&Token::Struct) {
            return self.parse_struct(doc_string);
        }

        // Function Declaration
        // S2L: fn Name(...)
        if self.check(&Token::Fn) {
            self.advance();
            let name = match self.current() {
                Some(Token::Identifier(s)) => s.clone(),
                _ => return Err("Expected Function Name".to_string()),
            };
            self.advance();

            self.consume(Token::LParen)?;
            let args = self.parse_args()?;
            self.consume(Token::RParen)?;

            let return_type = match self.current() {
                Some(Token::Identifier(s)) => { let t = s.clone(); self.advance(); t },
                Some(Token::LBrace) => "void".to_string(),
                _ => return Err("Expected Return Type".to_string()),
            };

            self.consume(Token::LBrace)?;
            let body = self.parse_block()?;

            return Ok(AstNode::FunctionDecl { return_type, name, args, body: Box::new(body), doc_string });
        }

        // Legacy C-Style Function: Type Name(...)
        let type_name = match self.current() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected Function or Struct".to_string()),
        };
        self.advance();

        let name = match self.current() {
            Some(Token::Identifier(s)) => s.clone(),
            _ => return Err("Expected Name".to_string()),
        };
        self.advance();

        self.consume(Token::LParen)?;
        let args = self.parse_args()?;
        self.consume(Token::RParen)?;
        self.consume(Token::LBrace)?;
        let body = self.parse_block()?;

        Ok(AstNode::FunctionDecl { return_type: type_name, name, args, body: Box::new(body), doc_string })
    }

    fn parse_struct(&mut self, doc_string: Option<String>) -> Result<AstNode, String> {
        self.consume(Token::Struct)?;
        let name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected struct name".to_string()) };
        self.advance();
        self.consume(Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && self.current().is_some() {
            let type_name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected field type".to_string()) };
            self.advance();
            let field_name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected field name".to_string()) };
            self.advance();
            self.consume(Token::Semicolon)?;
            fields.push((type_name, field_name));
        }
        self.consume(Token::RBrace)?;
        self.consume(Token::Semicolon)?; 
        Ok(AstNode::StructDecl { name, fields, doc_string })
    }

    fn parse_args(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut args = Vec::new();
        while !self.check(&Token::RParen) {
            // Skip qualifiers
            if let Some(Token::Identifier(s)) = self.current() {
                if s == "in" || s == "out" || s == "inout" { self.advance(); }
            }

            let first = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected arg ident".to_string()) };
            self.advance();

            // S2L: Name : Type
            if self.check(&Token::Colon) {
                self.advance();
                let type_name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected type".to_string()) };
                self.advance();
                args.push((type_name, first));
            } else {
                // C-Style: Type Name
                let name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected arg name".to_string()) };
                self.advance();
                args.push((first, name));
            }
            if self.check(&Token::Comma) { self.advance(); }
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
        if self.check(&Token::LBrace) {
            self.advance();
            return self.parse_block();
        }
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
            return Ok(AstNode::IfStmt { condition: Box::new(condition), then_branch: Box::new(then_branch), else_branch });
        }
        if self.check(&Token::Return) {
            self.advance();
            let expr = self.parse_expression()?;
            self.consume(Token::Semicolon)?;
            return Ok(AstNode::ReturnStmt(Box::new(expr)));
        }
        if self.check(&Token::Break) {
            self.advance();
            self.consume(Token::Semicolon)?;
            return Ok(AstNode::BreakStmt);
        }
        if self.check(&Token::For) {
            self.advance();
            self.consume(Token::LParen)?;
            let init = self.parse_statement()?; 
            let condition = self.parse_expression()?;
            self.consume(Token::Semicolon)?;
            let increment = self.parse_expression_assignment()?;
            self.consume(Token::RParen)?;
            let body = self.parse_statement()?;
            return Ok(AstNode::ForStmt { init: Box::new(init), condition: Box::new(condition), increment: Box::new(increment), body: Box::new(body) });
        }

        // Variable Declaration
        if let Some(Token::Identifier(id)) = self.current() {
            // S2L: var name : type = val;
            if id == "var" {
                if let Some(Token::Identifier(_)) = self.peek() {
                    self.advance(); // eat var
                    let name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected name".to_string()) };
                    self.advance();
                    self.consume(Token::Colon)?;
                    let type_name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected type".to_string()) };
                    self.advance();
                    
                    let mut value = None;
                    if self.check(&Token::Equals) {
                        self.advance();
                        value = Some(Box::new(self.parse_expression()?));
                    }
                    self.consume(Token::Semicolon)?;
                    return Ok(AstNode::VarDecl { type_name, name, value });
                }
            }
            
            // C-Style: Type Name = val;
            if let Some(Token::Identifier(_)) = self.peek() {
                // If next token is Identifier, assume Type Name pattern
                // But check it's not a function call or assignment
                if !self.check_next(&Token::LParen) && !self.check_next(&Token::Equals) && !self.check_next(&Token::Dot) {
                     let type_name = id.clone();
                     self.advance();
                     let name = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected name".to_string()) };
                     self.advance();
                     
                     let mut value = None;
                     if self.check(&Token::Equals) {
                        self.advance();
                        value = Some(Box::new(self.parse_expression()?));
                     }
                     self.consume(Token::Semicolon)?;
                     return Ok(AstNode::VarDecl { type_name, name, value });
                }
            }
        }

        // Fallback: Expr or Assignment
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

    fn check_next(&self, token: &Token) -> bool {
        self.peek() == Some(token)
    }

    fn parse_expression_assignment(&mut self) -> Result<AstNode, String> {
        let expr = self.parse_expression()?;
        if self.check(&Token::Equals) {
            self.advance();
            let value = self.parse_expression()?;
            return Ok(AstNode::Assignment { target: Box::new(expr), value: Box::new(value) });
        }
        Ok(expr)
    }

    fn parse_expression(&mut self) -> Result<AstNode, String> { self.parse_comparison() }
    
    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_math()?;
        loop {
            let op = match self.current() {
                Some(Token::Greater) => BinaryOperator::Greater,
                Some(Token::Less) => BinaryOperator::Less,
                Some(Token::DoubleEquals) => BinaryOperator::Equal,
                Some(Token::LessEqual) => BinaryOperator::LessEqual,       
                Some(Token::GreaterEqual) => BinaryOperator::GreaterEqual,
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
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.current() {
                Some(Token::Star) => BinaryOperator::Mul,
                Some(Token::Slash) => BinaryOperator::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<AstNode, String> {
        if self.check(&Token::Minus) {
            self.advance();
            let right = self.parse_unary()?;
            return Ok(AstNode::UnaryOp { op: UnaryOperator::Negate, right: Box::new(right) });
        }
        if self.check(&Token::Bang) {
            self.advance();
            let right = self.parse_unary()?;
            return Ok(AstNode::UnaryOp { op: UnaryOperator::Not, right: Box::new(right) });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<AstNode, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&Token::LParen) {
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
            } else if self.check(&Token::Dot) {
                self.advance();
                let member = match self.current() { Some(Token::Identifier(s)) => s.clone(), _ => return Err("Expected member".to_string()) };
                self.advance();
                expr = AstNode::MemberAccess { base: Box::new(expr), member };
            } else if self.check(&Token::LBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.consume(Token::RBracket)?;
                expr = AstNode::SubscriptAccess { base: Box::new(expr), index: Box::new(index) };
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
                if n.contains('.') { Ok(AstNode::LiteralFloat(n.parse().unwrap_or(0.0))) }
                else { Ok(AstNode::LiteralInt(n.parse().unwrap_or(0))) }
            },
            Some(Token::Identifier(s)) => { let n = s.clone(); self.advance(); Ok(AstNode::Variable(n)) },
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
