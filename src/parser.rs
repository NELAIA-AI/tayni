use crate::ir::{NelaiaGraph, Opcode};

pub struct Parser;

impl Parser {
    pub fn parse(content: &str) -> Result<NelaiaGraph, String> {
        let mut graph = NelaiaGraph::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Zero-syntax split: just by spaces
            // Quotes might have spaces, so a simple split isn't perfect for strings,
            // but for a strict Metalanguage, we enforce strict formatting.
            let mut tokens: Vec<&str> = Vec::new();
            let mut current = String::new();
            let mut in_quotes = false;

            for c in line.chars() {
                if c == '"' {
                    in_quotes = !in_quotes;
                } else if c == ' ' && !in_quotes {
                    if !current.is_empty() {
                        tokens.push(Box::leak(current.clone().into_boxed_str()));
                        current.clear();
                    }
                } else {
                    current.push(c);
                }
            }
            if !current.is_empty() {
                tokens.push(Box::leak(current.clone().into_boxed_str()));
            }

            if tokens.is_empty() { continue; }

            let opcode_str = tokens[0];
            let op = match opcode_str {
                "LDC" => {
                    let ref_id = Self::parse_ref(tokens[1])?;
                    let module = tokens[2].to_string();
                    Opcode::LDC { ref_id, module_name: module }
                },
                "STR" => {
                    let ref_id = Self::parse_ref(tokens[1])?;
                    let val = tokens[2].to_string();
                    Opcode::STR { ref_id, value: val }
                },
                "OUT" => {
                    let buffer_ref = Self::parse_ref(tokens[1])?;
                    Opcode::OUT { buffer_ref }
                },
                "EXE" => {
                    let target = tokens.get(1).unwrap_or(&"native").to_string();
                    Opcode::EXE { target }
                },
                _ => return Err(format!("Unrecognized token '{}' on line {}", opcode_str, line_idx + 1)),
            };

            graph.push(op);
        }

        Ok(graph)
    }

    fn parse_ref(token: &str) -> Result<usize, String> {
        if !token.starts_with('#') {
            return Err(format!("Expected reference (starting with #), found '{}'", token));
        }
        token[1..].parse::<usize>().map_err(|_| format!("Invalid reference number: {}", token))
    }
}
