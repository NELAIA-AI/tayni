//! TAYNI Interface Generation
//! Generates UI interfaces for different surfaces (web, native, terminal)

use crate::ir::{Graph, Node, Value, Arg, Op};

#[derive(Clone, Copy, PartialEq)]
pub enum Surface {
    Web,      // HTML/CSS/JS via WASM
    Native,   // Native GUI (Win32, Cocoa, GTK)
    Terminal, // TUI (terminal UI)
}

#[derive(Clone)]
pub struct InterfaceSpec {
    pub surface: Surface,
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub components: Vec<Component>,
}

#[derive(Clone)]
pub enum Component {
    Text { x: u32, y: u32, content: String },
    Button { x: u32, y: u32, width: u32, height: u32, label: String, action: String },
    Input { x: u32, y: u32, width: u32, height: u32, placeholder: String, id: String },
    List { x: u32, y: u32, width: u32, height: u32, items: Vec<String> },
    Container { x: u32, y: u32, width: u32, height: u32, children: Vec<Component> },
}

impl InterfaceSpec {
    pub fn new(surface: Surface, title: &str) -> Self {
        InterfaceSpec {
            surface,
            width: 800,
            height: 600,
            title: title.to_string(),
            components: Vec::new(),
        }
    }
    
    pub fn add_text(&mut self, x: u32, y: u32, content: &str) {
        self.components.push(Component::Text { x, y, content: content.to_string() });
    }
    
    pub fn add_button(&mut self, x: u32, y: u32, w: u32, h: u32, label: &str, action: &str) {
        self.components.push(Component::Button { 
            x, y, width: w, height: h, 
            label: label.to_string(), 
            action: action.to_string() 
        });
    }
    
    pub fn add_input(&mut self, x: u32, y: u32, w: u32, h: u32, placeholder: &str, id: &str) {
        self.components.push(Component::Input { 
            x, y, width: w, height: h, 
            placeholder: placeholder.to_string(), 
            id: id.to_string() 
        });
    }
}

/// Generate HTML for web surface
pub fn generate_html(spec: &InterfaceSpec) -> String {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str(&format!("  <title>{}</title>\n", spec.title));
    html.push_str("  <style>\n");
    html.push_str("    body { font-family: system-ui; margin: 0; padding: 20px; }\n");
    html.push_str("    .component { position: absolute; }\n");
    html.push_str("    button { cursor: pointer; padding: 8px 16px; }\n");
    html.push_str("    input { padding: 8px; border: 1px solid #ccc; }\n");
    html.push_str("  </style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str(&format!("  <div style=\"width:{}px;height:{}px;position:relative;\">\n", 
        spec.width, spec.height));
    
    for comp in &spec.components {
        match comp {
            Component::Text { x, y, content } => {
                html.push_str(&format!(
                    "    <span class=\"component\" style=\"left:{}px;top:{}px;\">{}</span>\n",
                    x, y, content
                ));
            }
            Component::Button { x, y, width, height, label, action } => {
                html.push_str(&format!(
                    "    <button class=\"component\" style=\"left:{}px;top:{}px;width:{}px;height:{}px;\" onclick=\"{}\">{}</button>\n",
                    x, y, width, height, action, label
                ));
            }
            Component::Input { x, y, width, height, placeholder, id } => {
                html.push_str(&format!(
                    "    <input class=\"component\" id=\"{}\" style=\"left:{}px;top:{}px;width:{}px;height:{}px;\" placeholder=\"{}\">\n",
                    id, x, y, width, height, placeholder
                ));
            }
            _ => {}
        }
    }
    
    html.push_str("  </div>\n");
    html.push_str("</body>\n</html>\n");
    
    html
}

/// Generate TAYNI code for native GUI (Windows)
pub fn generate_native_tayni(spec: &InterfaceSpec) -> String {
    let mut code = String::new();
    
    code.push_str("-- TAYNI Native GUI\n");
    code.push_str("-- Auto-generated interface\n\n");
    
    code.push_str(&format!(".title: \"{}\"\n", spec.title));
    code.push_str(&format!(".width: {}\n", spec.width));
    code.push_str(&format!(".height: {}\n", spec.height));
    code.push_str(".win: WIN .width .height .title\n\n");
    
    for (i, comp) in spec.components.iter().enumerate() {
        match comp {
            Component::Text { x, y, content } => {
                code.push_str(&format!(".txt{}: \"{}\"\n", i, content));
                code.push_str(&format!(".lbl{}: LBL .win {} {} 200 20 .txt{}\n", i, x, y, i));
            }
            Component::Button { x, y, width, height, label, .. } => {
                code.push_str(&format!(".btn_txt{}: \"{}\"\n", i, label));
                code.push_str(&format!(".btn{}: BTN .win {} {} {} {} .btn_txt{}\n", 
                    i, x, y, width, height, i));
            }
            Component::Input { x, y, width, height, id, .. } => {
                code.push_str(&format!(".{}: EDT .win {} {} {} {}\n", id, x, y, width, height));
            }
            _ => {}
        }
    }
    
    code.push_str("\n.loop: RUN .win\n!\n");
    
    code
}

/// Generate TAYNI code for terminal UI
pub fn generate_terminal_tayni(spec: &InterfaceSpec) -> String {
    let mut code = String::new();
    
    code.push_str("-- TAYNI Terminal UI\n");
    code.push_str("-- Auto-generated interface\n\n");
    
    // Clear screen
    code.push_str(".clear: \"\\x1B[2J\\x1B[H\"\n");
    code.push_str(".out_clear: PRT .clear 7\n\n");
    
    // Title
    code.push_str(&format!(".title: \"=== {} ===\\n\"\n", spec.title));
    code.push_str(&format!(".out_title: PRT .title {}\n\n", spec.title.len() + 9));
    
    for (i, comp) in spec.components.iter().enumerate() {
        match comp {
            Component::Text { content, .. } => {
                code.push_str(&format!(".txt{}: \"{}\\n\"\n", i, content));
                code.push_str(&format!(".out{}: PRT .txt{} {}\n", i, i, content.len() + 1));
            }
            Component::Button { label, .. } => {
                code.push_str(&format!(".btn{}: \"[{}]\\n\"\n", i, label));
                code.push_str(&format!(".out_btn{}: PRT .btn{} {}\n", i, i, label.len() + 3));
            }
            Component::Input { placeholder, id, .. } => {
                code.push_str(&format!(".prompt{}: \"{}: \"\n", i, placeholder));
                code.push_str(&format!(".out_prompt{}: PRT .prompt{} {}\n", i, i, placeholder.len() + 2));
                code.push_str(&format!(".{}_buf: ALC 256\n", id));
                code.push_str(&format!(".{}: INP .{}_buf 256\n", id, id));
            }
            _ => {}
        }
    }
    
    code.push_str("\n!\n");
    
    code
}

/// Parse interface specification from TAYNI graph
pub fn parse_interface_from_graph(graph: &Graph) -> Option<InterfaceSpec> {
    let mut spec = InterfaceSpec::new(Surface::Web, "TAYNI App");
    
    for node in &graph.nodes {
        if let Node::Operation { op: Op::Win, args, .. } = node {
            if args.len() >= 3 {
                if let (Arg::Lit(Value::Int(w)), Arg::Lit(Value::Int(h))) = (&args[0], &args[1]) {
                    spec.width = *w as u32;
                    spec.height = *h as u32;
                }
                if let Arg::Lit(Value::String(title)) = &args[2] {
                    spec.title = title.clone();
                }
            }
        }
    }
    
    Some(spec)
}
