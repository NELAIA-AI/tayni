//! ARM64 Code Generator for TAYNI
//!
//! Generates ARM64 machine code from TAYNI IR.

use crate::arm64::*;
use std::collections::HashMap;

// ============================================================================
// Register Allocation
// ============================================================================

/// Register allocator for ARM64
pub struct RegAllocator {
    /// Available temporary registers
    available: Vec<Reg>,
    /// Currently allocated registers
    allocated: HashMap<u32, Reg>,
    /// Spill slots on stack
    spill_offset: i32,
}

impl RegAllocator {
    pub fn new() -> Self {
        RegAllocator {
            available: vec![
                Reg::X9, Reg::X10, Reg::X11, Reg::X12,
                Reg::X13, Reg::X14, Reg::X15,
            ],
            allocated: HashMap::new(),
            spill_offset: 0,
        }
    }
    
    /// Allocate a register for a virtual register
    pub fn alloc(&mut self, vreg: u32) -> Option<Reg> {
        if let Some(&reg) = self.allocated.get(&vreg) {
            return Some(reg);
        }
        
        if let Some(reg) = self.available.pop() {
            self.allocated.insert(vreg, reg);
            Some(reg)
        } else {
            None // Need to spill
        }
    }
    
    /// Free a register
    pub fn free(&mut self, vreg: u32) {
        if let Some(reg) = self.allocated.remove(&vreg) {
            self.available.push(reg);
        }
    }
    
    /// Get register for virtual register (must be allocated)
    pub fn get(&self, vreg: u32) -> Option<Reg> {
        self.allocated.get(&vreg).copied()
    }
    
    /// Allocate spill slot
    pub fn alloc_spill(&mut self) -> i32 {
        self.spill_offset -= 8;
        self.spill_offset
    }
}

// ============================================================================
// IR Types (simplified)
// ============================================================================

/// Virtual register
pub type VReg = u32;

/// IR Operation
#[derive(Debug, Clone)]
pub enum IROp {
    /// Load immediate
    LoadImm(VReg, i64),
    /// Load from memory
    Load(VReg, VReg, i32),
    /// Store to memory
    Store(VReg, VReg, i32),
    /// Add
    Add(VReg, VReg, VReg),
    /// Subtract
    Sub(VReg, VReg, VReg),
    /// Multiply
    Mul(VReg, VReg, VReg),
    /// Divide (signed)
    Div(VReg, VReg, VReg),
    /// Modulo
    Mod(VReg, VReg, VReg),
    /// Bitwise AND
    And(VReg, VReg, VReg),
    /// Bitwise OR
    Or(VReg, VReg, VReg),
    /// Bitwise XOR
    Xor(VReg, VReg, VReg),
    /// Shift left
    Shl(VReg, VReg, VReg),
    /// Shift right (arithmetic)
    Shr(VReg, VReg, VReg),
    /// Compare and set flags
    Cmp(VReg, VReg),
    /// Conditional move
    CMov(VReg, VReg, Cond),
    /// Call function
    Call(VReg, String, Vec<VReg>),
    /// Return
    Ret(Option<VReg>),
    /// Jump
    Jump(String),
    /// Conditional jump
    JumpIf(Cond, String),
    /// Label
    Label(String),
    /// Syscall
    Syscall(u16),
}

/// IR Function
#[derive(Debug, Clone)]
pub struct IRFunc {
    pub name: String,
    pub params: Vec<VReg>,
    pub body: Vec<IROp>,
    pub stack_size: u32,
}

// ============================================================================
// Code Generator
// ============================================================================

/// ARM64 code generator
pub struct Arm64CodeGen {
    encoder: Arm64Encoder,
    allocator: RegAllocator,
    labels: HashMap<String, usize>,
    fixups: Vec<(usize, String, FixupKind)>,
}

#[derive(Debug, Clone)]
enum FixupKind {
    Branch,
    BranchCond(Cond),
    Call,
}

