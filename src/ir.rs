//! NELAIA v0.4 Intermediate Representation
//! Data Flow Graph representation

use std::collections::{HashMap, HashSet};

/// A node in the data flow graph
#[derive(Debug, Clone)]
pub enum Node {
    /// Literal value: .id: 42 or .id: "string"
    Literal { id: String, value: Value },
    
    /// Reference to another node: .id: .other
    Reference { id: String, target: String },
    
    /// Operation: .id: OP .args
    Operation { id: String, op: Op, args: Vec<Arg> },
    
    /// Sub-graph definition: .id: { IN ... OUT }
    SubGraph { id: String, inputs: Vec<String>, outputs: Vec<String>, nodes: Vec<Node> },
    
    /// Flow: .source > .dest or .source > EFFECT
    Flow { source: Arg, dest: FlowDest },
    
    /// Label (for sub-graph references)
    Label(String),
    
    /// Function definition: .id: FUN .param { body } !FUN
    Function { id: String, params: Vec<String>, body: Vec<Node> },
}

/// A value (literal)
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Pair(Box<Value>, Box<Value>),
}

/// An argument to an operation
#[derive(Debug, Clone)]
pub enum Arg {
    /// Reference to a node: .id
    Ref(String),
    /// Literal value
    Lit(Value),
    /// Grouped expression: (OP .args)
    Expr(Op, Vec<Arg>),
}

/// Operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Neg,
    // Comparison
    Eq, Ne, Lt, Gt, Le, Ge,
    // Logic
    And, Or, Not,
    // Collections
    Seq, Map, Fld, Flt, Len, Fst, Snd,
    // Control (BRN only - LOOP/BRK/CNT deprecated per Consortium 2026-06-13)
    Brn,
    // Conditional Jump (for real loops)
    Jmp,  // JMP cond label_true label_false -> conditional branch
    // While loop
    Whl,  // WHL cond body_label -> while loop
    // Effects (I/O)
    Prt, Inp, Opn, Acc, Get, Put, Cls, Err,
    // Network
    Tcp, Udp, Bnd, Lst, Con, Xmt, Rcv,
    // Async I/O
    Sel, Rdy, Nbk,
    // Socket options (ultra-low-latency)
    Ndl, Qck, Sbf, Kal,
    // High-performance I/O
    Epl, Ewa, Ect,
    // Threading
    Thr, Jon, Mtx, Lck, Ulk,
    // Atomic Queue (lock-free)
    Que,  // QUE capacity -> queue_ptr (creates atomic queue)
    Psh,  // PSH queue value -> success (atomic push)
    Pop,  // POP queue -> value (atomic pop, 0 if empty)
    // Functions
    Ret,
    // Memory
    Alc, Fre,
    // Memory operations (self-hosting)
    Cpy,  // CPY src dst len -> copy bytes
    Cmp,  // CMP a b len -> -1/0/1 comparison
    Fnd,  // FND buf len byte -> position or -1
    Sln,  // SLN ptr -> string length (until \0)
    // File I/O (self-hosting)
    Fop,  // FOP path mode -> file_handle (mode: 0=read, 1=write, 2=append)
    Frd,  // FRD handle buf len -> bytes_read
    Fwr,  // FWR handle buf len -> bytes_written
    Fcl,  // FCL handle -> close file
    // Dynamic Vectors (AI-native data structures)
    Vec,  // VEC capacity -> vec_ptr (creates growable vector)
    Vph,  // VPH vec value -> push value to vector
    Vgt,  // VGT vec index -> value at index
    Vst,  // VST vec index value -> set value at index
    Vln,  // VLN vec -> current length
    Vcp,  // VCP vec -> capacity
    // HashMap (self-hosting - symbol tables)
    Hmp,  // HMP capacity -> hashmap_ptr
    Hpt,  // HPT map key_ptr key_len value -> put key-value
    Hgt,  // HGT map key_ptr key_len -> value (0 if not found)
    Hhs,  // HHS map key_ptr key_len -> 1 if exists, 0 if not
    // String operations (self-hosting - code generation)
    Cat,  // CAT dst src1 src2 -> concatenate strings to dst
    Its,  // ITS num buf -> int to string, returns length
    Chr,  // CHR str index -> char code at index
    Sbs,  // SBS dst src start len -> substring
    // Error handling
    Chk,
    // GUI - Window Management
    Win,  // WIN width height title -> window_handle
    Shw,  // SHW handle -> show window
    Hid,  // HID handle -> hide window
    Evt,  // EVT handle -> poll events, returns event_type
    Run,  // RUN handle -> run message loop until window closes
    // GUI - Controls
    Lbl,  // LBL parent x y width height text -> label_handle
    Txb,  // TXB parent x y width height -> textbox_handle
    Btn,  // BTN parent x y width height text onclick_flag_ptr -> button_handle
    // GUI - Dialogs
    Dlg,  // DLG title message -> show alert dialog
    // GUI - Control Values
    Gvl,  // GVL control_handle buffer len -> get text value
    Svl,  // SVL control_handle text -> set text value
    // Sub-graph call
    Call(String),
}

