//! TAYNI Package Manager
//! 
//! A minimal package manager for TAYNI projects.
//! Handles project initialization, dependency resolution, and package registry.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use crate::json::{self, JsonValue, JsonNumber, JsonObject};

// ============================================================================
// Package Manifest (tayni.json)
// ============================================================================

/// Package manifest structure
#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub version: Version,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub dependencies: HashMap<String, VersionReq>,
    pub dev_dependencies: HashMap<String, VersionReq>,
    pub capabilities: Vec<String>,
    pub entry_point: String,
    pub targets: Vec<String>,
}

impl Default for Manifest {
    fn default() -> Self {
        Manifest {
            name: "my-project".to_string(),
            version: Version::new(0, 1, 0),
            description: None,
            authors: Vec::new(),
            license: Some("MIT".to_string()),
            repository: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            capabilities: Vec::new(),
            entry_point: "src/main.tayni".to_string(),
            targets: vec!["native".to_string()],
        }
    }
}

impl Manifest {
    /// Parse manifest from JSON string
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let value = json::parse(json_str)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        
        let obj = value.as_object()
            .ok_or("Manifest must be a JSON object")?;
        
        let name = get_string(&value, "name")
            .ok_or("Missing 'name' field")?;
        
        let version_str = get_string(&value, "version")
            .ok_or("Missing 'version' field")?;
        let version = Version::parse(&version_str)
            .ok_or_else(|| format!("Invalid version: {}", version_str))?;
        
        let mut manifest = Manifest {
            name,
            version,
            description: get_string(&value, "description"),
            authors: get_string_array(&value, "authors"),
            license: get_string(&value, "license"),
            repository: get_string(&value, "repository"),
            dependencies: parse_deps(&value, "dependencies"),
            dev_dependencies: parse_deps(&value, "devDependencies"),
            capabilities: get_string_array(&value, "capabilities"),
            entry_point: get_string(&value, "entryPoint")
                .unwrap_or_else(|| "src/main.tayni".to_string()),
            targets: get_string_array(&value, "targets"),
        };
        
        if manifest.targets.is_empty() {
            manifest.targets = vec!["native".to_string()];
        }
        
        Ok(manifest)
    }
    
    /// Serialize manifest to JSON string
    pub fn to_json(&self) -> String {
        let mut obj: JsonObject = Vec::new();
        
        obj.push(("name".to_string(), JsonValue::String(self.name.clone())));
        obj.push(("version".to_string(), JsonValue::String(self.version.to_string())));
        
        if let Some(ref desc) = self.description {
            obj.push(("description".to_string(), JsonValue::String(desc.clone())));
        }
        
        if !self.authors.is_empty() {
            let authors: Vec<JsonValue> = self.authors.iter()
                .map(|a| JsonValue::String(a.clone()))
                .collect();
            obj.push(("authors".to_string(), JsonValue::Array(authors)));
        }
        
        if let Some(ref license) = self.license {
            obj.push(("license".to_string(), JsonValue::String(license.clone())));
        }
        
        if let Some(ref repo) = self.repository {
            obj.push(("repository".to_string(), JsonValue::String(repo.clone())));
        }
        
        if !self.dependencies.is_empty() {
            let deps = deps_to_json(&self.dependencies);
            obj.push(("dependencies".to_string(), JsonValue::Object(deps)));
        }
        
        if !self.dev_dependencies.is_empty() {
            let deps = deps_to_json(&self.dev_dependencies);
            obj.push(("devDependencies".to_string(), JsonValue::Object(deps)));
        }
        
        if !self.capabilities.is_empty() {
            let caps: Vec<JsonValue> = self.capabilities.iter()
                .map(|c| JsonValue::String(c.clone()))
                .collect();
            obj.push(("capabilities".to_string(), JsonValue::Array(caps)));
        }
        
        obj.push(("entryPoint".to_string(), JsonValue::String(self.entry_point.clone())));
        
        let targets: Vec<JsonValue> = self.targets.iter()
            .map(|t| JsonValue::String(t.clone()))
            .collect();
        obj.push(("targets".to_string(), JsonValue::Array(targets)));
        
        json::encode_pretty(&JsonValue::Object(obj))
    }
}

// ============================================================================
// Semantic Versioning
// ============================================================================

