//! TAYNI v0.7 Intermediate Representation
//! Data Flow Graph representation with Capability System
//! Includes: Contracts, Guarantees, Negotiation (IA-first design)

use std::collections::{HashMap, HashSet};

/// Capability requirement for the program
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    // Core capabilities
    Math,       // ADD, SUB, MUL, DIV, MOD
    Memory,     // ALC, FRE, CPY, etc.
    IO,         // PRT, INP, file operations
    
    // Network capabilities
    HttpServer,
    HttpClient,
    TcpRaw,
    
    // Data capabilities
    Json,
    Xml,
    
    // Database capabilities
    Sql,
    SqlReadOnly,
    
    // System capabilities
    FileSystem,
    FileSystemReadOnly,
    Threading,
    Gui,
    
    // Custom capability
    Custom(String),
}

/// Resource guarantee (IA-first: not permissions, but guarantees)
#[derive(Debug, Clone, PartialEq)]
pub enum Guarantee {
    Available,              // Resource is available
    Latency(u64),          // Max latency in ms
    Bandwidth(u64),        // Bandwidth in bytes/sec
    Uptime(f64),           // Uptime ratio (0.0-1.0)
    Deterministic,         // Same input = same output
    NoPersistence,         // Won't persist data
    NoExfiltration,        // Won't send data externally
}

/// Resource limit
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceLimit {
    pub memory_bytes: Option<u64>,
    pub time_ms: Option<u64>,
    pub connections: Option<u32>,
    pub cpu_percent: Option<u8>,
}

impl Default for ResourceLimit {
    fn default() -> Self {
        Self {
            memory_bytes: None,
            time_ms: None,
            connections: None,
            cpu_percent: None,
        }
    }
}

/// Contract for resource usage (IA-first: replaces permissions)
#[derive(Debug, Clone, Default)]
pub struct Contract {
    pub guarantees: HashMap<Capability, Vec<Guarantee>>,
    pub limits: ResourceLimit,
    pub trust_level: TrustLevel,
}

/// Trust level for code execution
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TrustLevel {
    Full,       // Own code: full access
    #[default]
    Standard,   // Verified code: standard capabilities
    Minimal,    // Untrusted code: sandboxed
}

/// Capability offer (for negotiation)
#[derive(Debug, Clone, Default)]
pub struct CapabilityOffer {
    pub capabilities: HashSet<Capability>,
    pub guarantees: HashMap<Capability, Vec<Guarantee>>,
    pub cost_per_use: f64,
}

/// Capability requirement (for negotiation)
#[derive(Debug, Clone, Default)]
pub struct CapabilityNeed {
    pub capabilities: HashSet<Capability>,
    pub constraints: HashMap<Capability, Vec<Guarantee>>,
    pub max_budget: f64,
}

/// Custom capability definition
#[derive(Debug, Clone)]
pub struct CustomCapability {
    pub name: String,
    pub inputs: Vec<(String, String)>,  // (name, type)
    pub pattern: Vec<Node>,             // Implementation pattern
    pub provides: HashSet<String>,      // What it provides
    pub requires: HashSet<Capability>,  // What it needs
}

/// Test property (IA-first: property-based testing)
#[derive(Debug, Clone)]
pub struct TestProperty {
    pub name: String,
    pub variables: Vec<String>,
    pub assertion: Box<Node>,
}

/// Capability requirements block
#[derive(Debug, Clone, Default)]
pub struct Requirements {
    pub capabilities: HashSet<Capability>,
    pub contract: Contract,
    pub offers: Vec<CapabilityOffer>,
    pub needs: Vec<CapabilityNeed>,
    pub custom_capabilities: HashMap<String, CustomCapability>,
    pub test_properties: Vec<TestProperty>,
}

/// Cache entry for incremental compilation (IA-first: Content-Addressable)
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub hash: u64,
    pub ir: String,
    pub dependencies: Vec<u64>,
    pub signature: Option<Vec<u8>>,  // Optional cryptographic signature
}

/// Incremental compilation cache (IA-first: Content-Addressable Store)
#[derive(Debug, Default)]
pub struct CompilationCache {
    pub entries: HashMap<u64, CacheEntry>,
    pub hits: u64,
    pub misses: u64,
}

