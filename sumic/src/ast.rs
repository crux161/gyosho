// The Abstract Syntax Tree for Sumi Shader Language (S2L)

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Program(Vec<AstNode>),
    
    // Top Level Declarations
    FunctionDecl {
        return_type: String,
        name: String,
        args: Vec<(String, String)>, // (Type, Name)
        body: Box<AstNode>,          // Usually a Block
        doc_string: Option<String>,
    },
    StructDecl {
        name: String,
        fields: Vec<(String, String)>, // (Type, Name)
        doc_string: Option<String>,
    },

    // Statements
    Block(Vec<AstNode>),
    VarDecl {
        type_name: String,
        name: String,
        value: Option<Box<AstNode>>,
    },
    ArrayDecl {
        type_name: String,
        name: String,
        size: usize,
        values: Option<Vec<AstNode>>,
    },
    Assignment {
        target: Box<AstNode>,
        value: Box<AstNode>,
    },
    ReturnStmt(Box<AstNode>),
    IfStmt {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },

    ForStmt {
        init: Box<AstNode>,      // e.g. int i = 0
        condition: Box<AstNode>, // e.g. i < 80
        increment: Box<AstNode>, // e.g. i = i + 1
        body: Box<AstNode>,
    },
    BreakStmt,
// --- EXPRESSIONS ---
    BinaryOp { left: Box<AstNode>, op: BinaryOperator, right: Box<AstNode> },
    
    // --- NEW UNARY ---
    UnaryOp { op: UnaryOperator, right: Box<AstNode> },

    Call { func_name: String, args: Vec<AstNode> },
    SubscriptAccess { base: Box<AstNode>, index: Box<AstNode> },
    MemberAccess { base: Box<AstNode>, member: String },
    LiteralFloat(f64),
    LiteralInt(i64),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div,
    Equal, Less, Greater,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate, // -x
    Not,    // !x
}
