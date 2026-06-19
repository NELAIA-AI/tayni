//! ARM64 (AArch64) Instruction Encoder for TAYNI
//!
//! Provides low-level ARM64 instruction encoding for direct binary generation.
//! Used by the ELF ARM64 generator.

// ============================================================================
// ARM64 Registers
// ============================================================================

/// General-purpose register (X0-X30, XZR/SP)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg(pub u8);

impl Reg {
    // Argument/result registers
    pub const X0: Reg = Reg(0);
    pub const X1: Reg = Reg(1);
    pub const X2: Reg = Reg(2);
    pub const X3: Reg = Reg(3);
    pub const X4: Reg = Reg(4);
    pub const X5: Reg = Reg(5);
    pub const X6: Reg = Reg(6);
    pub const X7: Reg = Reg(7);
    
    // Indirect result / syscall number
    pub const X8: Reg = Reg(8);
    
    // Temporary registers
    pub const X9: Reg = Reg(9);
    pub const X10: Reg = Reg(10);
    pub const X11: Reg = Reg(11);
    pub const X12: Reg = Reg(12);
    pub const X13: Reg = Reg(13);
    pub const X14: Reg = Reg(14);
    pub const X15: Reg = Reg(15);
    
    // Intra-procedure-call scratch
    pub const X16: Reg = Reg(16);
    pub const X17: Reg = Reg(17);
    
    // Platform register
    pub const X18: Reg = Reg(18);
    
    // Callee-saved registers
    pub const X19: Reg = Reg(19);
    pub const X20: Reg = Reg(20);
    pub const X21: Reg = Reg(21);
    pub const X22: Reg = Reg(22);
    pub const X23: Reg = Reg(23);
    pub const X24: Reg = Reg(24);
    pub const X25: Reg = Reg(25);
    pub const X26: Reg = Reg(26);
    pub const X27: Reg = Reg(27);
    pub const X28: Reg = Reg(28);
    
    // Frame pointer
    pub const X29: Reg = Reg(29);
    pub const FP: Reg = Reg(29);
    
    // Link register
    pub const X30: Reg = Reg(30);
    pub const LR: Reg = Reg(30);
    
    // Zero register / Stack pointer (context dependent)
    pub const XZR: Reg = Reg(31);
    pub const SP: Reg = Reg(31);
}

/// Condition codes for conditional branches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    Eq = 0b0000,  // Equal (Z=1)
    Ne = 0b0001,  // Not equal (Z=0)
    Cs = 0b0010,  // Carry set / unsigned >= (C=1)
    Cc = 0b0011,  // Carry clear / unsigned < (C=0)
    Mi = 0b0100,  // Minus / negative (N=1)
    Pl = 0b0101,  // Plus / positive or zero (N=0)
    Vs = 0b0110,  // Overflow (V=1)
    Vc = 0b0111,  // No overflow (V=0)
    Hi = 0b1000,  // Unsigned > (C=1 && Z=0)
    Ls = 0b1001,  // Unsigned <= (C=0 || Z=1)
    Ge = 0b1010,  // Signed >= (N=V)
    Lt = 0b1011,  // Signed < (N!=V)
    Gt = 0b1100,  // Signed > (Z=0 && N=V)
    Le = 0b1101,  // Signed <= (Z=1 || N!=V)
    Al = 0b1110,  // Always
}

// Aliases
impl Cond {
    pub const Hs: Cond = Cond::Cs; // Unsigned >=
    pub const Lo: Cond = Cond::Cc; // Unsigned <
}

/// Shift type for data processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
    Lsl = 0b00, // Logical shift left
    Lsr = 0b01, // Logical shift right
    Asr = 0b10, // Arithmetic shift right
    Ror = 0b11, // Rotate right
}

// ============================================================================
// ARM64 Instruction Encoder
// ============================================================================

/// ARM64 instruction encoder
pub struct Arm64Encoder {
    pub code: Vec<u8>,
}

impl Arm64Encoder {
    pub fn new() -> Self {
        Arm64Encoder { code: Vec::new() }
    }
    
    /// Emit a 32-bit instruction
    fn emit(&mut self, instr: u32) {
        self.code.extend(&instr.to_le_bytes());
    }
    
    // ========================================================================
    // Data Processing - Immediate
    // ========================================================================
    