/// Capability cost information
#[derive(Debug, Clone, Default)]
pub struct CapabilityCost {
    pub memory_bytes: u64,
    pub time_ms: u64,
    pub tokens: u64,
}

/// Capability metadata for SEN registry (IA-first: ecosystem)
#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub guarantees: Vec<String>,      // Constitutional contracts
    pub cost: CapabilityCost,         // Efficiency metrics
    pub dependencies: Vec<String>,
    pub regions: Vec<String>,         // Regionalization (empty = global)
    pub keywords: Vec<String>,        // For DISCOVER search
}

/// SEN Registry - Sistema de Ecosistema TAYNI (IA-first: federated)
#[derive(Debug, Default)]
pub struct EcosystemRegistry {
    pub capabilities: HashMap<String, CapabilityMetadata>,
    pub local_node_id: String,
}

impl EcosystemRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            capabilities: HashMap::new(),
            local_node_id: "local".to_string(),
        };
        // Register built-in capabilities
        registry.register_builtins();
        registry
    }
    
    fn register_builtins(&mut self) {
        // HTTP capability
        self.capabilities.insert("http".to_string(), CapabilityMetadata {
            name: "http".to_string(),
            version: "0.1.0".to_string(),
            description: "HTTP server and client operations".to_string(),
            guarantees: vec!["network_access".to_string()],
            cost: CapabilityCost { memory_bytes: 1024 * 1024, time_ms: 100, tokens: 0 },
            dependencies: vec![],
            regions: vec![],  // Global
            keywords: vec!["http".to_string(), "web".to_string(), "rest".to_string(), "api".to_string()],
        });
        
        // SQL capability
        self.capabilities.insert("sql".to_string(), CapabilityMetadata {
            name: "sql".to_string(),
            version: "0.1.0".to_string(),
            description: "SQL database operations via ODBC".to_string(),
            guarantees: vec!["database_access".to_string()],
            cost: CapabilityCost { memory_bytes: 2 * 1024 * 1024, time_ms: 50, tokens: 0 },
            dependencies: vec![],
            regions: vec![],
            keywords: vec!["sql".to_string(), "database".to_string(), "db".to_string(), "query".to_string()],
        });
        
        // JSON capability
        self.capabilities.insert("json".to_string(), CapabilityMetadata {
            name: "json".to_string(),
            version: "0.1.0".to_string(),
            description: "JSON parsing and encoding".to_string(),
            guarantees: vec!["deterministic".to_string(), "no_side_effects".to_string()],
            cost: CapabilityCost { memory_bytes: 512 * 1024, time_ms: 10, tokens: 0 },
            dependencies: vec![],
            regions: vec![],
            keywords: vec!["json".to_string(), "parse".to_string(), "encode".to_string(), "data".to_string()],
        });
        
        // Threading capability
        self.capabilities.insert("threading".to_string(), CapabilityMetadata {
            name: "threading".to_string(),
            version: "0.1.0".to_string(),
            description: "Multi-threading and synchronization".to_string(),
            guarantees: vec!["concurrent_execution".to_string()],
            cost: CapabilityCost { memory_bytes: 64 * 1024, time_ms: 1, tokens: 0 },
            dependencies: vec![],
            regions: vec![],
            keywords: vec!["thread".to_string(), "parallel".to_string(), "concurrent".to_string(), "async".to_string()],
        });
        
        // GUI capability
        self.capabilities.insert("gui".to_string(), CapabilityMetadata {
            name: "gui".to_string(),
            version: "0.1.0".to_string(),
            description: "Graphical user interface".to_string(),
            guarantees: vec!["user_interaction".to_string()],
            cost: CapabilityCost { memory_bytes: 10 * 1024 * 1024, time_ms: 16, tokens: 0 },
            dependencies: vec![],
            regions: vec![],
            keywords: vec!["gui".to_string(), "window".to_string(), "ui".to_string(), "interface".to_string(), "button".to_string()],
        });
    }
    
    /// DISCOVER - Find capabilities matching description (GPT's dynamic discovery)
    pub fn discover(&self, query: &str) -> Vec<&CapabilityMetadata> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        let mut results: Vec<(&CapabilityMetadata, usize)> = self.capabilities.values()
            .map(|cap| {
                let mut score = 0;
                for word in &query_words {
                    if cap.name.to_lowercase().contains(word) { score += 10; }
                    if cap.description.to_lowercase().contains(word) { score += 5; }
                    for kw in &cap.keywords {
                        if kw.to_lowercase().contains(word) { score += 3; }
                    }
                }
                (cap, score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();
        
        results.sort_by(|a, b| b.1.cmp(&a.1));  // Sort by score descending
        results.into_iter().map(|(cap, _)| cap).collect()
    }
    
    /// PUBLISH - Register a new capability (open ecosystem)
    pub fn publish(&mut self, metadata: CapabilityMetadata) {
        self.capabilities.insert(metadata.name.clone(), metadata);
    }
    
    /// Check if capability is available in region (regionalization)
    pub fn is_available(&self, name: &str, region: &str) -> bool {
        if let Some(cap) = self.capabilities.get(name) {
            cap.regions.is_empty() || cap.regions.contains(&region.to_string())
        } else {
            false
        }
    }
}

impl CompilationCache {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get(&mut self, hash: u64) -> Option<&CacheEntry> {
        if self.entries.contains_key(&hash) {
            self.hits += 1;
            self.entries.get(&hash)
        } else {
            self.misses += 1;
            None
        }
    }
    
    pub fn put(&mut self, hash: u64, ir: String, dependencies: Vec<u64>) {
        self.entries.insert(hash, CacheEntry {
            hash,
            ir,
            dependencies,
            signature: None,
        });
    }
    
    pub fn invalidate(&mut self, hash: u64) {
        // Remove entry and all entries that depend on it
        if self.entries.remove(&hash).is_some() {
            let dependents: Vec<u64> = self.entries.iter()
                .filter(|(_, e)| e.dependencies.contains(&hash))
                .map(|(h, _)| *h)
                .collect();
            for dep in dependents {
                self.invalidate(dep);
            }
        }
    }
    
    pub fn stats(&self) -> (u64, u64, f64) {
        let total = self.hits + self.misses;
        let ratio = if total > 0 { self.hits as f64 / total as f64 } else { 0.0 };
        (self.hits, self.misses, ratio)
    }
}

/// A node in the data flow graph
#[derive(Debug, Clone)]
pub enum Node {
    /// Module import: USE http
    Use { module: String },
    
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
    
    /// Capability requirement declaration
    Requires { id: String, capabilities: Vec<Capability> },
    
    /// High-level capability operation (resolved by compiler)
    CapabilityOp { id: String, capability: Capability, operation: String, args: Vec<Arg> },
}

impl Node {
    /// Compute deterministic hash for incremental compilation (IA-first)
    pub fn compute_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        match self {
            Node::Literal { id, value } => {
                "Literal".hash(&mut hasher);
                id.hash(&mut hasher);
                format!("{:?}", value).hash(&mut hasher);
            }
            Node::Reference { id, target } => {
                "Reference".hash(&mut hasher);
                id.hash(&mut hasher);
                target.hash(&mut hasher);
            }
            Node::Operation { id, op, args } => {
                "Operation".hash(&mut hasher);
                id.hash(&mut hasher);
                format!("{:?}", op).hash(&mut hasher);
                for arg in args {
                    format!("{:?}", arg).hash(&mut hasher);
                }
            }
            Node::SubGraph { id, inputs, outputs, nodes } => {
                "SubGraph".hash(&mut hasher);
                id.hash(&mut hasher);
                inputs.hash(&mut hasher);
                outputs.hash(&mut hasher);
                for node in nodes {
                    node.compute_hash().hash(&mut hasher);
                }
            }
            Node::Flow { source, dest } => {
                "Flow".hash(&mut hasher);
                format!("{:?}", source).hash(&mut hasher);
                format!("{:?}", dest).hash(&mut hasher);
            }
            Node::Label(s) => {
                "Label".hash(&mut hasher);
                s.hash(&mut hasher);
            }
            Node::Use { module } => {
                "Use".hash(&mut hasher);
                module.hash(&mut hasher);
            }
            Node::Function { id, params, body } => {
                "Function".hash(&mut hasher);
                id.hash(&mut hasher);
                params.hash(&mut hasher);
                for node in body {
                    node.compute_hash().hash(&mut hasher);
                }
            }
            Node::Requires { id, capabilities } => {
                "Requires".hash(&mut hasher);
                id.hash(&mut hasher);
                for cap in capabilities {
                    format!("{:?}", cap).hash(&mut hasher);
                }
            }
            Node::CapabilityOp { id, capability, operation, args } => {
                "CapabilityOp".hash(&mut hasher);
                id.hash(&mut hasher);
                format!("{:?}", capability).hash(&mut hasher);
                operation.hash(&mut hasher);
                for arg in args {
                    format!("{:?}", arg).hash(&mut hasher);
                }
            }
        }
        
        hasher.finish()
    }
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
    // Control (BRN only - LOOP/BRK/CNT deprecated)
    Brn,
    // Conditional Jump (for real loops)
    Jmp,  // JMP cond label_true label_false -> conditional branch
    // While loop
    Whl,  // WHL cond body_label -> while loop
    // Conditional termination
    End,  // END cond -> terminate if cond == 0, continue if != 0
    // Graph Transform
    Trn,  // TRN input rule -> apply rule to each element, return vector
    // Finite State Machine
    Fsm,  // FSM input len rules -> tokenize with state machine
    // Parse Scan - AI-native token pattern scanner
    Psc,  // PSC tokens count -> scan for DOT-ID-COLON patterns, return [total, lits, refs, ops]
    // AST Builder - AI-native AST generator
    Ast,  // AST tokens count -> build AST from tokens, return [ast_ptr, node_count]
    // Code Emitter - AI-native code generator
    Emt,  // EMT ast_ptr node_count src_buf out_buf -> emit code, return bytes written
    // Effects (I/O)
    Prt, Inp, Opn, Acc, Get, Put, Cls, Err,
    // Read i64 from memory
    Ge8,  // GE8 ptr offset -> read i64
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
    Scm,  // SCM str1 str2 len -> 0 if equal, non-zero if different
    Wrt,  // WRT dst pos src len -> write src to dst at pos, returns pos+len
    Ifz,  // IFZ cond val_zero val_nonzero -> select value based on cond
    // Graph Transform (AI-native iteration) - Trn already defined above
    Red,  // RED input op init -> reduce input using op, starting with init
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
    
    // === CAPABILITY SYSTEM (SCN) ===
    // Capability declaration
    Req,  // REQ capabilities... -> declare required capabilities
    
    // HTTP Capability Operations
    HttpListen,   // HTTP.LISTEN port -> server_handle
    HttpAccept,   // HTTP.ACCEPT server -> request_handle
    HttpMethod,   // HTTP.METHOD request -> method string (GET, POST, etc.)
    HttpPath,     // HTTP.PATH request -> path string
    HttpBody,     // HTTP.BODY request -> body string
    HttpRespond,  // HTTP.RESPOND request status body -> send response
    HttpGet,      // HTTP.GET url -> response body
    HttpPost,     // HTTP.POST url body -> response body
    
    // SQL Capability Operations
    SqlConnect,   // SQL.CONNECT connection_string -> connection_handle
    SqlQuery,     // SQL.QUERY connection query -> result_set
    SqlExec,      // SQL.EXEC connection statement -> rows_affected
    SqlNext,      // SQL.NEXT result_set -> 1 if has row, 0 if done
    SqlGet,       // SQL.GET result_set column -> value
    SqlClose,     // SQL.CLOSE connection -> close connection
    
    // JSON Capability Operations
    JsonParse,    // JSON.PARSE string -> json_object
    JsonEncode,   // JSON.ENCODE object -> string
    JsonGet,      // JSON.GET object key -> value
    JsonSet,      // JSON.SET object key value -> modified object
    
    // === PHASE 8: CONTRACTS & NEGOTIATION (IA-first) ===
    
    // Contract operations
    Contract,     // CONTRACT { guarantees, limits } -> define resource contract
    Guarantee,    // GUARANTEE capability property -> declare guarantee
    Limit,        // LIMIT resource value -> set resource limit
    Sandbox,      // SANDBOX code contract -> execute under contract
    
    // Negotiation operations
    Provides,     // PROVIDES { capabilities, guarantees } -> offer capabilities
    Negotiate,    // NEGOTIATE offer need -> attempt to match
    Bind,         // BIND offer.cap TO need.cap -> create binding
    
    // Custom capability operations
    DefCap,       // DEFINE_CAPABILITY name { inputs, pattern, provides }
    ExtendCap,    // EXTEND_CAPABILITY base { additions }
    ComposeCap,   // COMPOSE_CAPABILITIES { cap1, cap2, ... }
    
    // === PHASE 10: TESTING (IA-first) ===
    
    // Property-based testing
    Property,     // PROPERTY FORALL vars: assertion
    GenTests,     // GENERATE_TESTS property count -> test cases
    Verify,       // VERIFY property -> check if holds
    
    // === PHASE 9.2: INCREMENTAL COMPILATION (IA-first) ===
    
    // Content-Addressable Cache operations
    Hash,         // HASH node -> deterministic hash of node
    CacheGet,     // CACHE_GET hash -> cached IR or null
    CachePut,     // CACHE_PUT hash ir -> store in cache
    CacheVerify,  // CACHE_VERIFY hash -> 1 if valid, 0 if corrupted
    CacheInvalidate, // CACHE_INVALIDATE hash -> remove from cache
    
    // === PHASE 11: SEN - Sistema de Ecosistema TAYNI (IA-first) ===
    
    // Capability discovery (design-time for IAs)
    Discover,       // DISCOVER "description" -> list of matching capabilities
    CapInfo,        // CAPABILITY_INFO cap_name -> capability metadata
    CapCost,        // CAPABILITY_COST cap_name -> { memory, time, tokens }
    CapPublish,     // PUBLISH capability -> register in ecosystem
    CapAvailable,   // CAPABILITY_AVAILABLE cap_name region -> 1 if available
    CapVersion,     // CAPABILITY_VERSION cap_name -> version string
    CapDeps,        // CAPABILITY_DEPS cap_name -> list of dependencies
    
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
    pub requirements: Requirements,
}

