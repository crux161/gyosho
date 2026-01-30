use crate::ast::{AstNode, BinaryOperator};

pub trait CodeGenerator {
    fn generate(&self, ast: &AstNode) -> String;
}

// --- Metal Shading Language (MSL) Generator ---

pub struct MetalGenerator {
    pub is_std_lib: bool, // Fixed: Bool -> bool
}

impl MetalGenerator {
    pub fn new(is_std_lib: bool) -> Self {
        Self { is_std_lib }
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
        }
    }
}

impl CodeGenerator for MetalGenerator {
    fn generate(&self, ast: &AstNode) -> String {
        match ast {
            AstNode::Program(nodes) => {
                nodes.iter()
                    .map(|n| self.generate(n))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n\n")
            },

            AstNode::StructDecl { name, fields, .. } => {
                let field_str = fields.iter()
                    .map(|(typ, n)| format!("    {} {};", typ, n))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("struct {} {{\n{}\n}};", name, field_str)
            },

            AstNode::FunctionDecl { return_type, name, args, body, .. } => {
                if self.is_std_lib {
                    return format!("// [Native Symbol] {}", name);
                }
                let arg_str = args.iter()
                    .map(|(typ, n)| format!("{} {}", typ, n))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {}({}) {}", return_type, name, arg_str, self.generate(body))
            },

            AstNode::Block(stmts) => {
                let inner = stmts.iter()
                    .map(|s| format!("    {}", self.generate(s)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{{\n{}\n}}", inner)
            },

            AstNode::ReturnStmt(expr) => {
                format!("return {};", self.generate(expr))
            },

            AstNode::IfStmt { condition, then_branch, else_branch } => {
                let mut code = format!("if ({}) {}", self.generate(condition), self.generate(then_branch));
                if let Some(else_b) = else_branch {
                    code.push_str(&format!(" else {}", self.generate(else_b)));
                }
                code
            },

            AstNode::VarDecl { type_name, name, value } => {
                if let Some(val) = value {
                    format!("{} {} = {};", type_name, name, self.generate(val))
                } else {
                    format!("{} {};", type_name, name)
                }
            },

            AstNode::ArrayDecl { type_name, name, size, values } => {
                let mut init_str = String::new();
                if let Some(vals) = values {
                    let v_str = vals.iter().map(|v| self.generate(v)).collect::<Vec<_>>().join(", ");
                    init_str = format!(" = {{ {} }}", v_str);
                }
                format!("{} {}[{}]{};", type_name, name, size, init_str)
            },

            AstNode::Assignment { target, value } => {
                format!("{} = {};", self.generate(target), self.generate(value))
            },

            AstNode::BinaryOp { left, op, right } => {
                format!("({} {} {})", self.generate(left), self.generate_op(op), self.generate(right))
            },

            AstNode::Call { func_name, args } => {
                let arg_str = args.iter().map(|a| self.generate(a)).collect::<Vec<_>>().join(", ");
                format!("{}({})", func_name, arg_str)
            },

            AstNode::MemberAccess { base, member } => {
                format!("{}.{}", self.generate(base), member)
            },

            AstNode::SubscriptAccess { base, index } => {
                format!("{}[{}]", self.generate(base), self.generate(index))
            },

            AstNode::LiteralFloat(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f) 
                } else {
                    format!("{}", f)
                }
            },
            AstNode::LiteralInt(i) => format!("{}", i),
            AstNode::Variable(name) => name.clone(),
        }
    }
}

// --- WGSL Generator (New!) ---

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
            "mat4"  => "mat4x4<f32>".to_string(), 
            "void"  => "".to_string(),
            _ => t.to_string(), // Custom structs pass through
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
        }
    }
}

