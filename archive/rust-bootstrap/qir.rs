//! TAYNI QIR (Quantum Intermediate Representation) Emitter
//! 
//! QIR is the ONLY native quantum executable format (like PE/ELF for classical CPUs).
//! Used by: Microsoft Azure Quantum, IonQ, Quantinuum
//! 
//! Other quantum "languages" are NOT architectures:
//! - Cirq (Google): Python DSL → translate FROM QIR
//! - OpenQASM (IBM): High-level language → translate FROM QIR  
//! - Quil (Rigetti): Proprietary assembly → translate FROM QIR
//!
//! Architecture:
//! ┌─────────────────────────────────────────────────────────────┐
//! │  TAYNI Source (.tyn)                                        │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  ┌─────────────┐                                            │
//! │  │ QIR Emitter │ ← Native quantum executable                │
//! │  └─────────────┘                                            │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
//! │  │ Azure/IonQ  │   │   Cirq      │   │   QASM      │       │
//! │  │ Quantinuum  │   │ (translate) │   │ (translate) │       │
//! │  │  (native)   │   └─────────────┘   └─────────────┘       │
//! │  └─────────────┘                                            │
//! └─────────────────────────────────────────────────────────────┘

use crate::ir::{Graph, Node, Value, Arg, Op};
use std::collections::HashMap;

/// Quantum gate types
#[derive(Clone, Copy, Debug)]
pub enum QuantumGate {
    // Single-qubit gates
    H,      // Hadamard
    X,      // Pauli-X (NOT)
    Y,      // Pauli-Y
    Z,      // Pauli-Z
    S,      // Phase gate (sqrt(Z))
    T,      // T gate (sqrt(S))
    Rx,     // Rotation around X
    Ry,     // Rotation around Y
    Rz,     // Rotation around Z
    
    // Two-qubit gates
    CNOT,   // Controlled-NOT
    CZ,     // Controlled-Z
    SWAP,   // Swap qubits
    
    // Three-qubit gates
    Toffoli, // CCX (Controlled-Controlled-NOT)
    Fredkin, // CSWAP (Controlled-SWAP)
}

/// QIR Emitter for quantum programs
pub struct QirEmitter {
    qubit_count: usize,
    result_count: usize,
    instructions: Vec<String>,
    measurements: Vec<(usize, usize)>, // (qubit, result)
}

impl QirEmitter {
    pub fn new() -> Self {
        QirEmitter {
            qubit_count: 0,
            result_count: 0,
            instructions: Vec::new(),
            measurements: Vec::new(),
        }
    }
    
    /// Generate QIR from TAYNI graph
    pub fn emit(graph: &Graph) -> Result<String, String> {
        let mut emitter = QirEmitter::new();
        emitter.emit_graph(graph)
    }
    
    fn emit_graph(&mut self, graph: &Graph) -> Result<String, String> {
        // First pass: count qubits and analyze quantum operations
        for node in &graph.nodes {
            self.analyze_node(node);
        }
        
        // Generate QIR
        let mut qir = String::new();
        
        // Header
        qir.push_str("; TAYNI Quantum IR (QIR) - Auto-generated\n");
        qir.push_str("; Compatible with Azure Quantum, IBM Qiskit\n\n");
        
        // QIR intrinsics declarations
        qir.push_str(&self.emit_qir_declarations());
        
        // Main quantum function
        qir.push_str("\ndefine void @TAYNI_quantum_main() {\n");
        qir.push_str("entry:\n");
        
        // Allocate qubits
        for i in 0..self.qubit_count {
            qir.push_str(&format!("  %q{} = call %Qubit* @__quantum__rt__qubit_allocate()\n", i));
        }
        
        // Allocate result registers
        for i in 0..self.result_count {
            qir.push_str(&format!("  %r{} = call %Result* @__quantum__rt__result_allocate()\n", i));
        }
        
        qir.push_str("\n");
        
        // Emit quantum operations
        for node in &graph.nodes {
            qir.push_str(&self.emit_quantum_node(node)?);
        }
        
        // Release qubits
        qir.push_str("\n  ; Release qubits\n");
        for i in 0..self.qubit_count {
            qir.push_str(&format!("  call void @__quantum__rt__qubit_release(%Qubit* %q{})\n", i));
        }
        
        qir.push_str("  ret void\n");
        qir.push_str("}\n");
        
        Ok(qir)
    }
    
