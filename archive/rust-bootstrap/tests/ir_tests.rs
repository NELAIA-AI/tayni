//! IR Unit Tests
//! Tests for Intermediate Representation structures and analysis

#[cfg(test)]
mod tests {
    use tayni_c::ir::*;
    use std::collections::HashSet;

    // ============================================
    // Graph Construction Tests
    // ============================================

    #[test]
    fn test_graph_new() {
        let graph = Graph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.node_map.is_empty());
    }

    #[test]
    fn test_graph_add_literal() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal {
            id: "x".to_string(),
            value: Value::Int(42),
            runtime: false,
        });
        
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.node_map.contains_key("x"));
    }

    #[test]
    fn test_graph_add_operation() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "sum".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("a".to_string()), Arg::Ref("b".to_string())],
            runtime: true,
        });
        
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.node_map.contains_key("sum"));
    }

    #[test]
    fn test_graph_add_use() {
        let mut graph = Graph::new();
        graph.add_node(Node::Use { module: "http".to_string() });
        graph.add_node(Node::Use { module: "json".to_string() });
        
        let uses = graph.get_uses();
        assert_eq!(uses.len(), 2);
        assert!(uses.contains(&"http".to_string()));
        assert!(uses.contains(&"json".to_string()));
    }

    #[test]
    fn test_graph_add_requires() {
        let mut graph = Graph::new();
        graph.add_node(Node::Requires {
            id: "caps".to_string(),
            capabilities: vec![Capability::HttpServer, Capability::Json],
        });
        
        assert!(graph.requirements.capabilities.contains(&Capability::HttpServer));
        assert!(graph.requirements.capabilities.contains(&Capability::Json));
    }

    #[test]
    fn test_graph_rebuild_node_map() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal { id: "a".to_string(), value: Value::Int(1), runtime: false });
        graph.add_node(Node::Literal { id: "b".to_string(), value: Value::Int(2), runtime: false });
        
        graph.node_map.clear();
        graph.rebuild_node_map();
        
        assert!(graph.node_map.contains_key("a"));
        assert!(graph.node_map.contains_key("b"));
    }

    // ============================================
    // Node Hash Tests (Content-Addressable)
    // ============================================

    #[test]
    fn test_node_hash_literal_deterministic() {
        let node1 = Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false };
        let node2 = Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false };
        
        assert_eq!(node1.compute_hash(), node2.compute_hash());
    }

    #[test]
    fn test_node_hash_different_values() {
        let node1 = Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false };
        let node2 = Node::Literal { id: "x".to_string(), value: Value::Int(43), runtime: false };
        
        assert_ne!(node1.compute_hash(), node2.compute_hash());
    }

    #[test]
    fn test_node_hash_different_ids() {
        let node1 = Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false };
        let node2 = Node::Literal { id: "y".to_string(), value: Value::Int(42), runtime: false };
        
        assert_ne!(node1.compute_hash(), node2.compute_hash());
    }

    #[test]
    fn test_node_hash_runtime_flag() {
        let node1 = Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false };
        let node2 = Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: true };
        
        assert_ne!(node1.compute_hash(), node2.compute_hash());
    }

    #[test]
    fn test_graph_hash_deterministic() {
        let mut graph1 = Graph::new();
        graph1.add_node(Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false });
        graph1.add_node(Node::Operation {
            id: "y".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("x".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        
        let mut graph2 = Graph::new();
        graph2.add_node(Node::Literal { id: "x".to_string(), value: Value::Int(42), runtime: false });
        graph2.add_node(Node::Operation {
            id: "y".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("x".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        
        assert_eq!(graph1.compute_hash(), graph2.compute_hash());
    }

    // ============================================
    // Graph Analysis Tests
    // ============================================

    #[test]
    fn test_analyze_no_issues() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal { id: "a".to_string(), value: Value::Int(10), runtime: false });
        graph.add_node(Node::Literal { id: "b".to_string(), value: Value::Int(20), runtime: false });
        graph.add_node(Node::Operation {
            id: "sum".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("a".to_string()), Arg::Ref("b".to_string())],
            runtime: true,
        });
        graph.add_node(Node::Operation {
            id: "out".to_string(),
            op: Op::Prt,
            args: vec![Arg::Ref("sum".to_string())],
            runtime: true,
        });
        
        let analysis = graph.analyze();
        assert!(analysis.undefined_refs.is_empty());
        assert!(analysis.cycles.is_empty());
    }

    #[test]
    fn test_analyze_undefined_reference() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "sum".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("undefined".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        
        let analysis = graph.analyze();
        assert!(analysis.undefined_refs.contains(&"undefined".to_string()));
    }

    #[test]
    fn test_analyze_cycle_detection() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "a".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("b".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        graph.add_node(Node::Operation {
            id: "b".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("a".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        
        let analysis = graph.analyze();
        assert!(!analysis.cycles.is_empty());
    }

    #[test]
    fn test_analyze_dead_nodes() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal { id: "used".to_string(), value: Value::Int(42), runtime: false });
        graph.add_node(Node::Literal { id: "unused".to_string(), value: Value::Int(99), runtime: false });
        graph.add_node(Node::Operation {
            id: "out".to_string(),
            op: Op::Prt,
            args: vec![Arg::Ref("used".to_string())],
            runtime: true,
        });
        
        let analysis = graph.analyze();
        assert!(analysis.dead_nodes.contains(&"unused".to_string()));
    }

    // ============================================
    // Tree Shaking Tests
    // ============================================

    #[test]
    fn test_tree_shake_removes_dead_nodes() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal { id: "used".to_string(), value: Value::Int(42), runtime: false });
        graph.add_node(Node::Literal { id: "unused".to_string(), value: Value::Int(99), runtime: false });
        graph.add_node(Node::Operation {
            id: "out".to_string(),
            op: Op::Prt,
            args: vec![Arg::Ref("used".to_string())],
            runtime: true,
        });
        
        let removed = graph.tree_shake();
        assert!(removed > 0);
        assert!(!graph.node_map.contains_key("unused"));
    }

    #[test]
    fn test_tree_shake_keeps_used_nodes() {
        let mut graph = Graph::new();
        graph.add_node(Node::Literal { id: "a".to_string(), value: Value::Int(10), runtime: false });
        graph.add_node(Node::Literal { id: "b".to_string(), value: Value::Int(20), runtime: false });
        graph.add_node(Node::Operation {
            id: "sum".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("a".to_string()), Arg::Ref("b".to_string())],
            runtime: true,
        });
        graph.add_node(Node::Operation {
            id: "out".to_string(),
            op: Op::Prt,
            args: vec![Arg::Ref("sum".to_string())],
            runtime: true,
        });
        
        let removed = graph.tree_shake();
        assert_eq!(removed, 0);
        assert_eq!(graph.nodes.len(), 4);
    }

    #[test]
    fn test_tree_shake_aggressive() {
        let mut graph = Graph::new();
        // Chain: a -> b -> c -> d, only d is used
        graph.add_node(Node::Literal { id: "a".to_string(), value: Value::Int(1), runtime: false });
        graph.add_node(Node::Operation {
            id: "b".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("a".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        graph.add_node(Node::Operation {
            id: "c".to_string(),
            op: Op::Add,
            args: vec![Arg::Ref("b".to_string()), Arg::Lit(Value::Int(1))],
            runtime: true,
        });
        graph.add_node(Node::Operation {
            id: "d".to_string(),
            op: Op::Prt,
            args: vec![Arg::Ref("c".to_string())],
            runtime: true,
        });
        
        let removed = graph.tree_shake_aggressive();
        assert_eq!(removed, 0); // All nodes are used in chain
    }

    // ============================================
    // Usage Analysis Tests
    // ============================================

    #[test]
    fn test_usage_analysis_network() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "sock".to_string(),
            op: Op::Tcp,
            args: vec![],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_network);
        assert!(analysis.used_ops.contains(&Op::Tcp));
    }

    #[test]
    fn test_usage_analysis_threading() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "thread".to_string(),
            op: Op::Thr,
            args: vec![Arg::Ref("func".to_string()), Arg::Ref("arg".to_string())],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_threading);
    }

    #[test]
    fn test_usage_analysis_gui() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "win".to_string(),
            op: Op::Win,
            args: vec![],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_gui);
    }

    #[test]
    fn test_usage_analysis_file_io() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "file".to_string(),
            op: Op::Fop,
            args: vec![Arg::Ref("path".to_string()), Arg::Lit(Value::Int(0))],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_file_io);
        assert!(analysis.used_capabilities.contains(&Capability::FileSystem));
    }

    #[test]
    fn test_usage_analysis_console() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "out".to_string(),
            op: Op::Prt,
            args: vec![Arg::Ref("msg".to_string())],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_console);
        assert!(analysis.used_capabilities.contains(&Capability::IO));
    }

    #[test]
    fn test_usage_analysis_http_server() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "server".to_string(),
            op: Op::HttpListen,
            args: vec![Arg::Lit(Value::Int(8080))],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_network);
        assert!(analysis.used_capabilities.contains(&Capability::HttpServer));
    }

    #[test]
    fn test_usage_analysis_http_client() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "resp".to_string(),
            op: Op::HttpGet,
            args: vec![Arg::Ref("url".to_string())],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.uses_network);
        assert!(analysis.used_capabilities.contains(&Capability::HttpClient));
    }

    #[test]
    fn test_usage_analysis_sql() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "conn".to_string(),
            op: Op::SqlConnect,
            args: vec![Arg::Ref("connstr".to_string())],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.used_capabilities.contains(&Capability::Sql));
    }

    #[test]
    fn test_usage_analysis_json() {
        let mut graph = Graph::new();
        graph.add_node(Node::Operation {
            id: "obj".to_string(),
            op: Op::JsonParse,
            args: vec![Arg::Ref("input".to_string())],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        assert!(analysis.used_capabilities.contains(&Capability::Json));
    }

    #[test]
    fn test_usage_analysis_unused_capabilities() {
        let mut graph = Graph::new();
        graph.add_node(Node::Requires {
            id: "caps".to_string(),
            capabilities: vec![Capability::HttpServer, Capability::Json, Capability::Sql],
        });
        graph.add_node(Node::Operation {
            id: "obj".to_string(),
            op: Op::JsonParse,
            args: vec![Arg::Ref("input".to_string())],
            runtime: true,
        });
        
        let analysis = UsageAnalysis::analyze(&graph);
        let unused = analysis.unused_capabilities();
        
        assert!(unused.contains(&Capability::HttpServer));
        assert!(unused.contains(&Capability::Sql));
        assert!(!unused.contains(&Capability::Json));
    }

    // ============================================
    // Compilation Cache Tests
    // ============================================

    #[test]
    fn test_cache_put_get() {
        let mut cache = CompilationCache::new();
        cache.put(12345, "test_ir".to_string(), vec![]);
        
        let entry = cache.get(12345);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().ir, "test_ir");
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = CompilationCache::new();
        let entry = cache.get(99999);
        assert!(entry.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = CompilationCache::new();
        cache.put(1, "ir1".to_string(), vec![]);
        cache.put(2, "ir2".to_string(), vec![]);
        
        cache.get(1); // hit
        cache.get(2); // hit
        cache.get(3); // miss
        
        let (hits, misses, ratio) = cache.stats();
        assert_eq!(hits, 2);
        assert_eq!(misses, 1);
        assert!((ratio - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = CompilationCache::new();
        cache.put(1, "ir1".to_string(), vec![]);
        cache.put(2, "ir2".to_string(), vec![1]); // depends on 1
        
        cache.invalidate(1);
        
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_none()); // Should also be invalidated
    }

    // ============================================
    // Ecosystem Registry Tests
    // ============================================

    #[test]
    fn test_registry_builtins() {
        let registry = EcosystemRegistry::new();
        
        assert!(registry.capabilities.contains_key("http"));
        assert!(registry.capabilities.contains_key("sql"));
        assert!(registry.capabilities.contains_key("json"));
        assert!(registry.capabilities.contains_key("threading"));
        assert!(registry.capabilities.contains_key("gui"));
    }

    #[test]
    fn test_registry_discover() {
        let registry = EcosystemRegistry::new();
        
        let results = registry.discover("web api");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.name == "http"));
    }

    #[test]
    fn test_registry_discover_database() {
        let registry = EcosystemRegistry::new();
        
        let results = registry.discover("database query");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.name == "sql"));
    }

    #[test]
    fn test_registry_publish() {
        let mut registry = EcosystemRegistry::new();
        
        registry.publish(CapabilityMetadata {
            name: "custom".to_string(),
            version: "1.0.0".to_string(),
            description: "Custom capability".to_string(),
            guarantees: vec![],
            cost: CapabilityCost::default(),
            dependencies: vec![],
            regions: vec![],
            keywords: vec!["custom".to_string()],
        });
        
        assert!(registry.capabilities.contains_key("custom"));
    }

    #[test]
    fn test_registry_availability() {
        let registry = EcosystemRegistry::new();
        
        // Global capabilities are available everywhere
        assert!(registry.is_available("http", "us-east"));
        assert!(registry.is_available("http", "eu-west"));
        
        // Non-existent capability
        assert!(!registry.is_available("nonexistent", "us-east"));
    }

    // ============================================
    // Value Tests
    // ============================================

    #[test]
    fn test_value_int() {
        let v = Value::Int(42);
        if let Value::Int(n) = v {
            assert_eq!(n, 42);
        } else {
            panic!("Expected Int");
        }
    }

    #[test]
    fn test_value_float() {
        let v = Value::Float(3.14);
        if let Value::Float(f) = v {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_value_string() {
        let v = Value::String("hello".to_string());
        if let Value::String(s) = v {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_value_pair() {
        let v = Value::Pair(
            Box::new(Value::Int(1)),
            Box::new(Value::Int(2)),
        );
        if let Value::Pair(a, b) = v {
            if let (Value::Int(x), Value::Int(y)) = (*a, *b) {
                assert_eq!(x, 1);
                assert_eq!(y, 2);
            }
        } else {
            panic!("Expected Pair");
        }
    }

    // ============================================
    // Capability Tests
    // ============================================

    #[test]
    fn test_capability_equality() {
        assert_eq!(Capability::Math, Capability::Math);
        assert_ne!(Capability::Math, Capability::Memory);
    }

    #[test]
    fn test_capability_custom() {
        let cap = Capability::Custom("mymodule".to_string());
        if let Capability::Custom(name) = cap {
            assert_eq!(name, "mymodule");
        }
    }

    #[test]
    fn test_capability_hashset() {
        let mut caps: HashSet<Capability> = HashSet::new();
        caps.insert(Capability::HttpServer);
        caps.insert(Capability::Json);
        caps.insert(Capability::HttpServer); // Duplicate
        
        assert_eq!(caps.len(), 2);
    }

    // ============================================
    // Contract Tests
    // ============================================

    #[test]
    fn test_contract_default() {
        let contract = Contract::default();
        assert!(contract.guarantees.is_empty());
        assert_eq!(contract.trust_level, TrustLevel::Standard);
    }

    #[test]
    fn test_resource_limit_default() {
        let limit = ResourceLimit::default();
        assert!(limit.memory_bytes.is_none());
        assert!(limit.time_ms.is_none());
        assert!(limit.connections.is_none());
        assert!(limit.cpu_percent.is_none());
    }

    #[test]
    fn test_trust_level() {
        assert_eq!(TrustLevel::default(), TrustLevel::Standard);
        assert_ne!(TrustLevel::Full, TrustLevel::Minimal);
    }
}