impl Graph {
    /// Compute deterministic hash for the entire graph (IA-first: Content-Addressable)
    pub fn compute_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash all nodes
        for node in &self.nodes {
            node.compute_hash().hash(&mut hasher);
        }
        
        // Hash requirements
        for cap in &self.requirements.capabilities {
            format!("{:?}", cap).hash(&mut hasher);
        }
        
        hasher.finish()
    }
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
            requirements: Requirements::default(),
        }
    }
    
    pub fn add_node(&mut self, node: Node) {
        // Extract capabilities from Requires nodes
        if let Node::Requires { capabilities, .. } = &node {
            for cap in capabilities {
                self.requirements.capabilities.insert(cap.clone());
            }
        }
        
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
    
    /// Get all USE directives (imported modules)
    pub fn get_uses(&self) -> Vec<String> {
        self.nodes.iter()
            .filter_map(|node| {
                if let Node::Use { module } = node {
                    Some(module.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Rebuild node_map after modifying nodes vector
    pub fn rebuild_node_map(&mut self) {
        self.node_map.clear();
        for (idx, node) in self.nodes.iter().enumerate() {
            if let Some(id) = Self::get_node_id(node) {
                self.node_map.insert(id, idx);
            }
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
    
    /// Tree-shaking: Remove unused nodes from the graph
    /// Returns the number of nodes removed
    pub fn tree_shake(&mut self) -> usize {
        let analysis = self.analyze();
        let dead_nodes: HashSet<String> = analysis.dead_nodes.into_iter().collect();
        
        if dead_nodes.is_empty() {
            return 0;
        }
        
        let original_count = self.nodes.len();
        
        // Remove dead nodes
        self.nodes.retain(|node| {
            let id = match node {
                Node::Literal { id, .. } => Some(id),
                Node::Reference { id, .. } => Some(id),
                Node::Operation { id, .. } => Some(id),
                Node::SubGraph { id, .. } => Some(id),
                Node::Function { id, .. } => Some(id),
                _ => None,
            };
            
            match id {
                Some(node_id) => !dead_nodes.contains(node_id),
                None => true, // Keep nodes without IDs (Flow, Label, Use)
            }
        });
        
        // Rebuild node map
        self.rebuild_node_map();
        
        original_count - self.nodes.len()
    }
    
    /// Aggressive tree-shaking: Multiple passes until no more nodes can be removed
    pub fn tree_shake_aggressive(&mut self) -> usize {
        let mut total_removed = 0;
        loop {
            let removed = self.tree_shake();
            if removed == 0 {
                break;
            }
            total_removed += removed;
        }
        total_removed
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
    // Phase 9: Capability-aware DCE
    pub uses_contracts: bool,
    pub uses_negotiation: bool,
    pub uses_custom_caps: bool,
    pub uses_testing: bool,
    pub used_capabilities: HashSet<Capability>,
    pub declared_capabilities: HashSet<Capability>,
}

impl UsageAnalysis {
    pub fn analyze(graph: &Graph) -> Self {
        let mut analysis = UsageAnalysis::default();
        
        // Collect declared capabilities from requirements
        for cap in &graph.requirements.capabilities {
            analysis.declared_capabilities.insert(cap.clone());
        }
        
        for node in &graph.nodes {
            analysis.visit_node(node);
        }
        
        analysis
    }
    
    /// Returns capabilities that are declared but never used
    pub fn unused_capabilities(&self) -> HashSet<Capability> {
        self.declared_capabilities
            .difference(&self.used_capabilities)
            .cloned()
            .collect()
    }
    
    /// Returns true if any capability-related code should be included
    pub fn needs_capability_runtime(&self) -> bool {
        self.uses_contracts || self.uses_negotiation || 
        self.uses_custom_caps || self.uses_testing ||
        !self.used_capabilities.is_empty()
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
                self.used_capabilities.insert(Capability::FileSystem);
            }
            // Dynamic Vectors
            Op::Vec | Op::Vph | Op::Vgt | Op::Vst | Op::Vln | Op::Vcp => {
                self.uses_file_io = true;  // Uses memory allocation
                self.used_capabilities.insert(Capability::Memory);
            }
            // HashMap
            Op::Hmp | Op::Hpt | Op::Hgt | Op::Hhs => {
                self.uses_file_io = true;  // Uses memory allocation
                self.used_capabilities.insert(Capability::Memory);
            }
            // String operations
            Op::Cat | Op::Its | Op::Chr | Op::Sbs => {
                self.uses_file_io = true;  // Uses memory
                self.used_capabilities.insert(Capability::Memory);
            }
            // Console I/O
            Op::Prt | Op::Inp | Op::Err => {
                self.uses_console = true;
                self.used_capabilities.insert(Capability::IO);
            }
            // Capability System operations
            Op::Req => {
                // Requirements declaration - no runtime effect
            }
            Op::HttpListen | Op::HttpAccept | Op::HttpMethod | Op::HttpPath |
            Op::HttpBody | Op::HttpRespond => {
                self.uses_network = true;
                self.used_capabilities.insert(Capability::HttpServer);
            }
            Op::HttpGet | Op::HttpPost => {
                self.uses_network = true;
                self.used_capabilities.insert(Capability::HttpClient);
            }
            Op::SqlConnect | Op::SqlQuery | Op::SqlExec | Op::SqlNext |
            Op::SqlGet | Op::SqlClose => {
                self.uses_file_io = true;  // SQL uses I/O
                self.used_capabilities.insert(Capability::Sql);
            }
            Op::JsonParse | Op::JsonEncode | Op::JsonGet | Op::JsonSet => {
                self.used_capabilities.insert(Capability::Json);
            }
            // Phase 8: Contracts & Negotiation
            Op::Contract | Op::Guarantee | Op::Limit | Op::Sandbox => {
                self.uses_contracts = true;
            }
            Op::Provides | Op::Negotiate | Op::Bind => {
                self.uses_negotiation = true;
            }
            Op::DefCap | Op::ExtendCap | Op::ComposeCap => {
                self.uses_custom_caps = true;
            }
            // Phase 10: Testing
            Op::Property | Op::GenTests | Op::Verify => {
                self.uses_testing = true;
            }
            // Phase 9.2: Incremental Compilation (compile-time operations)
            Op::Hash | Op::CacheGet | Op::CachePut | 
            Op::CacheVerify | Op::CacheInvalidate => {
                // Cache operations - compile-time
            }
            // Phase 11: SEN - Ecosystem (design-time operations for IAs)
            Op::Discover | Op::CapInfo | Op::CapCost | Op::CapPublish |
            Op::CapAvailable | Op::CapVersion | Op::CapDeps => {
                // SEN operations - design-time for IAs
            }
            _ => {}
        }
    }
}
