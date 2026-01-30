#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Program(Vec<AstNode>),
    
    // Declarations
    FunctionDecl { 
        return_type: String, 
        name: String, 
        args: Vec<(String, String)>, 
        body: Box<AstNode>, 
        doc_string: Option<String> 
    },
    StructDecl { 
        name: String, 
        fields: Vec<(String, String)>, 
        doc_string: Option<String> 
    },
    VarDecl { 
        type_name: String, 
        name: String, 
        value: Option<Box<AstNode>> 
    },
    ArrayDecl { 
        type_name: String, 
        name: String, 
        size: usize, 
        values: Option<Vec<AstNode>> 
    },

    // Statements
    Block(Vec<AstNode>),
    Assignment { target: Box<AstNode>, value: Box<AstNode> },
    ReturnStmt(Box<AstNode>),
    IfStmt { 
        condition: Box<AstNode>, 
        then_branch: Box<AstNode>, 
        else_branch: Option<Box<AstNode>> 
    },
    ForStmt {
        init: Box<AstNode>,
        condition: Box<AstNode>,
        increment: Box<AstNode>,
        body: Box<AstNode>,
    },
    BreakStmt,

    // Expressions
    BinaryOp { 
        left: Box<AstNode>, 
        op: BinaryOperator, 
        right: Box<AstNode> 
    },
    UnaryOp { 
        op: UnaryOperator, 
        right: Box<AstNode> 
    },
    Call { 
        func_name: String, 
        args: Vec<AstNode> 
    },
    SubscriptAccess { 
        base: Box<AstNode>, 
        index: Box<AstNode> 
    },
    MemberAccess { 
        base: Box<AstNode>, 
        member: String 
    },
    
    // Literals
    LiteralFloat(f64),
    LiteralInt(i64),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div,
    Equal, Less, Greater,
    LessEqual,    // <=
    GreaterEqual, // >=
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate, // -x
    Not,    // !x
}
