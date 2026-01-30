use crate::ast::{AstNode, BinaryOperator, UnaryOperator};

pub trait CodeGenerator {
    fn generate(&self, ast: &AstNode) -> String;
}

// --- Metal Generator ---

pub struct MetalGenerator { pub is_std_lib: bool }

impl MetalGenerator {
    pub fn new(is_std_lib: bool) -> Self { Self { is_std_lib } }

    fn generate_op(&self, op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+", 
            BinaryOperator::Sub => "-",
            BinaryOperator::Mul => "*", 
            BinaryOperator::Div => "/",
            BinaryOperator::Equal => "==", 
            BinaryOperator::Less => "<", 
            BinaryOperator::Greater => ">",
            BinaryOperator::LessEqual => "<=",    // <--- Added
            BinaryOperator::GreaterEqual => ">=", // <--- Added
        }
    }
}

impl CodeGenerator for MetalGenerator {
    fn generate(&self, ast: &AstNode) -> String {
        match ast {
            AstNode::Program(nodes) => nodes.iter().map(|n| self.generate(n)).collect::<Vec<_>>().join("\n\n"),
            
            AstNode::FunctionDecl { return_type, name, args, body, .. } => {
                let arg_str = args.iter().map(|(t,n)| format!("{} {}", t, n)).collect::<Vec<_>>().join(", ");
                format!("{} {}({}) {}", return_type, name, arg_str, self.generate(body))
            },
            
            AstNode::StructDecl { name, fields, .. } => {
                let f_str = fields.iter().map(|(t,n)| format!("    {} {};", t, n)).collect::<Vec<_>>().join("\n");
                format!("struct {} {{\n{}\n}};", name, f_str)
            },

            AstNode::Block(stmts) => format!("{{\n{}\n}}", stmts.iter().map(|s| format!("    {}", self.generate(s))).collect::<Vec<_>>().join("\n")),
            
            AstNode::ReturnStmt(expr) => format!("return {};", self.generate(expr)),
            
            AstNode::IfStmt { condition, then_branch, else_branch } => {
                let base = format!("if ({}) {}", self.generate(condition), self.generate(then_branch));
                if let Some(e) = else_branch { format!("{} else {}", base, self.generate(e)) } else { base }
            },

            AstNode::ForStmt { init, condition, increment, body } => {
                let i = self.generate(init);
                let c = self.generate(condition);
                let inc = self.generate(increment);
                format!("for ({} {}; {}) {}", i, c, inc.trim_end_matches(';'), self.generate(body))
            },

            AstNode::BreakStmt => "break;".to_string(),

            AstNode::VarDecl { type_name, name, value } => {
                if let Some(v) = value { format!("{} {} = {};", type_name, name, self.generate(v)) } 
                else { format!("{} {};", type_name, name) }
            },

            AstNode::ArrayDecl { type_name, name, size, values } => {
                let mut init_str = String::new();
                if let Some(vals) = values {
                    let v_str = vals.iter().map(|v| self.generate(v)).collect::<Vec<_>>().join(", ");
                    init_str = format!(" = {{ {} }}", v_str);
                }
                format!("{} {}[{}]{};", type_name, name, size, init_str)
            },

            AstNode::Assignment { target, value } => format!("{} = {};", self.generate(target), self.generate(value)),
            
            AstNode::BinaryOp { left, op, right } => format!("({} {} {})", self.generate(left), self.generate_op(op), self.generate(right)),
            
            AstNode::UnaryOp { op, right } => {
                let s = match op { UnaryOperator::Negate => "-", UnaryOperator::Not => "!" };
                format!("({}{})", s, self.generate(right))
            },

            AstNode::Call { func_name, args } => format!("{}({})", func_name, args.iter().map(|a| self.generate(a)).collect::<Vec<_>>().join(", ")),
            
            AstNode::MemberAccess { base, member } => format!("{}.{}", self.generate(base), member),
            AstNode::SubscriptAccess { base, index } => format!("{}[{}]", self.generate(base), self.generate(index)),
            
            AstNode::LiteralFloat(f) => if f.fract() == 0.0 { format!("{:.1}", f) } else { format!("{}", f) },
            AstNode::LiteralInt(i) => format!("{}", i),
            AstNode::Variable(n) => n.clone(),
        }
    }
}

// --- WGSL Generator ---

pub struct WgslGenerator;

impl WgslGenerator {
    pub fn new() -> Self { Self }

    fn map_type(&self, t: &str) -> String {
        match t {
            "float" => "f32".to_string(),
            "int"   => "i32".to_string(),
            "uint"  => "u32".to_string(),
            "bool"  => "bool".to_string(),
            "vec2"  => "vec2<f32>".to_string(),
            "vec3"  => "vec3<f32>".to_string(),
            "vec4"  => "vec4<f32>".to_string(),
            "mat2"  => "mat2x2<f32>".to_string(),
            "mat3"  => "mat3x3<f32>".to_string(),
            "mat4"  => "mat4x4<f32>".to_string(),
            "void"  => "".to_string(),
            _ => t.to_string(),
        }
    }

