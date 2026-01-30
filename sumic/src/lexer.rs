use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")] // Skip whitespace
pub enum Token {
    // --- Keywords ---
    #[token("struct")]
    Struct,
    #[token("fn")]     // Future-proofing for S2L, though Swift used implicit func
    Fn, 
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,

    #[token("for")] For,
    #[token("break")] Break,
    #[token("!")] Bang, 

    // --- Symbols ---
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token(";")] Semicolon,
    #[token(":")] Colon,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token("=")] Equals,

    // --- Operators ---
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token(">")] Greater,
    #[token("<")] Less,
    #[token("==")] DoubleEquals,

    // --- Literals & Identifiers ---
    
    // Matches alphanumeric identifiers (must start with letter or _)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Matches numbers (integers or floats)
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_string())]
    Number(String),

    // --- Comments ---

    // Doc Comments (///): We want to keep these for the AST
    #[regex(r"///.*", |lex| lex.slice().trim_start_matches("///").trim().to_string())]
    DocComment(String),

    // Standard Comments (//): We skip these entirely
    #[regex(r"//[^/].*", logos::skip)] // Matches // followed by not /, then anything
    #[regex(r"//", logos::skip)]       // Matches empty //
    Comment,
}

// Helper for tests to see what the lexer produced
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_lexing() {
        let input = "struct Material { vec3 color; }";
        let tokens: Vec<_> = Token::lexer(input).collect();
        
        assert_eq!(tokens[0], Ok(Token::Struct));
        assert_eq!(tokens[1], Ok(Token::Identifier("Material".to_string())));
        assert_eq!(tokens[2], Ok(Token::LBrace));
        assert_eq!(tokens[3], Ok(Token::Identifier("vec3".to_string())));
    }

    #[test]
    fn test_doc_comments() {
        let input = "/// This is a doc\n// This is hidden\nstruct";
        let tokens: Vec<_> = Token::lexer(input).collect();
        
        match &tokens[0] {
            Ok(Token::DocComment(s)) => assert_eq!(s, "This is a doc"),
            _ => panic!("Expected DocComment"),
        }
        assert_eq!(tokens[1], Ok(Token::Struct)); // The // comment was skipped
    }
}