impl CodeGenerator for WgslGenerator {
    fn generate(&self, ast: &AstNode) -> String {
        match ast {
            AstNode::Program(nodes) => {
                nodes.iter()
                    .map(|n| self.generate(n))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n\n")
            },

            AstNode::StructDecl { name, fields, .. } => {
                let field_str = fields.iter()
                    .map(|(typ, n)| format!("    {}: {},", n, self.map_type(typ)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("struct {} {{\n{}\n}};", name, field_str)
            },

            AstNode::FunctionDecl { return_type, name, args, body, .. } => {
                let mapped_ret = self.map_type(return_type);
                let ret_str = if mapped_ret.is_empty() { "".to_string() } else { format!("-> {}", mapped_ret) };
                
                let arg_str = args.iter()
                    .map(|(typ, n)| format!("{}: {}", n, self.map_type(typ)))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                format!("fn {}({}) {} {}", name, arg_str, ret_str, self.generate(body))
            },

            AstNode::Block(stmts) => {
                let inner = stmts.iter()
                    .map(|s| format!("    {}", self.generate(s)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{{\n{}\n}}", inner)
            },

            AstNode::ReturnStmt(expr) => format!("return {};", self.generate(expr)),

            AstNode::IfStmt { condition, then_branch, else_branch } => {
                let mut code = format!("if ({}) {}", self.generate(condition), self.generate(then_branch));
                if let Some(else_b) = else_branch {
                    code.push_str(&format!(" else {}", self.generate(else_b)));
                }
                code
            },

            AstNode::VarDecl { type_name, name, value } => {
                let mapped_type = self.map_type(type_name);
                if let Some(val) = value {
                    format!("var {}: {} = {};", name, mapped_type, self.generate(val))
                } else {
                    format!("var {}: {};", name, mapped_type)
                }
            },

            // Arrays in WGSL need strict sizing, e.g., var a: array<f32, 4>;
            AstNode::ArrayDecl { type_name, name, size, values } => {
                let mapped_type = self.map_type(type_name);
                let type_str = format!("array<{}, {}>", mapped_type, size);
                
                if let Some(vals) = values {
                    let v_str = vals.iter().map(|v| self.generate(v)).collect::<Vec<_>>().join(", ");
                    format!("var {}: {} = {}({});", name, type_str, type_str, v_str)
                } else {
                    format!("var {}: {};", name, type_str)
                }
            },

            AstNode::Assignment { target, value } => {
                format!("{} = {};", self.generate(target), self.generate(value))
            },

            AstNode::BinaryOp { left, op, right } => {
                format!("({} {} {})", self.generate(left), self.generate_op(op), self.generate(right))
            },

            // WGSL specific: built-ins often match GLSL, but type constructors are explicit
            AstNode::Call { func_name, args } => {
                let arg_str = args.iter().map(|a| self.generate(a)).collect::<Vec<_>>().join(", ");
                // If the function name is a type (vec3), WGSL handles it as vec3<f32>(...)
                if ["vec2", "vec3", "vec4"].contains(&func_name.as_str()) {
                     format!("{}<f32>({})", func_name, arg_str)
                } else {
                     format!("{}({})", func_name, arg_str)
                }
            },

            AstNode::MemberAccess { base, member } => format!("{}.{}", self.generate(base), member),
            AstNode::SubscriptAccess { base, index } => format!("{}[{}]", self.generate(base), self.generate(index)),
            
            AstNode::LiteralFloat(f) => {
                if f.fract() == 0.0 { format!("{:.1}", f) } else { format!("{}", f) }
            },
            AstNode::LiteralInt(i) => format!("{}", i),
            AstNode::Variable(name) => name.clone(),
        }
    }
}

// --- Markdown Documentation Generator ---

pub struct MarkdownGenerator;

impl CodeGenerator for MarkdownGenerator {
    fn generate(&self, ast: &AstNode) -> String {
        if let AstNode::Program(nodes) = ast {
            let docs: Vec<String> = nodes.iter()
                .filter_map(|n| self.generate_node_doc(n))
                .collect();
            
            if docs.is_empty() { return String::new(); }
            
            format!("# API Documentation\n\n{}", docs.join("\n---\n"))
        } else {
            String::new()
        }
    }
}

impl MarkdownGenerator {
    fn generate_node_doc(&self, ast: &AstNode) -> Option<String> {
        match ast {
            AstNode::FunctionDecl { return_type, name, args, doc_string, .. } => {
                let arg_list = args.iter()
                    .map(|(t, n)| format!("`{}` {}", t, n))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let mut md = format!("### `{}`\n", name);
                if let Some(doc) = doc_string {
                    md.push_str(&format!("> {}\n\n", doc));
                }
                md.push_str(&format!("- **Signature**: `{} {}({})`\n", return_type, name, arg_list));
                Some(md)
            },
            AstNode::StructDecl { name, fields, doc_string } => {
                let mut md = format!("### `struct {}`\n", name);
                if let Some(doc) = doc_string {
                    md.push_str(&format!("> {}\n\n", doc));
                }
                md.push_str("- **Fields**:\n");
                for (t, n) in fields {
                    md.push_str(&format!("  - `{}` {}\n", t, n));
                }
                Some(md)
            },
            _ => None,
        }
    }
}

// --- TESTS ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_metal_var_decl() {
        let node = AstNode::VarDecl { 
            type_name: "vec3".into(), 
            name: "pos".into(), 
            value: Some(Box::new(AstNode::LiteralFloat(1.0))) 
        };
        let gen = MetalGenerator::new(false);
        assert_eq!(gen.generate(&node), "vec3 pos = 1.0;");
    }

    #[test]
    fn test_metal_struct() {
        let node = AstNode::StructDecl {
            name: "Light".into(),
            fields: vec![("vec3".into(), "color".into())],
            doc_string: None
        };
        let gen = MetalGenerator::new(false);
        let output = gen.generate(&node);
        assert!(output.contains("struct Light {"));
        assert!(output.contains("vec3 color;"));
    }

    #[test]
    fn test_markdown_gen() {
        let node = AstNode::FunctionDecl {
            return_type: "void".into(),
            name: "main".into(),
            args: vec![],
            body: Box::new(AstNode::Block(vec![])),
            doc_string: Some("Entry point".into())
        };
        let gen = MarkdownGenerator;
        let output = gen.generate_node_doc(&node).unwrap();
        assert!(output.contains("### `main`"));
        assert!(output.contains("> Entry point"));
    }
}