    /// MOV Xd, #imm16 (MOVZ)
    pub fn mov_imm(&mut self, rd: Reg, imm: u16) {
        // MOVZ Xd, #imm16, LSL #0
        // 1 10 100101 00 imm16 Rd
        let instr = 0xD2800000 | ((imm as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// MOV Xd, #imm16, LSL #shift (shift = 0, 16, 32, 48)
    pub fn movz(&mut self, rd: Reg, imm: u16, shift: u8) {
        let hw = (shift / 16) as u32;
        let instr = 0xD2800000 | (hw << 21) | ((imm as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// MOVK Xd, #imm16, LSL #shift (keep other bits)
    pub fn movk(&mut self, rd: Reg, imm: u16, shift: u8) {
        let hw = (shift / 16) as u32;
        let instr = 0xF2800000 | (hw << 21) | ((imm as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// Load 64-bit immediate into register
    pub fn mov_imm64(&mut self, rd: Reg, imm: u64) {
        let imm0 = (imm & 0xFFFF) as u16;
        let imm1 = ((imm >> 16) & 0xFFFF) as u16;
        let imm2 = ((imm >> 32) & 0xFFFF) as u16;
        let imm3 = ((imm >> 48) & 0xFFFF) as u16;
        
        self.movz(rd, imm0, 0);
        if imm1 != 0 { self.movk(rd, imm1, 16); }
        if imm2 != 0 { self.movk(rd, imm2, 32); }
        if imm3 != 0 { self.movk(rd, imm3, 48); }
    }
    
    /// ADD Xd, Xn, #imm12
    pub fn add_imm(&mut self, rd: Reg, rn: Reg, imm: u16) {
        // ADD Xd, Xn, #imm12
        // 1 0 0 10001 00 imm12 Rn Rd
        let instr = 0x91000000 | ((imm as u32 & 0xFFF) << 10) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// SUB Xd, Xn, #imm12
    pub fn sub_imm(&mut self, rd: Reg, rn: Reg, imm: u16) {
        let instr = 0xD1000000 | ((imm as u32 & 0xFFF) << 10) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    // ========================================================================
    // Data Processing - Register
    // ========================================================================
    
    /// ADD Xd, Xn, Xm
    pub fn add_reg(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        // ADD Xd, Xn, Xm
        // 1 0 0 01011 00 0 Rm 000000 Rn Rd
        let instr = 0x8B000000 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// SUB Xd, Xn, Xm
    pub fn sub_reg(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        let instr = 0xCB000000 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// MUL Xd, Xn, Xm (alias for MADD Xd, Xn, Xm, XZR)
    pub fn mul_reg(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        // MADD Xd, Xn, Xm, XZR
        // 1 00 11011 000 Rm 0 11111 Rn Rd
        let instr = 0x9B007C00 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// SDIV Xd, Xn, Xm (signed divide)
    pub fn sdiv(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        // SDIV Xd, Xn, Xm
        // 1 0 0 11010110 Rm 00001 1 Rn Rd
        let instr = 0x9AC00C00 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// UDIV Xd, Xn, Xm (unsigned divide)
    pub fn udiv(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        let instr = 0x9AC00800 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// AND Xd, Xn, Xm
    pub fn and_reg(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        let instr = 0x8A000000 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// ORR Xd, Xn, Xm
    pub fn orr_reg(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        let instr = 0xAA000000 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// EOR Xd, Xn, Xm (XOR)
    pub fn eor_reg(&mut self, rd: Reg, rn: Reg, rm: Reg) {
        let instr = 0xCA000000 | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// MOV Xd, Xm (alias for ORR Xd, XZR, Xm)
    pub fn mov_reg(&mut self, rd: Reg, rm: Reg) {
        self.orr_reg(rd, Reg::XZR, rm);
    }
    
    /// CMP Xn, Xm (alias for SUBS XZR, Xn, Xm)
    pub fn cmp_reg(&mut self, rn: Reg, rm: Reg) {
        // SUBS XZR, Xn, Xm
        let instr = 0xEB00001F | ((rm.0 as u32) << 16) | ((rn.0 as u32) << 5);
        self.emit(instr);
    }
    
    /// CMP Xn, #imm12
    pub fn cmp_imm(&mut self, rn: Reg, imm: u16) {
        // SUBS XZR, Xn, #imm12
        let instr = 0xF100001F | ((imm as u32 & 0xFFF) << 10) | ((rn.0 as u32) << 5);
        self.emit(instr);
    }
    
    // ========================================================================
    // Load/Store
    // ========================================================================
    
    /// LDR Xt, [Xn, #imm12*8] (64-bit load)
    pub fn ldr(&mut self, rt: Reg, rn: Reg, offset: i16) {
        let imm12 = ((offset as u32) >> 3) & 0xFFF;
        let instr = 0xF9400000 | (imm12 << 10) | ((rn.0 as u32) << 5) | (rt.0 as u32);
        self.emit(instr);
    }
    
    /// STR Xt, [Xn, #imm12*8] (64-bit store)
    pub fn str(&mut self, rt: Reg, rn: Reg, offset: i16) {
        let imm12 = ((offset as u32) >> 3) & 0xFFF;
        let instr = 0xF9000000 | (imm12 << 10) | ((rn.0 as u32) << 5) | (rt.0 as u32);
        self.emit(instr);
    }
    
    /// LDRB Wt, [Xn, #imm12] (byte load)
    pub fn ldrb(&mut self, rt: Reg, rn: Reg, offset: u16) {
        let instr = 0x39400000 | ((offset as u32 & 0xFFF) << 10) | ((rn.0 as u32) << 5) | (rt.0 as u32);
        self.emit(instr);
    }
    
    /// STRB Wt, [Xn, #imm12] (byte store)
    pub fn strb(&mut self, rt: Reg, rn: Reg, offset: u16) {
        let instr = 0x39000000 | ((offset as u32 & 0xFFF) << 10) | ((rn.0 as u32) << 5) | (rt.0 as u32);
        self.emit(instr);
    }
    
    /// STP Xt1, Xt2, [Xn, #imm7*8]! (store pair, pre-index)
    pub fn stp_pre(&mut self, rt1: Reg, rt2: Reg, rn: Reg, offset: i16) {
        let imm7 = ((offset >> 3) as u32) & 0x7F;
        let instr = 0xA9800000 | (imm7 << 15) | ((rt2.0 as u32) << 10) | ((rn.0 as u32) << 5) | (rt1.0 as u32);
        self.emit(instr);
    }
    
    /// LDP Xt1, Xt2, [Xn], #imm7*8 (load pair, post-index)
    pub fn ldp_post(&mut self, rt1: Reg, rt2: Reg, rn: Reg, offset: i16) {
        let imm7 = ((offset >> 3) as u32) & 0x7F;
        let instr = 0xA8C00000 | (imm7 << 15) | ((rt2.0 as u32) << 10) | ((rn.0 as u32) << 5) | (rt1.0 as u32);
        self.emit(instr);
    }
    
    // ========================================================================
    // Branches
    // ========================================================================
    
    /// B label (unconditional branch)
    pub fn b(&mut self, offset: i32) {
        // B imm26
        // 000101 imm26
        let imm26 = ((offset >> 2) as u32) & 0x3FFFFFF;
        let instr = 0x14000000 | imm26;
        self.emit(instr);
    }
    
    /// BL label (branch with link)
    pub fn bl(&mut self, offset: i32) {
        let imm26 = ((offset >> 2) as u32) & 0x3FFFFFF;
        let instr = 0x94000000 | imm26;
        self.emit(instr);
    }
    
    /// B.cond label (conditional branch)
    pub fn b_cond(&mut self, cond: Cond, offset: i32) {
        // B.cond imm19
        // 01010100 imm19 0 cond
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        let instr = 0x54000000 | (imm19 << 5) | (cond as u32);
        self.emit(instr);
    }
    
    /// CBZ Xt, label (compare and branch if zero)
    pub fn cbz(&mut self, rt: Reg, offset: i32) {
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        let instr = 0xB4000000 | (imm19 << 5) | (rt.0 as u32);
        self.emit(instr);
    }
    
    /// CBNZ Xt, label (compare and branch if not zero)
    pub fn cbnz(&mut self, rt: Reg, offset: i32) {
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        let instr = 0xB5000000 | (imm19 << 5) | (rt.0 as u32);
        self.emit(instr);
    }
    
    /// BR Xn (branch to register)
    pub fn br(&mut self, rn: Reg) {
        let instr = 0xD61F0000 | ((rn.0 as u32) << 5);
        self.emit(instr);
    }
    
    /// BLR Xn (branch with link to register)
    pub fn blr(&mut self, rn: Reg) {
        let instr = 0xD63F0000 | ((rn.0 as u32) << 5);
        self.emit(instr);
    }
    
    /// RET (return, alias for BR X30)
    pub fn ret(&mut self) {
        self.emit(0xD65F03C0);
    }
    
    // ========================================================================
    // System
    // ========================================================================
    
    /// SVC #imm16 (supervisor call / syscall)
    pub fn svc(&mut self, imm: u16) {
        let instr = 0xD4000001 | ((imm as u32) << 5);
        self.emit(instr);
    }
    
    /// NOP
    pub fn nop(&mut self) {
        self.emit(0xD503201F);
    }
    
    // ========================================================================
    // Address Generation
    // ========================================================================
    
    /// ADR Xd, label (PC-relative address, ±1MB)
    pub fn adr(&mut self, rd: Reg, offset: i32) {
        let immlo = (offset & 0x3) as u32;
        let immhi = ((offset >> 2) & 0x7FFFF) as u32;
        let instr = 0x10000000 | (immlo << 29) | (immhi << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    /// ADRP Xd, label (PC-relative page address, ±4GB)
    pub fn adrp(&mut self, rd: Reg, offset: i32) {
        let immlo = ((offset >> 12) & 0x3) as u32;
        let immhi = ((offset >> 14) & 0x7FFFF) as u32;
        let instr = 0x90000000 | (immlo << 29) | (immhi << 5) | (rd.0 as u32);
        self.emit(instr);
    }
    
    // ========================================================================
    // Stack Operations
    // ========================================================================
    
    /// Function prologue: save FP and LR, set up frame
    pub fn prologue(&mut self, frame_size: u16) {
        // STP X29, X30, [SP, #-frame_size]!
        self.stp_pre(Reg::FP, Reg::LR, Reg::SP, -(frame_size as i16));
        // MOV X29, SP
        self.mov_reg(Reg::FP, Reg::SP);
    }
    
    /// Function epilogue: restore FP and LR, return
    pub fn epilogue(&mut self, frame_size: u16) {
        // LDP X29, X30, [SP], #frame_size
        self.ldp_post(Reg::FP, Reg::LR, Reg::SP, frame_size as i16);
        // RET
        self.ret();
    }
    
    // ========================================================================
    // Linux Syscalls
    // ========================================================================
    
    /// Emit write syscall: write(fd, buf, count)
    pub fn syscall_write(&mut self, fd: Reg, buf: Reg, count: Reg) {
        if fd.0 != 0 { self.mov_reg(Reg::X0, fd); }
        if buf.0 != 1 { self.mov_reg(Reg::X1, buf); }
        if count.0 != 2 { self.mov_reg(Reg::X2, count); }
        self.mov_imm(Reg::X8, 64); // __NR_write
        self.svc(0);
    }
    
    /// Emit exit syscall: exit(status)
    pub fn syscall_exit(&mut self, status: Reg) {
        if status.0 != 0 { self.mov_reg(Reg::X0, status); }
        self.mov_imm(Reg::X8, 93); // __NR_exit
        self.svc(0);
    }
    
    /// Emit socket syscall: socket(domain, type, protocol)
    pub fn syscall_socket(&mut self) {
        self.mov_imm(Reg::X8, 198); // __NR_socket
        self.svc(0);
    }
    
    /// Emit bind syscall: bind(fd, addr, addrlen)
    pub fn syscall_bind(&mut self) {
        self.mov_imm(Reg::X8, 200); // __NR_bind
        self.svc(0);
    }
    
    /// Emit listen syscall: listen(fd, backlog)
    pub fn syscall_listen(&mut self) {
        self.mov_imm(Reg::X8, 201); // __NR_listen
        self.svc(0);
    }
    
    /// Emit accept syscall: accept(fd, addr, addrlen)
    pub fn syscall_accept(&mut self) {
        self.mov_imm(Reg::X8, 202); // __NR_accept
        self.svc(0);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mov_imm() {
        let mut enc = Arm64Encoder::new();
        enc.mov_imm(Reg::X0, 42);
        assert_eq!(enc.code.len(), 4);
        // MOV X0, #42 = 0xD2800540
        assert_eq!(enc.code, vec![0x40, 0x05, 0x80, 0xD2]);
    }
    
    #[test]
    fn test_add_reg() {
        let mut enc = Arm64Encoder::new();
        enc.add_reg(Reg::X0, Reg::X1, Reg::X2);
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_branch() {
        let mut enc = Arm64Encoder::new();
        enc.b(0x100); // Branch forward 256 bytes
        assert_eq!(enc.code.len(), 4);
    }
    
    #[test]
    fn test_conditional_branch() {
        let mut enc = Arm64Encoder::new();
        enc.cmp_reg(Reg::X0, Reg::X1);
        enc.b_cond(Cond::Eq, 8); // Branch if equal
        assert_eq!(enc.code.len(), 8);
    }
    
    #[test]
    fn test_syscall() {
        let mut enc = Arm64Encoder::new();
        enc.svc(0);
        assert_eq!(enc.code, vec![0x01, 0x00, 0x00, 0xD4]);
    }
    
    #[test]
    fn test_prologue_epilogue() {
        let mut enc = Arm64Encoder::new();
        enc.prologue(16);
        enc.nop();
        enc.epilogue(16);
        assert_eq!(enc.code.len(), 20); // 5 instructions (stp, mov, nop, ldp, ret)
    }
    
    #[test]
    fn test_hello_world() {
        let mut enc = Arm64Encoder::new();
        
        // write(1, msg, 14)
        enc.mov_imm(Reg::X0, 1);      // fd = stdout
        enc.adr(Reg::X1, 24);         // buf = msg (6 instructions * 4 = 24 bytes ahead)
        enc.mov_imm(Reg::X2, 14);     // count = 14
        enc.mov_imm(Reg::X8, 64);     // syscall = write
        enc.svc(0);
        
        // exit(0)
        enc.mov_imm(Reg::X0, 0);      // status = 0
        enc.mov_imm(Reg::X8, 93);     // syscall = exit
        enc.svc(0);
        
        // Message would go here
        
        assert_eq!(enc.code.len(), 32); // 8 instructions
    }
}
