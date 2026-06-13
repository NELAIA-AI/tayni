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
#[derive(Debug, Clone, PartialEq)]
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