/// Semantic version (major.minor.patch)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch, prerelease: None }
    }
    
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('v');
        let (version_part, prerelease) = if let Some(idx) = s.find('-') {
            (&s[..idx], Some(s[idx+1..].to_string()))
        } else {
            (s, None)
        };
        
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return None;
        }
        
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
        
        Some(Version { major, minor, patch, prerelease })
    }
    
    pub fn satisfies(&self, req: &VersionReq) -> bool {
        match req {
            VersionReq::Exact(v) => self == v,
            VersionReq::Caret(v) => {
                if v.major == 0 {
                    self.major == v.major && self.minor == v.minor && self.patch >= v.patch
                } else {
                    self.major == v.major && (self.minor > v.minor || 
                        (self.minor == v.minor && self.patch >= v.patch))
                }
            }
            VersionReq::Tilde(v) => {
                self.major == v.major && self.minor == v.minor && self.patch >= v.patch
            }
            VersionReq::Range { min, max } => {
                self >= min && self < max
            }
            VersionReq::Any => true,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref pre) = self.prerelease {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Version requirement
#[derive(Debug, Clone)]
pub enum VersionReq {
    Exact(Version),
    Caret(Version),   // ^1.2.3 - compatible with
    Tilde(Version),   // ~1.2.3 - approximately
    Range { min: Version, max: Version },
    Any,              // * or latest
}

impl VersionReq {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        
        if s == "*" || s == "latest" {
            return Some(VersionReq::Any);
        }
        
        if s.starts_with('^') {
            let v = Version::parse(&s[1..])?;
            return Some(VersionReq::Caret(v));
        }
        
        if s.starts_with('~') {
            let v = Version::parse(&s[1..])?;
            return Some(VersionReq::Tilde(v));
        }
        
        if s.starts_with(">=") {
            let v = Version::parse(&s[2..])?;
            return Some(VersionReq::Range { 
                min: v, 
                max: Version::new(u32::MAX, 0, 0) 
            });
        }
        
        // Exact version
        let v = Version::parse(s)?;
        Some(VersionReq::Exact(v))
    }
}

impl std::fmt::Display for VersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionReq::Exact(v) => write!(f, "{}", v),
            VersionReq::Caret(v) => write!(f, "^{}", v),
            VersionReq::Tilde(v) => write!(f, "~{}", v),
            VersionReq::Range { min, max } => write!(f, ">={} <{}", min, max),
            VersionReq::Any => write!(f, "*"),
        }
    }
}

// ============================================================================
// Package Registry
// ============================================================================

/// Package metadata from registry
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub versions: Vec<Version>,
    pub description: String,
    pub repository: Option<String>,
    pub latest: Version,
}

/// Package registry client
pub struct Registry {
    pub base_url: String,
    cache: HashMap<String, PackageInfo>,
}

impl Registry {
    pub fn new(base_url: &str) -> Self {
        Registry {
            base_url: base_url.to_string(),
            cache: HashMap::new(),
        }
    }
    
    /// Default TAYNI registry
    pub fn default() -> Self {
        Registry::new("https://registry.tayni.dev")
    }
    
    /// Get package info (from cache or fetch)
    pub fn get_package(&mut self, name: &str) -> Option<&PackageInfo> {
        if !self.cache.contains_key(name) {
            // In a real implementation, this would fetch from the registry
            // For now, return None for unknown packages
            return None;
        }
        self.cache.get(name)
    }
    
    /// Register a package in the local cache (for testing)
    pub fn register_local(&mut self, info: PackageInfo) {
        self.cache.insert(info.name.clone(), info);
    }
}

// ============================================================================
// Dependency Resolution
// ============================================================================

/// Resolved dependency
#[derive(Debug, Clone)]
pub struct ResolvedDep {
    pub name: String,
    pub version: Version,
    pub source: DepSource,
}

/// Dependency source
#[derive(Debug, Clone)]
pub enum DepSource {
    Registry(String),
    Git { url: String, rev: Option<String> },
    Path(PathBuf),
}

/// Dependency resolver
pub struct Resolver {
    registry: Registry,
    resolved: HashMap<String, ResolvedDep>,
}

impl Resolver {
    pub fn new(registry: Registry) -> Self {
        Resolver {
            registry,
            resolved: HashMap::new(),
        }
    }
    
