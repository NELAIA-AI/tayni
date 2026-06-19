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
    Time,
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
    /// runtime: true if prefixed with @ (evaluated at runtime)
    Literal { id: String, value: Value, runtime: bool },
    
    /// Reference to another node: .id: .other
    Reference { id: String, target: String },
    
    /// Operation: .id: OP .args
    /// runtime: true if prefixed with @ (executed at runtime)
    Operation { id: String, op: Op, args: Vec<Arg>, runtime: bool },
    
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
            Node::Literal { id, value, runtime } => {
                "Literal".hash(&mut hasher);
                id.hash(&mut hasher);
                format!("{:?}", value).hash(&mut hasher);
                runtime.hash(&mut hasher);
            }
            Node::Reference { id, target } => {
                "Reference".hash(&mut hasher);
                id.hash(&mut hasher);
                target.hash(&mut hasher);
            }
            Node::Operation { id, op, args, runtime } => {
                "Operation".hash(&mut hasher);
                id.hash(&mut hasher);
                format!("{:?}", op).hash(&mut hasher);
                runtime.hash(&mut hasher);
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
    Jmp,  // JMP :label -> unconditional jump to label
    Jz,   // JZ cond :label -> jump to label if cond == 0
    Jnz,  // JNZ cond :label -> jump to label if cond != 0
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
    Thr, Jon, Mtx, Lck, Ulk, Tlk, Yld,
    // Atomic Operations (lock-free primitives)
    AtmLd,   // ATM.LD ptr -> value (atomic load, SeqCst)
    AtmSt,   // ATM.ST ptr val -> store atomically
    AtmXchg, // ATM.XCHG ptr new -> old (atomic exchange)
    AtmCas,  // ATM.CAS ptr expected desired -> success (compare-and-swap)
    AtmAdd,  // ATM.ADD ptr val -> old (atomic fetch-add)
    AtmSub,  // ATM.SUB ptr val -> old (atomic fetch-sub)
    Fence,   // FNC -> memory fence (full barrier)
    // Channels (CSP-style communication)
    Chn,     // CHN capacity -> channel_ptr
    ChnSnd,  // CHN.SND chan val -> success
    ChnRcv,  // CHN.RCV chan -> val
    ChnCls,  // CHN.CLS chan -> close channel
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
    // Command Line Arguments
    Argc, // ARGC -> number of arguments
    Argv, // ARGV index -> pointer to argument string at index
    // Random Numbers (PRNG using time as seed)
    Rnd,  // RND -> random 64-bit integer
    Rng,  // RNG max -> random integer in [0, max)
    // Logging
    Log,  // LOG buf len -> print "[timestamp] message\n"
    // HTTP Routing
    Route, // ROUTE path handler -> register route
    // Environment Variables
    GetEnv, // GETENV name buf -> length (get env var into buffer)
    // Path Operations
    PathJoin,  // PATH.JOIN dst a b -> join paths a and b into dst
    PathDir,   // PATH.DIR dst src -> get directory part of path
    PathBase,  // PATH.BASE dst src -> get filename part of path
    PathExt,   // PATH.EXT dst src -> get extension part of path
    // Hash Operations
    HashMd5,    // HASH.MD5 dst src len -> compute MD5 hash
    HashSha256, // HASH.SHA256 dst src len -> compute SHA256 hash
    // Time Formatting Operations
    TimeFmt,    // TIME.FMT dst timestamp -> format timestamp as ISO 8601
    TimeYear,   // TIME.YEAR timestamp -> extract year
    TimeMonth,  // TIME.MONTH timestamp -> extract month (1-12)
    TimeDay,    // TIME.DAY timestamp -> extract day (1-31)
    TimeHour,   // TIME.HOUR timestamp -> extract hour (0-23)
    TimeMin,    // TIME.MIN timestamp -> extract minute (0-59)
    TimeSec,    // TIME.SEC timestamp -> extract second (0-59)
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
    
    // === STDLIB TIER 0: Core Operations ===
    
    // LOG module
    LogInfo,        // LOG.INFO msg len -> print info message
    LogError,       // LOG.ERROR msg len -> print error message
    LogWarn,        // LOG.WARN msg len -> print warning message
    LogDebug,       // LOG.DEBUG msg len -> print debug message
    
    // ROUTER module
    RouterMatch,    // ROUTER.MATCH path path_len route route_len -> 1 if match, 0 if not
    RouterParam,    // ROUTER.PARAM path path_len prefix prefix_len -> param_ptr
    
    // HTTP module (extended)
    HttpIsOptions,  // HTTP.IS_OPTIONS req -> 1 if OPTIONS method
    HttpPathLen,    // HTTP.PATH_LEN req -> path length
    HttpClose,      // HTTP.CLOSE server -> close server
    
    // CORS module
    CorsConfigNew,  // CORS.CONFIG_NEW -> config_ptr
    CorsAllowOrigin,    // CORS.ALLOW_ORIGIN config origin len -> config
    CorsAllowAllOrigins, // CORS.ALLOW_ALL_ORIGINS config -> config
    CorsAllowMethods,   // CORS.ALLOW_METHODS config methods len -> config
    CorsAllowHeaders,   // CORS.ALLOW_HEADERS config headers len -> config
    CorsAllowCredentials, // CORS.ALLOW_CREDENTIALS config -> config
    CorsHandle,     // CORS.HANDLE config req -> headers_ptr
    CorsHandlePreflight, // CORS.HANDLE_PREFLIGHT config req -> headers_ptr
    CorsPreflight,  // CORS.IS_PREFLIGHT req -> 1 if preflight
    
    // === STDLIB TIER 1: Common Operations ===
    
    // TIME module
    TimeNow,        // TIME.NOW -> unix timestamp seconds
    TimeNowMs,      // TIME.NOW_MS -> unix timestamp milliseconds
    TimeSleep,      // TIME.SLEEP ms -> sleep for ms milliseconds
    
    // UUID module
    UuidV4,         // UUID.V4 buf -> generate UUID v4 into buf
    UuidV7,         // UUID.V7 buf -> generate UUID v7 into buf
    
    // HASH module (aliases for compatibility)
    // HashSha256 and HashMd5 defined above in Hash Operations section
    
    // BASE64 module
    Base64Encode,   // BASE64.ENCODE data len out -> encode to base64
    Base64Decode,   // BASE64.DECODE data len out -> decode from base64
    
    // ENV module
    EnvGet,         // ENV.GET name name_len buf buf_len -> get env var
    EnvSet,         // ENV.SET name name_len value value_len -> set env var
    
    // PATH module (aliases for compatibility)
    PathDirname,    // PATH.DIRNAME path len out -> get directory (alias for PathDir)
    PathBasename,   // PATH.BASENAME path len out -> get filename (alias for PathBase)
    
    // FORMAT module
    FormatInt,      // FORMAT.INT num buf -> format int to string
    FormatHex,      // FORMAT.HEX num buf -> format int as hex
    
    // VALIDATION module
    ValidateEmail,  // VALIDATE.EMAIL str len -> 1 if valid email
    ValidateUrl,    // VALIDATE.URL str len -> 1 if valid URL
    ValidateUuid,   // VALIDATE.UUID str len -> 1 if valid UUID
    ValidateIpv4,   // VALIDATE.IPV4 str len -> 1 if valid IPv4
    
    // TEST module
    TestAssert,     // TEST.ASSERT cond msg len -> assert condition
    TestAssertEq,   // TEST.ASSERT_EQ a b msg len -> assert equal
    TestSummary,    // TEST.SUMMARY -> print test summary
    
    // JWT module
    JwtEncode,      // JWT.ENCODE payload payload_len secret secret_len out -> encode JWT (HS256)
    JwtDecode,      // JWT.DECODE token token_len secret secret_len out -> decode JWT payload
    JwtVerify,      // JWT.VERIFY token token_len secret secret_len -> 1 if valid
    
    // REGEX module (simplified pattern matching)
    RegexMatch,     // REGEX.MATCH str len pattern pat_len -> 1 if matches
    RegexFind,      // REGEX.FIND str len pattern pat_len -> position or -1
    
    // ASYNC module (uses existing threading)
    AsyncSpawn,     // ASYNC.SPAWN func arg -> handle (alias for THR)
    AsyncAwait,     // ASYNC.AWAIT handle -> result (alias for JON)
    
    // TIMEOUT module
    TimeoutSet,     // TIMEOUT.SET ms -> sets timeout for next operation
    TimeoutCheck,   // TIMEOUT.CHECK -> 1 if timeout occurred
    
    // === STDLIB TIER 2: Specialized Operations ===
    
    // YAML module
    YamlParse,      // YAML.PARSE str len -> yaml_obj
    YamlGet,        // YAML.GET obj key key_len -> value
    YamlEncode,     // YAML.ENCODE obj buf -> encode to YAML
    
    // CSV module
    CsvParse,       // CSV.PARSE str len -> csv_obj
    CsvNextRow,     // CSV.NEXT_ROW csv -> 1 if has row
    CsvGetField,    // CSV.GET_FIELD csv col -> field value
    CsvEncode,      // CSV.ENCODE rows buf -> encode to CSV
    
    // XML module
    XmlParse,       // XML.PARSE str len -> xml_obj
    XmlRoot,        // XML.ROOT xml -> root element
    XmlTag,         // XML.TAG elem -> tag name
    XmlAttr,        // XML.ATTR elem name name_len -> attr value
    XmlText,        // XML.TEXT elem -> text content
    
    // CRYPTO module
    AesEncrypt,     // AES.ENCRYPT key data len out -> encrypt with AES-256
    AesDecrypt,     // AES.DECRYPT key data len out -> decrypt with AES-256
    RsaGenerate,    // RSA.GENERATE bits pub_out priv_out -> generate keypair
    RsaEncrypt,     // RSA.ENCRYPT pub data len out -> encrypt with RSA
    RsaDecrypt,     // RSA.DECRYPT priv data len out -> decrypt with RSA
    Sha1,           // SHA1 dst src len -> compute SHA1 hash (20 bytes / 40 hex)
    Md5,            // MD5 dst src len -> compute MD5 hash (16 bytes / 32 hex)
    HmacSha256,     // HMAC.SHA256 dst key key_len data data_len -> compute HMAC-SHA256
    
    // POSTGRES module
    PgConnect,      // PG.CONNECT connstr -> connection handle
    PgQuery,        // PG.QUERY conn query query_len -> result handle
    PgFetch,        // PG.FETCH result dst -> row data length
    PgClose,        // PG.CLOSE conn -> close connection
    
    // REDIS module
    RedisConnect,   // REDIS.CONNECT host port -> connection handle
    RedisSet,       // REDIS.SET conn key key_len val val_len -> 1 if OK
    RedisGet,       // REDIS.GET conn dst key key_len -> value length
    RedisDel,       // REDIS.DEL conn key key_len -> 1 if deleted
    RedisClose,     // REDIS.CLOSE conn -> close connection
    
    // SQL/ODBC module - SqlFetch added (others already exist)
    SqlFetch,       // SQL.FETCH result dst -> row data length
    
    // WEBSOCKET module
    WsConnect,      // WS.CONNECT url url_len -> ws handle
    WsAccept,       // WS.ACCEPT server -> ws handle (for server)
    WsSend,         // WS.SEND ws data len -> bytes sent
    WsRecv,         // WS.RECV ws buf buf_len -> bytes received
    WsClose,        // WS.CLOSE ws -> close websocket
    
    // TLS module
    TlsConnect,     // TLS.CONNECT host port -> tls handle
    TlsAccept,      // TLS.ACCEPT server cert key -> tls handle
    TlsSend,        // TLS.SEND tls data len -> bytes sent
    TlsRecv,        // TLS.RECV tls buf buf_len -> bytes received
    TlsClose,       // TLS.CLOSE tls -> close connection
    
    // GRPC module
    GrpcChannel,    // GRPC.CHANNEL host port -> channel handle
    GrpcCall,       // GRPC.CALL channel method method_len -> call handle
    GrpcSend,       // GRPC.SEND call data len -> bytes sent
    GrpcRecv,       // GRPC.RECV call buf buf_len -> bytes received
    GrpcClose,      // GRPC.CLOSE channel -> close channel
    
    // PQC (Post-Quantum Cryptography) module
    KyberKeygen,    // KYBER.KEYGEN pub_out priv_out -> 1 if success
    KyberEncaps,    // KYBER.ENCAPS pub ct_out ss_out -> 1 if success
    KyberDecaps,    // KYBER.DECAPS priv ct ss_out -> 1 if success
    DilithiumKeygen,// DILITHIUM.KEYGEN pub_out priv_out -> 1 if success
    DilithiumSign,  // DILITHIUM.SIGN priv msg msg_len sig_out -> sig_len
    DilithiumVerify,// DILITHIUM.VERIFY pub msg msg_len sig sig_len -> 1 if valid
    
    // GZIP module
    GzipCompress,   // GZIP.COMPRESS data len out -> compress
    GzipDecompress, // GZIP.DECOMPRESS data len out -> decompress
    
    // RETRY module
    RetryConfigNew, // RETRY.CONFIG_NEW -> config_ptr
    RetryExecute,   // RETRY.EXECUTE config fn -> execute with retry
    
    // COOKIE module
    CookieParse,    // COOKIE.PARSE str len -> cookie_obj
    CookieGet,      // COOKIE.GET cookie name name_len -> value
    
    // GUI module (Windows user32.dll)
    GuiWin,         // GUI.WIN title title_len width height -> hwnd
    GuiShow,        // GUI.SHOW hwnd -> 1 if shown
    GuiHide,        // GUI.HIDE hwnd -> 1 if hidden
    GuiEvent,       // GUI.EVENT -> event_type (0=none, 1=click, 2=close, etc)
    GuiRun,         // GUI.RUN -> runs message loop until WM_QUIT
    GuiLabel,       // GUI.LABEL parent text text_len x y w h -> hwnd
    GuiTextbox,     // GUI.TEXTBOX parent x y w h -> hwnd
    GuiButton,      // GUI.BUTTON parent text text_len x y w h -> hwnd
    GuiGetVal,      // GUI.GETVAL hwnd dst max_len -> actual_len
    GuiSetVal,      // GUI.SETVAL hwnd text text_len -> 1 if set
    GuiMsgBox,      // GUI.MSGBOX title title_len text text_len -> button_id
    GuiDlg,         // GUI.DLG template parent -> dialog handle
    
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
#[derive(Debug, Clone)]
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
                Node::Label(label_name) => {
                    // Labels are defined names (with : prefix for consistency)
                    defined.insert(format!(":{}", label_name));
                }
                Node::Literal { id, .. } => {
                    defined.insert(id.clone());
                    deps.insert(id.clone(), Vec::new());
                }
                Node::Reference { id, target } => {
                    defined.insert(id.clone());
                    deps.insert(id.clone(), vec![target.clone()]);
                    used.insert(target.clone());
                }
                Node::Operation { id, op, args, .. } => {
                    defined.insert(id.clone());
                    let mut node_deps = Vec::new();
                    for arg in args {
                        Self::collect_refs(arg, &mut node_deps, &mut used);
                    }
                    deps.insert(id.clone(), node_deps);
                    
                    // Operations with side effects are always "used"
                    if matches!(op, Op::Put | Op::Prt | Op::Fwr | Op::Fop | Op::Frd | Op::Fcl | Op::Jmp | Op::Jz | Op::Jnz | Op::TimeSleep | Op::Thr | Op::Jon | Op::Mtx | Op::Lck | Op::Ulk | Op::Tlk | Op::Yld | Op::AtmSt | Op::AtmXchg | Op::AtmCas | Op::AtmAdd | Op::AtmSub | Op::Fence) {
                        used.insert(id.clone());
                    }
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
                // Skip label references (they start with ':')
                if !name.starts_with(':') {
                    deps.push(name.clone());
                    used.insert(name.clone());
                }
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
            Node::Operation { op, args: _, id: _, runtime: _ } => {
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
            Op::TimeNow | Op::TimeSleep => {
                self.used_capabilities.insert(Capability::Time);
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