impl Arm64CodeGen {
    pub fn new() -> Self {
        Arm64CodeGen {
            encoder: Arm64Encoder::new(),
            allocator: RegAllocator::new(),
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }
    
    /// Generate code for a function
    pub fn gen_function(&mut self, func: &IRFunc) {
        // Prologue
        let frame_size = ((func.stack_size + 15) & !15) as u16;
        if frame_size > 0 {
            self.encoder.prologue(frame_size);
        }
        
        // Move parameters to allocated registers
        for (i, &param) in func.params.iter().enumerate() {
            if i < 8 {
                let arg_reg = Reg(i as u8);
                if let Some(dst) = self.allocator.alloc(param) {
                    if dst != arg_reg {
                        self.encoder.mov_reg(dst, arg_reg);
                    }
                }
            }
        }
        
        // Generate body
        for op in &func.body {
            self.gen_op(op.clone());
        }
        
        // Epilogue (if not already returned)
        if frame_size > 0 {
            self.encoder.epilogue(frame_size);
        }
        
        // Apply fixups
        self.apply_fixups();
    }
    
    /// Generate code for an operation
    fn gen_op(&mut self, op: IROp) {
        match op {
            IROp::LoadImm(dst, imm) => {
                if let Some(rd) = self.allocator.alloc(dst) {
                    if imm >= 0 && imm <= 0xFFFF {
                        self.encoder.mov_imm(rd, imm as u16);
                    } else {
                        self.encoder.mov_imm64(rd, imm as u64);
                    }
                }
            }
            
            IROp::Load(dst, base, offset) => {
                if let (Some(rd), Some(rn)) = (self.allocator.alloc(dst), self.allocator.get(base)) {
                    self.encoder.ldr(rd, rn, offset as i16);
                }
            }
            
            IROp::Store(src, base, offset) => {
                if let (Some(rt), Some(rn)) = (self.allocator.get(src), self.allocator.get(base)) {
                    self.encoder.str(rt, rn, offset as i16);
                }
            }
            
            IROp::Add(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.add_reg(rd, rn, rm));
            }
            
            IROp::Sub(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.sub_reg(rd, rn, rm));
            }
            
            IROp::Mul(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.mul_reg(rd, rn, rm));
            }
            