    /// Resolve all dependencies for a manifest
    pub fn resolve(&mut self, manifest: &Manifest) -> Result<Vec<ResolvedDep>, String> {
        for (name, req) in &manifest.dependencies {
            self.resolve_dep(name, req)?;
        }
        
        Ok(self.resolved.values().cloned().collect())
    }
    
    fn resolve_dep(&mut self, name: &str, req: &VersionReq) -> Result<(), String> {
        if self.resolved.contains_key(name) {
            // Check if existing resolution satisfies requirement
            let existing = &self.resolved[name];
            if !existing.version.satisfies(req) {
                return Err(format!(
                    "Version conflict for '{}': {} required, {} resolved",
                    name, req, existing.version
                ));
            }
            return Ok(());
        }
        
        // Look up in registry
        let info = self.registry.get_package(name)
            .ok_or_else(|| format!("Package '{}' not found in registry", name))?;
        
        // Find best matching version
        let version = info.versions.iter()
            .filter(|v| v.satisfies(req))
            .max()
            .cloned()
            .ok_or_else(|| format!(
                "No version of '{}' satisfies requirement {}",
                name, req
            ))?;
        
        self.resolved.insert(name.to_string(), ResolvedDep {
            name: name.to_string(),
            version,
            source: DepSource::Registry(self.registry.base_url.clone()),
        });
        
        Ok(())
    }
}

// ============================================================================
// Lock File (tayni.lock)
// ============================================================================

/// Lock file for reproducible builds
#[derive(Debug, Clone)]
pub struct LockFile {
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub version: Version,
    pub checksum: String,
    pub source: String,
}

impl LockFile {
    pub fn new() -> Self {
        LockFile { packages: Vec::new() }
    }
    
    pub fn from_resolved(deps: &[ResolvedDep]) -> Self {
        let packages = deps.iter().map(|d| {
            LockedPackage {
                name: d.name.clone(),
                version: d.version.clone(),
                checksum: "sha256:0000000000000000".to_string(), // Placeholder
                source: match &d.source {
                    DepSource::Registry(url) => url.clone(),
                    DepSource::Git { url, rev } => {
                        if let Some(r) = rev {
                            format!("{}#{}", url, r)
                        } else {
                            url.clone()
                        }
                    }
                    DepSource::Path(p) => p.display().to_string(),
                },
            }
        }).collect();
        
        LockFile { packages }
    }
    
    pub fn to_json(&self) -> String {
        let packages: Vec<JsonValue> = self.packages.iter().map(|p| {
            let mut obj: JsonObject = Vec::new();
            obj.push(("name".to_string(), JsonValue::String(p.name.clone())));
            obj.push(("version".to_string(), JsonValue::String(p.version.to_string())));
            obj.push(("checksum".to_string(), JsonValue::String(p.checksum.clone())));
            obj.push(("source".to_string(), JsonValue::String(p.source.clone())));
            JsonValue::Object(obj)
        }).collect();
        
        let mut root: JsonObject = Vec::new();
        root.push(("lockfileVersion".to_string(), JsonValue::Number(JsonNumber::Integer(1))));
        root.push(("packages".to_string(), JsonValue::Array(packages)));
        
        json::encode_pretty(&JsonValue::Object(root))
    }
}

// ============================================================================
// Project Commands
// ============================================================================

/// Initialize a new TAYNI project
pub fn init_project(path: &Path, name: &str) -> Result<(), String> {
    // Create directory structure
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("Failed to create src directory: {}", e))?;
    
    // Create manifest
    let manifest = Manifest {
        name: name.to_string(),
        version: Version::new(0, 1, 0),
        description: Some(format!("A TAYNI project: {}", name)),
        authors: Vec::new(),
        license: Some("MIT".to_string()),
        repository: None,
        dependencies: HashMap::new(),
        dev_dependencies: HashMap::new(),
        capabilities: vec!["stdio".to_string()],
        entry_point: "src/main.tayni".to_string(),
        targets: vec!["native".to_string()],
    };
    
    let manifest_path = path.join("tayni.json");
    fs::write(&manifest_path, manifest.to_json())
        .map_err(|e| format!("Failed to write manifest: {}", e))?;
    
    // Create main.tayni
    let main_content = r#"// TAYNI Project: {name}
// Generated by tayni init