/// Flow destination
#[derive(Debug, Clone)]
pub enum FlowDest {
    /// Flow to another node (single execution)
    Node(String),
    /// Cyclic flow to another node (intentional repetition)
    CyclicNode(String),
    /// Flow to an effect
    Effect(Effect),
}

/// Side effects
#[derive(Debug, Clone)]
pub enum Effect {
    Print,
    OpenNet(i32),
    OpenDsk(String),
    Accept,
    Get,
    Put,
    Close,
}

/// Resource type for OPN
#[derive(Debug, Clone)]
pub enum ResourceType {
    Net,
    Dsk,
}

/// The complete graph
#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub node_map: HashMap<String, usize>,
}

/// Analysis results
#[derive(Debug)]
pub struct GraphAnalysis {
    pub cycles: Vec<Vec<String>>,
    pub dead_nodes: Vec<String>,
    pub undefined_refs: Vec<String>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            node_map: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: Node) {
        if let Some(id) = Self::get_node_id(&node) {
            self.node_map.insert(id, self.nodes.len());
        }
        self.nodes.push(node);
    }
    
    fn get_node_id(node: &Node) -> Option<String> {
        match node {
            Node::Literal { id, .. } => Some(id.clone()),
            Node::Reference { id, .. } => Some(id.clone()),
            Node::Operation { id, .. } => Some(id.clone()),
            Node::SubGraph { id, .. } => Some(id.clone()),
            _ => None,
        }
    }
    
    /// Analyze the graph for cycles, dead nodes, and undefined references
    pub fn analyze(&self) -> GraphAnalysis {
        let mut analysis = GraphAnalysis {
            cycles: Vec::new(),
            dead_nodes: Vec::new(),
            undefined_refs: Vec::new(),
        };
        
        // Build dependency graph
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
        let mut used: HashSet<String> = HashSet::new();
        let mut defined: HashSet<String> = HashSet::new();
        
        for node in &self.nodes {
            match node {
                Node::Literal { id, .. } => {
                    defined.insert(id.clone());
                    deps.insert(id.clone(), Vec::new());
                }
                Node::Reference { id, target } => {
                    defined.insert(id.clone());
                    deps.insert(id.clone(), vec![target.clone()]);
                    used.insert(target.clone());
                }
                Node::Operation { id, args, .. } => {
                    defined.insert(id.clone());
                    let mut node_deps = Vec::new();
                    for arg in args {
                        Self::collect_refs(arg, &mut node_deps, &mut used);
                    }
                    deps.insert(id.clone(), node_deps);
                }
                Node::Flow { source, dest } => {
                    Self::collect_refs(source, &mut Vec::new(), &mut used);
                    if let FlowDest::Node(target) = dest {
                        used.insert(target.clone());
                    }
                }
                Node::Function { id, params: _, body: _ } => {
                    // Functions are defined names
                    defined.insert(id.clone());
                    deps.insert(id.clone(), Vec::new());
                }
                _ => {}
            }
        }
        
        // Find undefined references
        for ref_name in &used {
            if !defined.contains(ref_name) {
                analysis.undefined_refs.push(ref_name.clone());
            }
        }
        
        // Find dead nodes (defined but never used, except in flows)
        for def_name in &defined {
            if !used.contains(def_name) {
                analysis.dead_nodes.push(def_name.clone());
            }
        }
        
        // Detect cycles using DFS
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        
        for node_id in deps.keys() {
            if !visited.contains(node_id) {
                let mut path = Vec::new();
                if self.detect_cycle(node_id, &deps, &mut visited, &mut rec_stack, &mut path) {
                    analysis.cycles.push(path);
                }
            }
        }
        
        analysis
    }
    
    fn collect_refs(arg: &Arg, deps: &mut Vec<String>, used: &mut HashSet<String>) {
        match arg {
            Arg::Ref(name) => {
                deps.push(name.clone());
                used.insert(name.clone());
            }
            Arg::Expr(_, inner_args) => {
                for inner in inner_args {
                    Self::collect_refs(inner, deps, used);
                }
            }
            _ => {}
        }
    }
    
    fn detect_cycle(
        &self,
        node: &str,
        deps: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(neighbors) = deps.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.detect_cycle(neighbor, deps, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    path.push(neighbor.clone());
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        path.pop();
        false
    }
}