    fn analyze_node(&mut self, node: &Node) {
        if let Node::Operation { op, args, .. } = node {
            match op {
                Op::Call(name) if name.starts_with("Q") => {
                    // Count qubits used
                    for arg in args {
                        if let Arg::Lit(Value::Int(n)) = arg {
                            if *n as usize >= self.qubit_count {
                                self.qubit_count = *n as usize + 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    fn emit_qir_declarations(&self) -> String {
        r#"
; QIR Type definitions
%Qubit = type opaque
%Result = type opaque

; Qubit management
declare %Qubit* @__quantum__rt__qubit_allocate()
declare void @__quantum__rt__qubit_release(%Qubit*)
declare %Result* @__quantum__rt__result_allocate()

; Single-qubit gates
declare void @__quantum__qis__h__body(%Qubit*)
declare void @__quantum__qis__x__body(%Qubit*)
declare void @__quantum__qis__y__body(%Qubit*)
declare void @__quantum__qis__z__body(%Qubit*)
declare void @__quantum__qis__s__body(%Qubit*)
declare void @__quantum__qis__t__body(%Qubit*)
declare void @__quantum__qis__rx__body(double, %Qubit*)
declare void @__quantum__qis__ry__body(double, %Qubit*)
declare void @__quantum__qis__rz__body(double, %Qubit*)

; Two-qubit gates
declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)

; Three-qubit gates
declare void @__quantum__qis__ccx__body(%Qubit*, %Qubit*, %Qubit*)

; Measurement
declare %Result* @__quantum__qis__m__body(%Qubit*)
declare i1 @__quantum__rt__result_equal(%Result*, %Result*)
declare %Result* @__quantum__rt__result_get_one()
declare %Result* @__quantum__rt__result_get_zero()

"#.to_string()
    }
    
    fn emit_quantum_node(&self, node: &Node) -> Result<String, String> {
        let mut code = String::new();
        
        if let Node::Operation { id, op, args, runtime: _ } = node {
            match op {
                Op::Call(name) => {
                    match name.as_str() {
                        "QH" => {
                            // Hadamard gate
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  call void @__quantum__qis__h__body(%Qubit* %q{})\n", q));
                            }
                        }
                        "QX" => {
                            // Pauli-X gate
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  call void @__quantum__qis__x__body(%Qubit* %q{})\n", q));
                            }
                        }
                        "QY" => {
                            // Pauli-Y gate
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  call void @__quantum__qis__y__body(%Qubit* %q{})\n", q));
                            }
                        }
                        "QZ" => {
                            // Pauli-Z gate
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  call void @__quantum__qis__z__body(%Qubit* %q{})\n", q));
                            }
                        }
                        "QS" => {
                            // S gate
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  call void @__quantum__qis__s__body(%Qubit* %q{})\n", q));
                            }
                        }
                        "QT" => {
                            // T gate
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  call void @__quantum__qis__t__body(%Qubit* %q{})\n", q));
                            }
                        }
                        "QRX" => {
                            // Rx rotation
                            if args.len() >= 2 {
                                if let (Arg::Lit(Value::Float(angle)), Arg::Lit(Value::Int(q))) = (&args[0], &args[1]) {
                                    code.push_str(&format!("  call void @__quantum__qis__rx__body(double {}, %Qubit* %q{})\n", angle, q));
                                }
                            }
                        }
                        "QRY" => {
                            // Ry rotation
                            if args.len() >= 2 {
                                if let (Arg::Lit(Value::Float(angle)), Arg::Lit(Value::Int(q))) = (&args[0], &args[1]) {
                                    code.push_str(&format!("  call void @__quantum__qis__ry__body(double {}, %Qubit* %q{})\n", angle, q));
                                }
                            }
                        }
                        "QRZ" => {
                            // Rz rotation
                            if args.len() >= 2 {
                                if let (Arg::Lit(Value::Float(angle)), Arg::Lit(Value::Int(q))) = (&args[0], &args[1]) {
                                    code.push_str(&format!("  call void @__quantum__qis__rz__body(double {}, %Qubit* %q{})\n", angle, q));
                                }
                            }
                        }
                        "QCNOT" | "QCX" => {
                            // CNOT gate
                            if args.len() >= 2 {
                                if let (Arg::Lit(Value::Int(ctrl)), Arg::Lit(Value::Int(tgt))) = (&args[0], &args[1]) {
                                    code.push_str(&format!("  call void @__quantum__qis__cnot__body(%Qubit* %q{}, %Qubit* %q{})\n", ctrl, tgt));
                                }
                            }
                        }
                        "QCZ" => {
                            // CZ gate
                            if args.len() >= 2 {
                                if let (Arg::Lit(Value::Int(ctrl)), Arg::Lit(Value::Int(tgt))) = (&args[0], &args[1]) {
                                    code.push_str(&format!("  call void @__quantum__qis__cz__body(%Qubit* %q{}, %Qubit* %q{})\n", ctrl, tgt));
                                }
                            }
                        }
                        "QSWAP" => {
                            // SWAP gate
                            if args.len() >= 2 {
                                if let (Arg::Lit(Value::Int(q1)), Arg::Lit(Value::Int(q2))) = (&args[0], &args[1]) {
                                    code.push_str(&format!("  call void @__quantum__qis__swap__body(%Qubit* %q{}, %Qubit* %q{})\n", q1, q2));
                                }
                            }
                        }
                        "QCCX" | "QTOFFOLI" => {
                            // Toffoli gate
                            if args.len() >= 3 {
                                if let (Arg::Lit(Value::Int(c1)), Arg::Lit(Value::Int(c2)), Arg::Lit(Value::Int(tgt))) = (&args[0], &args[1], &args[2]) {
                                    code.push_str(&format!("  call void @__quantum__qis__ccx__body(%Qubit* %q{}, %Qubit* %q{}, %Qubit* %q{})\n", c1, c2, tgt));
                                }
                            }
                        }
                        "QM" | "QMEASURE" => {
                            // Measurement
                            if let Some(Arg::Lit(Value::Int(q))) = args.first() {
                                code.push_str(&format!("  %{} = call %Result* @__quantum__qis__m__body(%Qubit* %q{})\n", id, q));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        Ok(code)
    }
}

/// Generate a simple Bell state circuit
pub fn generate_bell_state_qir() -> String {
    r#"; TAYNI Quantum IR - Bell State Example
; Creates entangled pair |00⟩ + |11⟩

%Qubit = type opaque
%Result = type opaque

declare %Qubit* @__quantum__rt__qubit_allocate()
declare void @__quantum__rt__qubit_release(%Qubit*)
declare void @__quantum__qis__h__body(%Qubit*)
declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)
declare %Result* @__quantum__qis__m__body(%Qubit*)

define void @bell_state() {
entry:
  ; Allocate 2 qubits
  %q0 = call %Qubit* @__quantum__rt__qubit_allocate()
  %q1 = call %Qubit* @__quantum__rt__qubit_allocate()
  
  ; Apply Hadamard to q0
  call void @__quantum__qis__h__body(%Qubit* %q0)
  
  ; Apply CNOT with q0 as control, q1 as target
  call void @__quantum__qis__cnot__body(%Qubit* %q0, %Qubit* %q1)
  
  ; Measure both qubits
  %r0 = call %Result* @__quantum__qis__m__body(%Qubit* %q0)
  %r1 = call %Result* @__quantum__qis__m__body(%Qubit* %q1)
  
  ; Release qubits
  call void @__quantum__rt__qubit_release(%Qubit* %q0)
  call void @__quantum__rt__qubit_release(%Qubit* %q1)
  
  ret void
}
"#.to_string()
}

// =============================================================================
// QIR TRANSLATION TARGETS (NOT architectures - export formats only)
// =============================================================================

/// Translation target format
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QirTranslation {
    Cirq,   // Google - Python DSL
    Qasm,   // IBM - OpenQASM 3.0
    Quil,   // Rigetti - proprietary assembly
}

/// Translate QIR to other quantum formats
/// Note: These are NOT native targets, just convenience exports
pub struct QirTranslator;

impl QirTranslator {
    /// Translate QIR to OpenQASM 3.0 (IBM)
    pub fn to_qasm(qir: &str, num_qubits: usize) -> String {
        let mut qasm = String::new();
        qasm.push_str("// Generated by TAYNI from QIR\n");
        qasm.push_str("// Target: IBM Quantum (OpenQASM 3.0)\n");
        qasm.push_str("OPENQASM 3.0;\n");
        qasm.push_str("include \"stdgates.inc\";\n\n");
        
        // Declare qubits and classical bits
        qasm.push_str(&format!("qubit[{}] q;\n", num_qubits));
        qasm.push_str(&format!("bit[{}] c;\n\n", num_qubits));
        
        // Parse QIR and translate gates
        for line in qir.lines() {
            let line = line.trim();
            if line.contains("@__quantum__qis__h__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("h q[{}];\n", q));
                }
            } else if line.contains("@__quantum__qis__x__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("x q[{}];\n", q));
                }
            } else if line.contains("@__quantum__qis__y__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("y q[{}];\n", q));
                }
            } else if line.contains("@__quantum__qis__z__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("z q[{}];\n", q));
                }
            } else if line.contains("@__quantum__qis__s__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("s q[{}];\n", q));
                }
            } else if line.contains("@__quantum__qis__t__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("t q[{}];\n", q));
                }
            } else if line.contains("@__quantum__qis__cnot__body") {
                if let Some((ctrl, tgt)) = Self::extract_two_qubits(line) {
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", ctrl, tgt));
                }
            } else if line.contains("@__quantum__qis__cz__body") {
                if let Some((ctrl, tgt)) = Self::extract_two_qubits(line) {
                    qasm.push_str(&format!("cz q[{}], q[{}];\n", ctrl, tgt));
                }
            } else if line.contains("@__quantum__qis__swap__body") {
                if let Some((q1, q2)) = Self::extract_two_qubits(line) {
                    qasm.push_str(&format!("swap q[{}], q[{}];\n", q1, q2));
                }
            } else if line.contains("@__quantum__qis__m__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    qasm.push_str(&format!("c[{}] = measure q[{}];\n", q, q));
                }
            }
        }
        
        qasm
    }
    
    /// Translate QIR to Cirq (Google) - Python code
    pub fn to_cirq(qir: &str, num_qubits: usize) -> String {
        let mut cirq = String::new();
        cirq.push_str("# Generated by TAYNI from QIR\n");
        cirq.push_str("# Target: Google Cirq\n");
        cirq.push_str("import cirq\n\n");
        
        // Create qubits
        cirq.push_str(&format!("qubits = cirq.LineQubit.range({})\n", num_qubits));
        cirq.push_str("circuit = cirq.Circuit()\n\n");
        
        // Parse QIR and translate gates
        for line in qir.lines() {
            let line = line.trim();
            if line.contains("@__quantum__qis__h__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.H(qubits[{}]))\n", q));
                }
            } else if line.contains("@__quantum__qis__x__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.X(qubits[{}]))\n", q));
                }
            } else if line.contains("@__quantum__qis__y__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.Y(qubits[{}]))\n", q));
                }
            } else if line.contains("@__quantum__qis__z__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.Z(qubits[{}]))\n", q));
                }
            } else if line.contains("@__quantum__qis__s__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.S(qubits[{}]))\n", q));
                }
            } else if line.contains("@__quantum__qis__t__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.T(qubits[{}]))\n", q));
                }
            } else if line.contains("@__quantum__qis__cnot__body") {
                if let Some((ctrl, tgt)) = Self::extract_two_qubits(line) {
                    cirq.push_str(&format!("circuit.append(cirq.CNOT(qubits[{}], qubits[{}]))\n", ctrl, tgt));
                }
            } else if line.contains("@__quantum__qis__cz__body") {
                if let Some((ctrl, tgt)) = Self::extract_two_qubits(line) {
                    cirq.push_str(&format!("circuit.append(cirq.CZ(qubits[{}], qubits[{}]))\n", ctrl, tgt));
                }
            } else if line.contains("@__quantum__qis__swap__body") {
                if let Some((q1, q2)) = Self::extract_two_qubits(line) {
                    cirq.push_str(&format!("circuit.append(cirq.SWAP(qubits[{}], qubits[{}]))\n", q1, q2));
                }
            } else if line.contains("@__quantum__qis__m__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    cirq.push_str(&format!("circuit.append(cirq.measure(qubits[{}], key='m{}'))\n", q, q));
                }
            }
        }
        
        cirq.push_str("\nprint(circuit)\n");
        cirq
    }
    
    /// Translate QIR to Quil (Rigetti)
    pub fn to_quil(qir: &str, num_qubits: usize) -> String {
        let mut quil = String::new();
        quil.push_str("# Generated by TAYNI from QIR\n");
        quil.push_str("# Target: Rigetti Quil\n\n");
        
        // Declare classical memory
        quil.push_str(&format!("DECLARE ro BIT[{}]\n\n", num_qubits));
        
        // Parse QIR and translate gates
        for line in qir.lines() {
            let line = line.trim();
            if line.contains("@__quantum__qis__h__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("H {}\n", q));
                }
            } else if line.contains("@__quantum__qis__x__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("X {}\n", q));
                }
            } else if line.contains("@__quantum__qis__y__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("Y {}\n", q));
                }
            } else if line.contains("@__quantum__qis__z__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("Z {}\n", q));
                }
            } else if line.contains("@__quantum__qis__s__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("S {}\n", q));
                }
            } else if line.contains("@__quantum__qis__t__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("T {}\n", q));
                }
            } else if line.contains("@__quantum__qis__cnot__body") {
                if let Some((ctrl, tgt)) = Self::extract_two_qubits(line) {
                    quil.push_str(&format!("CNOT {} {}\n", ctrl, tgt));
                }
            } else if line.contains("@__quantum__qis__cz__body") {
                if let Some((ctrl, tgt)) = Self::extract_two_qubits(line) {
                    quil.push_str(&format!("CZ {} {}\n", ctrl, tgt));
                }
            } else if line.contains("@__quantum__qis__swap__body") {
                if let Some((q1, q2)) = Self::extract_two_qubits(line) {
                    quil.push_str(&format!("SWAP {} {}\n", q1, q2));
                }
            } else if line.contains("@__quantum__qis__m__body") {
                if let Some(q) = Self::extract_qubit(line) {
                    quil.push_str(&format!("MEASURE {} ro[{}]\n", q, q));
                }
            }
        }
        
        quil
    }
    
    // Helper: extract qubit number from QIR line like "%q0"
    fn extract_qubit(line: &str) -> Option<usize> {
        // Look for %q followed by a number
        if let Some(pos) = line.find("%q") {
            let rest = &line[pos + 2..];
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            num_str.parse().ok()
        } else {
            None
        }
    }
    
    // Helper: extract two qubit numbers from QIR line
    fn extract_two_qubits(line: &str) -> Option<(usize, usize)> {
        let mut qubits = Vec::new();
        let mut pos = 0;
        while let Some(idx) = line[pos..].find("%q") {
            let start = pos + idx + 2;
            let rest = &line[start..];
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = num_str.parse() {
                qubits.push(n);
            }
            pos = start + num_str.len();
        }
        if qubits.len() >= 2 {
            Some((qubits[0], qubits[1]))
        } else {
            None
        }
    }
}
