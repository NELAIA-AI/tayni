//! TAYNI Module Resolution
//! Resolves USE directives to actual module files
//! Search order: stdlib/tier0 → stdlib/tier1 → stdlib/tier2 → local

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

/// Module resolver for USE directives
pub struct ModuleResolver {
    /// Base path for stdlib (usually next to compiler)
    stdlib_path: PathBuf,
    /// Already resolved modules (to detect circular deps)
    resolved: HashSet<String>,
    /// Currently resolving (for circular detection)
    resolving: Vec<String>,
}

/// Result of module resolution
#[derive(Debug)]
pub struct ResolvedModule {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub tier: ModuleTier,
}

/// Module tier (affects search order and semantics)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModuleTier {
    Tier0,  // Essential (http, json, file, string, args, router, log, base64, url, random)
    Tier1,  // Common (env, path, hash, jwt, time, regex, format, test, uuid, validation, async, timeout)
    Tier2,  // Specialized (sql, postgres, redis, websocket, grpc, yaml, csv, xml, crypto, tls, pqc, cors, cookie, gzip, retry)
    Local,  // User-defined module in same directory
}

impl ModuleResolver {
    /// Create a new resolver with the given stdlib path
    pub fn new(stdlib_path: PathBuf) -> Self {
        Self {
            stdlib_path,
            resolved: HashSet::new(),
            resolving: Vec::new(),
        }
    }
    
    /// Create resolver with default stdlib path (relative to executable)
    pub fn with_default_path() -> Self {
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."));
        let stdlib_path = exe_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("stdlib");
        Self::new(stdlib_path)
    }
    
    /// Resolve a module by name
    /// Returns the module content and metadata, or an error
    pub fn resolve(&mut self, module_name: &str, source_dir: &Path) -> Result<ResolvedModule, String> {
        let name = module_name.to_lowercase();
        
        // Check for circular dependency
        if self.resolving.contains(&name) {
            let cycle: Vec<_> = self.resolving.iter()
                .skip_while(|m| *m != &name)
                .cloned()
                .collect();
            return Err(format!(
                "Circular dependency detected: {} -> {}",
                cycle.join(" -> "),
                name
            ));
        }
        
        // Already resolved?
        if self.resolved.contains(&name) {
            // Return cached (we'd need a cache for this, for now re-resolve)
        }
        
        // Mark as resolving
        self.resolving.push(name.clone());
        
        // Search order: tier0 → tier1 → tier2 → local
        let result = self.search_module(&name, source_dir);
        
        // Remove from resolving stack
        self.resolving.pop();
        
        match result {
            Ok(module) => {
                self.resolved.insert(name);
                Ok(module)
            }
            Err(e) => Err(e)
        }
    }
    
    /// Search for a module in all locations
    fn search_module(&self, name: &str, source_dir: &Path) -> Result<ResolvedModule, String> {
        // Tier 0: Essential modules
        let tier0_path = self.stdlib_path.join("tier0").join(format!("{}.tyn", name));
        if tier0_path.exists() {
            return self.load_module(name, &tier0_path, ModuleTier::Tier0);
        }
        
        // Tier 1: Common modules
        let tier1_path = self.stdlib_path.join("tier1").join(format!("{}.tyn", name));
        if tier1_path.exists() {
            return self.load_module(name, &tier1_path, ModuleTier::Tier1);
        }
        
        // Tier 2: Specialized modules
        let tier2_path = self.stdlib_path.join("tier2").join(format!("{}.tyn", name));
        if tier2_path.exists() {
            return self.load_module(name, &tier2_path, ModuleTier::Tier2);
        }
        
        // Local: Same directory as source file
        let local_path = source_dir.join(format!("{}.tyn", name));
        if local_path.exists() {
            return self.load_module(name, &local_path, ModuleTier::Local);
        }
        
        Err(format!(
            "Module '{}' not found. Searched:\n  - {}\n  - {}\n  - {}\n  - {}",
            name,
            tier0_path.display(),
            tier1_path.display(),
            tier2_path.display(),
            local_path.display()
        ))
    }
    
    /// Load module content from file
    fn load_module(&self, name: &str, path: &Path, tier: ModuleTier) -> Result<ResolvedModule, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read module '{}': {}", name, e))?;
        
        Ok(ResolvedModule {
            name: name.to_string(),
            path: path.to_path_buf(),
            content,
            tier,
        })
    }
    
    /// Get list of all available modules
    pub fn list_available(&self) -> Vec<(String, ModuleTier)> {
        let mut modules = Vec::new();
        
        for (tier, tier_name) in [
            (ModuleTier::Tier0, "tier0"),
            (ModuleTier::Tier1, "tier1"),
            (ModuleTier::Tier2, "tier2"),
        ] {
            let tier_path = self.stdlib_path.join(tier_name);
            if let Ok(entries) = fs::read_dir(&tier_path) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.path().file_stem() {
                        if entry.path().extension().map_or(false, |e| e == "tyn") {
                            modules.push((name.to_string_lossy().to_string(), tier));
                        }
                    }
                }
            }
        }
        
        modules
    }
    
    /// Check if a module exists
    pub fn exists(&self, name: &str, source_dir: &Path) -> bool {
        let name = name.to_lowercase();
        
        self.stdlib_path.join("tier0").join(format!("{}.tyn", name)).exists()
            || self.stdlib_path.join("tier1").join(format!("{}.tyn", name)).exists()
            || self.stdlib_path.join("tier2").join(format!("{}.tyn", name)).exists()
            || source_dir.join(format!("{}.tyn", name)).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    
    #[test]
    fn test_circular_detection() {
        let mut resolver = ModuleResolver::new(temp_dir());
        resolver.resolving.push("a".to_string());
        resolver.resolving.push("b".to_string());
        
        let result = resolver.resolve("a", Path::new("."));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }
}