/// Usage analysis for dead code elimination
#[derive(Debug, Default)]
pub struct UsageAnalysis {
    pub used_ops: HashSet<Op>,
    pub uses_gui: bool,
    pub uses_network: bool,
    pub uses_threading: bool,
    pub uses_file_io: bool,
    pub uses_console: bool,
}

impl UsageAnalysis {
    pub fn analyze(graph: &Graph) -> Self {
        let mut analysis = UsageAnalysis::default();
        
        for node in &graph.nodes {
            analysis.visit_node(node);
        }
        
        analysis
    }
    
    fn visit_node(&mut self, node: &Node) {
        match node {
            Node::Operation { op, args: _, id: _ } => {
                self.used_ops.insert(op.clone());
                self.categorize_op(op);
            }
            Node::Function { body, .. } => {
                for body_node in body {
                    self.visit_node(body_node);
                }
            }
            _ => {}
        }
    }
    
    fn categorize_op(&mut self, op: &Op) {
        match op {
            // Network operations
            Op::Tcp | Op::Udp | Op::Bnd | Op::Lst | Op::Acc | 
            Op::Con | Op::Xmt | Op::Rcv | Op::Sel | Op::Rdy | 
            Op::Nbk | Op::Ndl | Op::Qck | Op::Sbf | Op::Kal |
            Op::Epl | Op::Ewa | Op::Ect => {
                self.uses_network = true;
            }
            // Threading operations
            Op::Thr | Op::Jon | Op::Mtx | Op::Lck | Op::Ulk |
            Op::Que | Op::Psh | Op::Pop => {
                self.uses_threading = true;
            }
            // GUI operations
            Op::Win | Op::Shw | Op::Hid | Op::Evt | Op::Run |
            Op::Lbl | Op::Txb | Op::Btn | Op::Dlg | Op::Gvl | Op::Svl => {
                self.uses_gui = true;
            }
            // Memory operations
            Op::Alc | Op::Fre | Op::Cpy | Op::Cmp | Op::Fnd | Op::Sln => {
                self.uses_file_io = true;  // Uses memory syscalls
            }
            // File I/O
            Op::Fop | Op::Frd | Op::Fwr | Op::Fcl => {
                self.uses_file_io = true;
            }
            // Dynamic Vectors
            Op::Vec | Op::Vph | Op::Vgt | Op::Vst | Op::Vln | Op::Vcp => {
                self.uses_file_io = true;  // Uses memory allocation
            }
            // HashMap
            Op::Hmp | Op::Hpt | Op::Hgt | Op::Hhs => {
                self.uses_file_io = true;  // Uses memory allocation
            }
            // String operations
            Op::Cat | Op::Its | Op::Chr | Op::Sbs => {
                self.uses_file_io = true;  // Uses memory
            }
            // Console I/O
            Op::Prt | Op::Inp | Op::Err => {
                self.uses_console = true;
            }
            _ => {}
        }
    }
}