    fn generate_op(&self, op: &BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+", 
            BinaryOperator::Sub => "-",
            BinaryOperator::Mul => "*", 
            BinaryOperator::Div => "/",
            BinaryOperator::Equal => "==", 
            BinaryOperator::Less => "<", 
            BinaryOperator::Greater => ">",
            BinaryOperator::LessEqual => "<=",    // <--- Added
            BinaryOperator::GreaterEqual => ">=", // <--- Added
        }
    }
}

impl CodeGenerator for WgslGenerator {
    fn generate(&self, ast: &AstNode) -> String {
        match ast {
            AstNode::Program(nodes) => nodes.iter().map(|n| self.generate(n)).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n\n"),

            AstNode::StructDecl { name, fields, .. } => {
                let f_str = fields.iter().map(|(t,n)| format!("    {}: {},", n, self.map_type(t))).collect::<Vec<_>>().join("\n");
                format!("struct {} {{\n{}\n}};", name, f_str)
            },

            AstNode::FunctionDecl { return_type, name, args, body, .. } => {
                let ret = self.map_type(return_type);
                let ret_str = if ret.is_empty() { "".to_string() } else { format!("-> {}", ret) };
                let arg_str = args.iter().map(|(t,n)| format!("{}: {}", n, self.map_type(t))).collect::<Vec<_>>().join(", ");
                format!("fn {}({}) {} {}", name, arg_str, ret_str, self.generate(body))
            },

            AstNode::Block(stmts) => {
                let inner = stmts.iter().map(|s| format!("    {}", self.generate(s))).collect::<Vec<_>>().join("\n");
                format!("{{\n{}\n}}", inner)
            },

            AstNode::ReturnStmt(expr) => format!("return {};", self.generate(expr)),

            AstNode::IfStmt { condition, then_branch, else_branch } => {
                let base = format!("if ({}) {}", self.generate(condition), self.generate(then_branch));
                if let Some(e) = else_branch { format!("{} else {}", base, self.generate(e)) } else { base }
            },

            AstNode::ForStmt { init, condition, increment, body } => {
                let i = self.generate(init);
                let c = self.generate(condition);
                let inc = self.generate(increment);
                format!("for ({} {}; {}) {}", i, c, inc.trim_end_matches(';'), self.generate(body))
            },

            AstNode::BreakStmt => "break;".to_string(),

            AstNode::VarDecl { type_name, name, value } => {
                let t = self.map_type(type_name);
                if let Some(v) = value { format!("var {}: {} = {};", name, t, self.generate(v)) }
                else { format!("var {}: {};", name, t) }
            },

            AstNode::ArrayDecl { type_name, name, size, values } => {
                let t = self.map_type(type_name);
                let t_arr = format!("array<{}, {}>", t, size);
                if let Some(vals) = values {
                    let v_str = vals.iter().map(|v| self.generate(v)).collect::<Vec<_>>().join(", ");
                    format!("var {}: {} = {}({});", name, t_arr, t_arr, v_str)
                } else { format!("var {}: {};", name, t_arr) }
            },

            AstNode::Assignment { target, value } => format!("{} = {};", self.generate(target), self.generate(value)),

            AstNode::BinaryOp { left, op, right } => format!("({} {} {})", self.generate(left), self.generate_op(op), self.generate(right)),

            AstNode::UnaryOp { op, right } => {
                let s = match op { UnaryOperator::Negate => "-", UnaryOperator::Not => "!" };
                format!("({}{})", s, self.generate(right))
            },

            AstNode::Call { func_name, args } => {
                let arg_str = args.iter().map(|a| self.generate(a)).collect::<Vec<_>>().join(", ");
                match func_name.as_str() {
                    "vec2" | "vec3" | "vec4" => format!("{}<f32>({})", func_name, arg_str),
                    "mat2" => format!("mat2x2<f32>({})", arg_str),
                    "mat3" => format!("mat3x3<f32>({})", arg_str),
                    "mat4" => format!("mat4x4<f32>({})", arg_str),
                    "float" => format!("f32({})", arg_str),
                    "int" => format!("i32({})", arg_str),
                    "uint" => format!("u32({})", arg_str),
                    _ => format!("{}({})", func_name, arg_str)
                }
            },

            AstNode::MemberAccess { base, member } => format!("{}.{}", self.generate(base), member),
            AstNode::SubscriptAccess { base, index } => format!("{}[{}]", self.generate(base), self.generate(index)),
            
            AstNode::LiteralFloat(f) => if f.fract() == 0.0 { format!("{:.1}", f) } else { format!("{}", f) },
            AstNode::LiteralInt(i) => format!("{}", i),
            
            AstNode::Variable(name) => match name.as_str() {
                "iTime" => "u.time".to_string(),
                "iResolution" => "vec3<f32>(u.resolution, 1.0)".to_string(),
                "iMouse" => "u.mouse".to_string(),
                _ => name.clone(),
            },
        }
    }
}

pub struct MarkdownGenerator;
impl CodeGenerator for MarkdownGenerator { fn generate(&self, _: &AstNode) -> String { String::new() } }
