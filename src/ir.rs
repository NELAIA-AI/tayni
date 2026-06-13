#[derive(Debug, Clone)]
pub enum Opcode {
    LDC { ref_id: usize, module_name: String }, // LOAD_CRATE
    STR { ref_id: usize, value: String },       // STATIC_LOAD
    OUT { buffer_ref: usize },                  // SYS_ECHO
    EXE { target: String },                     // BUILD_STANDALONE
}

#[derive(Debug)]
pub struct NelaiaGraph {
    pub nodes: Vec<Opcode>,
}

impl NelaiaGraph {
    pub fn new() -> Self {
        NelaiaGraph { nodes: Vec::new() }
    }

    pub fn push(&mut self, op: Opcode) {
        self.nodes.push(op);
    }
}