            IROp::Div(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.sdiv(rd, rn, rm));
            }
            
            IROp::Mod(dst, lhs, rhs) => {
                // ARM64 doesn't have mod, use: dst = lhs - (lhs / rhs) * rhs
                if let (Some(rd), Some(rn), Some(rm)) = (
                    self.allocator.alloc(dst),
                    self.allocator.get(lhs),
                    self.allocator.get(rhs)
                ) {
                    self.encoder.sdiv(Reg::X16, rn, rm);
                    self.encoder.mul_reg(Reg::X16, Reg::X16, rm);
                    self.encoder.sub_reg(rd, rn, Reg::X16);
                }
            }
            
            IROp::And(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.and_reg(rd, rn, rm));
            }
            
            IROp::Or(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.orr_reg(rd, rn, rm));
            }
            
            IROp::Xor(dst, lhs, rhs) => {
                self.gen_binop(dst, lhs, rhs, |enc, rd, rn, rm| enc.eor_reg(rd, rn, rm));
            }
            
            IROp::Shl(dst, lhs, rhs) => {
                // LSL Xd, Xn, Xm
                if let (Some(rd), Some(rn), Some(rm)) = (
                    self.allocator.alloc(dst),
                    self.allocator.get(lhs),
                    self.allocator.get(rhs)
                ) {
                    // LSLV Xd, Xn, Xm: 1 0 0 11010110 Rm 0010 00 Rn Rd
                    let instr = 0x9AC02000 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
                    self.encoder.code.extend(&instr.to_le_bytes());
                }
            }
            
            IROp::Shr(dst, lhs, rhs) => {
                // ASR Xd, Xn, Xm
                if let (Some(rd), Some(rn), Some(rm)) = (
                    self.allocator.alloc(dst),
                    self.allocator.get(lhs),
                    self.allocator.get(rhs)
                ) {
                    // ASRV Xd, Xn, Xm: 1 0 0 11010110 Rm 0010 10 Rn Rd
                    let instr = 0x9AC02800 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
                    self.encoder.code.extend(&instr.to_le_bytes());
                }
            }
            
            IROp::Cmp(lhs, rhs) => {
                if let (Some(rn), Some(rm)) = (self.allocator.get(lhs), self.allocator.get(rhs)) {
                    self.encoder.cmp_reg(rn, rm);
                }
            }
            
            IROp::CMov(dst, src, cond) => {
                if let (Some(rd), Some(rn)) = (self.allocator.alloc(dst), self.allocator.get(src)) {
                    // CSEL Xd, Xn, Xd, cond
                    let instr = 0x9A800000 | ((rn.0 as u32) << 16) | ((cond as u32) << 12) | ((rd.0 as u32) << 5) | (rd.0 as u32);
                    self.encoder.code.extend(&instr.to_le_bytes());
                }
            }
            
            IROp::Call(_dst, name, args) => {
                // Move args to X0-X7
                for (i, &arg) in args.iter().enumerate().take(8) {
                    if let Some(src) = self.allocator.get(arg) {
                        let dst = Reg(i as u8);
                        if src != dst {
                            self.encoder.mov_reg(dst, src);
                        }
                    }
                }
                
                // Record fixup for call
                let pos = self.encoder.code.len();
                self.fixups.push((pos, name, FixupKind::Call));
                self.encoder.bl(0); // Placeholder
            }
            
            IROp::Ret(val) => {
                if let Some(vreg) = val {
                    if let Some(src) = self.allocator.get(vreg) {
                        if src != Reg::X0 {
                            self.encoder.mov_reg(Reg::X0, src);
                        }
                    }
                }
                self.encoder.ret();
            }
            
            IROp::Jump(label) => {
                let pos = self.encoder.code.len();
                self.fixups.push((pos, label, FixupKind::Branch));
                self.encoder.b(0); // Placeholder
            }
            
            IROp::JumpIf(cond, label) => {
                let pos = self.encoder.code.len();
                self.fixups.push((pos, label, FixupKind::BranchCond(cond)));
                self.encoder.b_cond(cond, 0); // Placeholder
            }
            
            IROp::Label(name) => {
                self.labels.insert(name, self.encoder.code.len());
            }
            
            IROp::Syscall(num) => {
                self.encoder.mov_imm(Reg::X8, num);
                self.encoder.svc(0);
            }
        }
    }
    
    /// Generate binary operation
    fn gen_binop<F>(&mut self, dst: VReg, lhs: VReg, rhs: VReg, f: F)
    where
        F: FnOnce(&mut Arm64Encoder, Reg, Reg, Reg),
    {
        if let (Some(rd), Some(rn), Some(rm)) = (
            self.allocator.alloc(dst),
            self.allocator.get(lhs),
            self.allocator.get(rhs)
        ) {
            f(&mut self.encoder, rd, rn, rm);
        }
    }
    
    /// Apply branch fixups
    fn apply_fixups(&mut self) {
        for (pos, label, kind) in &self.fixups {
            if let Some(&target) = self.labels.get(label) {
                let offset = (target as i32) - (*pos as i32);
                
                match kind {
                    FixupKind::Branch => {
                        let imm26 = ((offset >> 2) as u32) & 0x3FFFFFF;
                        let instr = 0x14000000 | imm26;
                        self.encoder.code[*pos..*pos + 4].copy_from_slice(&instr.to_le_bytes());
                    }
                    FixupKind::BranchCond(cond) => {
                        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
                        let instr = 0x54000000 | (imm19 << 5) | (*cond as u32);
                        self.encoder.code[*pos..*pos + 4].copy_from_slice(&instr.to_le_bytes());
                    }
                    FixupKind::Call => {
                        let imm26 = ((offset >> 2) as u32) & 0x3FFFFFF;
                        let instr = 0x94000000 | imm26;
                        self.encoder.code[*pos..*pos + 4].copy_from_slice(&instr.to_le_bytes());
                    }
                }
            }
        }
    }
    
    /// Get generated code
    pub fn finish(self) -> Vec<u8> {
        self.encoder.code
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reg_allocator() {
        let mut alloc = RegAllocator::new();
        let r1 = alloc.alloc(0).unwrap();
        let r2 = alloc.alloc(1).unwrap();
        assert_ne!(r1, r2);
        
        alloc.free(0);
        let r3 = alloc.alloc(2).unwrap();
        assert_eq!(r1, r3);
    }
    
    #[test]
    fn test_gen_load_imm() {
        let mut gen = Arm64CodeGen::new();
        gen.gen_op(IROp::LoadImm(0, 42));
        assert!(gen.encoder.code.len() >= 4);
    }
    
    #[test]
    fn test_gen_add() {
        let mut gen = Arm64CodeGen::new();
        gen.allocator.alloc(0);
        gen.allocator.alloc(1);
        gen.gen_op(IROp::Add(2, 0, 1));
        assert!(gen.encoder.code.len() >= 4);
    }
    
    #[test]
    fn test_gen_function() {
        let func = IRFunc {
            name: "add".to_string(),
            params: vec![0, 1],
            body: vec![
                IROp::Add(2, 0, 1),
                IROp::Ret(Some(2)),
            ],
            stack_size: 0,
        };
        
        let mut gen = Arm64CodeGen::new();
        gen.gen_function(&func);
        let code = gen.finish();
        assert!(code.len() > 0);
    }
    
    #[test]
    fn test_gen_branch() {
        let mut gen = Arm64CodeGen::new();
        gen.gen_op(IROp::Label("start".to_string()));
        gen.gen_op(IROp::Jump("start".to_string()));
        gen.apply_fixups();
        assert!(gen.encoder.code.len() >= 4);
    }
    
    #[test]
    fn test_gen_syscall() {
        let mut gen = Arm64CodeGen::new();
        gen.gen_op(IROp::Syscall(64)); // write
        assert!(gen.encoder.code.len() >= 8);
    }
}