fn main() {
    print("Hello from {name}!")
}
"#.replace("{name}", name);
    
    let main_path = src_dir.join("main.tayni");
    fs::write(&main_path, main_content)
        .map_err(|e| format!("Failed to write main.tayni: {}", e))?;
    
    // Create .gitignore
    let gitignore = r#"# Build outputs
/target/
*.exe
*.wasm

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
"#;
    
    let gitignore_path = path.join(".gitignore");
    fs::write(&gitignore_path, gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;
    
    Ok(())
}

/// Add a dependency to the project
pub fn add_dependency(manifest_path: &Path, name: &str, version: &str) -> Result<(), String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    
    let mut manifest = Manifest::from_json(&content)?;
    
    let req = VersionReq::parse(version)
        .ok_or_else(|| format!("Invalid version requirement: {}", version))?;
    
    manifest.dependencies.insert(name.to_string(), req);
    
    fs::write(manifest_path, manifest.to_json())
        .map_err(|e| format!("Failed to write manifest: {}", e))?;
    
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_string(value: &JsonValue, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(|s| s.to_string())
}

fn get_string_array(value: &JsonValue, key: &str) -> Vec<String> {
    value.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_deps(value: &JsonValue, key: &str) -> HashMap<String, VersionReq> {
    let mut deps = HashMap::new();
    
    if let Some(obj) = value.get(key).and_then(|v| v.as_object()) {
        for (name, ver) in obj {
            if let Some(ver_str) = ver.as_str() {
                if let Some(req) = VersionReq::parse(ver_str) {
                    deps.insert(name.clone(), req);
                }
            }
        }
    }
    
    deps
}

fn deps_to_json(deps: &HashMap<String, VersionReq>) -> JsonObject {
    deps.iter()
        .map(|(k, v)| (k.clone(), JsonValue::String(v.to_string())))
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parse() {
        assert_eq!(Version::parse("1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse("v1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse("0.1"), Some(Version::new(0, 1, 0)));
        
        let v = Version::parse("1.0.0-beta").unwrap();
        assert_eq!(v.prerelease, Some("beta".to_string()));
    }
    
    #[test]
    fn test_version_satisfies() {
        let v = Version::new(1, 5, 3);
        
        assert!(v.satisfies(&VersionReq::Exact(Version::new(1, 5, 3))));
        assert!(!v.satisfies(&VersionReq::Exact(Version::new(1, 5, 4))));
        
        assert!(v.satisfies(&VersionReq::Caret(Version::new(1, 0, 0))));
        assert!(v.satisfies(&VersionReq::Caret(Version::new(1, 5, 0))));
        assert!(!v.satisfies(&VersionReq::Caret(Version::new(2, 0, 0))));
        
        assert!(v.satisfies(&VersionReq::Tilde(Version::new(1, 5, 0))));
        assert!(!v.satisfies(&VersionReq::Tilde(Version::new(1, 6, 0))));
    }
    
    #[test]
    fn test_manifest_roundtrip() {
        let manifest = Manifest {
            name: "test-project".to_string(),
            version: Version::new(1, 0, 0),
            description: Some("A test project".to_string()),
            authors: vec!["Test Author".to_string()],
            license: Some("MIT".to_string()),
            repository: Some("https://github.com/test/test".to_string()),
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("http-utils".to_string(), VersionReq::Caret(Version::new(1, 0, 0)));
                deps
            },
            dev_dependencies: HashMap::new(),
            capabilities: vec!["http".to_string(), "file".to_string()],
            entry_point: "src/main.tayni".to_string(),
            targets: vec!["native".to_string(), "wasm".to_string()],
        };
        
        let json = manifest.to_json();
        let parsed = Manifest::from_json(&json).unwrap();
        
        assert_eq!(parsed.name, manifest.name);
        assert_eq!(parsed.version, manifest.version);
        assert_eq!(parsed.description, manifest.description);
        assert_eq!(parsed.capabilities, manifest.capabilities);
    }
    
    #[test]
    fn test_lock_file() {
        let deps = vec![
            ResolvedDep {
                name: "http-utils".to_string(),
                version: Version::new(1, 2, 3),
                source: DepSource::Registry("https://registry.tayni.dev".to_string()),
            },
        ];
        
        let lock = LockFile::from_resolved(&deps);
        let json = lock.to_json();
        
        assert!(json.contains("http-utils"));
        assert!(json.contains("1.2.3"));
    }
}
