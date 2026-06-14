//! NELAIA Binary Format (.nbin) - Serialization/Deserialization
//! AI-native graph format for direct memory loading

use crate::ir::{Graph, Node, Op, Arg, Value, FlowDest};
use std::collections::HashMap;
use std::io::{Read, Write, Cursor};

const MAGIC: &[u8; 4] = b"NBIN";
const VERSION: u16 = 1;

/// Op code mapping for binary format
fn op_to_byte(op: &Op) -> u8 {
    match op {
        // Arithmetic 0x00-0x0F
        Op::Add => 0x00, Op::Sub => 0x01, Op::Mul => 0x02, Op::Div => 0x03, Op::Mod => 0x04,
        Op::Neg => 0x05,
        // Comparison 0x10-0x1F
        Op::Eq => 0x10, Op::Ne => 0x11, Op::Lt => 0x12, Op::Gt => 0x13, Op::Le => 0x14, Op::Ge => 0x15,
        // Logic 0x18-0x1F
        Op::And => 0x18, Op::Or => 0x19, Op::Not => 0x1A,
        // Memory 0x20-0x2F
        Op::Alc => 0x20, Op::Fre => 0x21, Op::Put => 0x22, Op::Get => 0x23,
        Op::Cpy => 0x24, Op::Cmp => 0x25, Op::Fnd => 0x26, Op::Sln => 0x27,
        // I/O 0x30-0x3F
        Op::Prt => 0x30, Op::Inp => 0x31, Op::Fop => 0x32, Op::Frd => 0x33, Op::Fwr => 0x34, Op::Fcl => 0x35,
        Op::Err => 0x36,
        // Network 0x40-0x4F
        Op::Tcp => 0x40, Op::Udp => 0x41, Op::Bnd => 0x42, Op::Lst => 0x43,
        Op::Acc => 0x44, Op::Con => 0x45, Op::Xmt => 0x46, Op::Rcv => 0x47, Op::Cls => 0x48,
        Op::Sel => 0x49, Op::Rdy => 0x4A, Op::Nbk => 0x4B,
        Op::Ndl => 0x4C, Op::Qck => 0x4D, Op::Sbf => 0x4E, Op::Kal => 0x4F,
        // Threading 0x50-0x5F
        Op::Thr => 0x50, Op::Jon => 0x51, Op::Mtx => 0x52, Op::Lck => 0x53, Op::Ulk => 0x54,
        Op::Que => 0x55, Op::Psh => 0x56, Op::Pop => 0x57,
        // Vectors 0x60-0x6F
        Op::Vec => 0x60, Op::Vph => 0x61, Op::Vgt => 0x62, Op::Vst => 0x63, Op::Vln => 0x64, Op::Vcp => 0x65,
        // HashMap 0x68-0x6F
        Op::Hmp => 0x68, Op::Hpt => 0x69, Op::Hgt => 0x6A, Op::Hhs => 0x6B,
        // String ops 0x6C-0x6F
        Op::Cat => 0x6C, Op::Its => 0x6D, Op::Chr => 0x6E, Op::Sbs => 0x6F,
        // Control 0x70-0x7F
        Op::Brn => 0x70, Op::Ret => 0x71, Op::Jmp => 0x72, Op::Whl => 0x73, Op::End => 0x74, Op::Trn => 0x75,
        // GUI 0x80-0x8F
        Op::Win => 0x80, Op::Shw => 0x81, Op::Hid => 0x82, Op::Evt => 0x83, Op::Run => 0x84,
        Op::Lbl => 0x85, Op::Txb => 0x86, Op::Btn => 0x87, Op::Dlg => 0x88,
        Op::Gvl => 0x89, Op::Svl => 0x8A,
        // Epoll 0x90-0x9F
        Op::Epl => 0x90, Op::Ect => 0x91, Op::Ewa => 0x92,
        // Collections (legacy)
        Op::Seq => 0xA0, Op::Map => 0xA1, Op::Fld => 0xA2, Op::Flt => 0xA3,
        Op::Len => 0xA4, Op::Fst => 0xA5, Op::Snd => 0xA6,
        // Other
        Op::Opn => 0xB0, Op::Chk => 0xB1,
        Op::Call(_) => 0xFF,
    }
}

fn byte_to_op(b: u8) -> Option<Op> {
    Some(match b {
        0x00 => Op::Add, 0x01 => Op::Sub, 0x02 => Op::Mul, 0x03 => Op::Div, 0x04 => Op::Mod,
        0x05 => Op::Neg,
        0x10 => Op::Eq, 0x11 => Op::Ne, 0x12 => Op::Lt, 0x13 => Op::Gt, 0x14 => Op::Le, 0x15 => Op::Ge,
        0x18 => Op::And, 0x19 => Op::Or, 0x1A => Op::Not,
        0x20 => Op::Alc, 0x21 => Op::Fre, 0x22 => Op::Put, 0x23 => Op::Get,
        0x24 => Op::Cpy, 0x25 => Op::Cmp, 0x26 => Op::Fnd, 0x27 => Op::Sln,
        0x30 => Op::Prt, 0x31 => Op::Inp, 0x32 => Op::Fop, 0x33 => Op::Frd, 0x34 => Op::Fwr, 0x35 => Op::Fcl,
        0x36 => Op::Err,
        0x40 => Op::Tcp, 0x41 => Op::Udp, 0x42 => Op::Bnd, 0x43 => Op::Lst,
        0x44 => Op::Acc, 0x45 => Op::Con, 0x46 => Op::Xmt, 0x47 => Op::Rcv, 0x48 => Op::Cls,
        0x49 => Op::Sel, 0x4A => Op::Rdy, 0x4B => Op::Nbk,
        0x4C => Op::Ndl, 0x4D => Op::Qck, 0x4E => Op::Sbf, 0x4F => Op::Kal,
        0x50 => Op::Thr, 0x51 => Op::Jon, 0x52 => Op::Mtx, 0x53 => Op::Lck, 0x54 => Op::Ulk,
        0x55 => Op::Que, 0x56 => Op::Psh, 0x57 => Op::Pop,
        0x60 => Op::Vec, 0x61 => Op::Vph, 0x62 => Op::Vgt, 0x63 => Op::Vst, 0x64 => Op::Vln, 0x65 => Op::Vcp,
        0x68 => Op::Hmp, 0x69 => Op::Hpt, 0x6A => Op::Hgt, 0x6B => Op::Hhs,
        0x6C => Op::Cat, 0x6D => Op::Its, 0x6E => Op::Chr, 0x6F => Op::Sbs,
        0x70 => Op::Brn, 0x71 => Op::Ret, 0x72 => Op::Jmp, 0x73 => Op::Whl, 0x74 => Op::End, 0x75 => Op::Trn,
        0x80 => Op::Win, 0x81 => Op::Shw, 0x82 => Op::Hid, 0x83 => Op::Evt, 0x84 => Op::Run,
        0x85 => Op::Lbl, 0x86 => Op::Txb, 0x87 => Op::Btn, 0x88 => Op::Dlg,
        0x89 => Op::Gvl, 0x8A => Op::Svl,
        0x90 => Op::Epl, 0x91 => Op::Ect, 0x92 => Op::Ewa,
        _ => return None,
    })
}

/// Serialize a Graph to binary format
pub fn serialize(graph: &Graph) -> Vec<u8> {
    let mut out = Vec::new();
    let mut strings: Vec<String> = Vec::new();
    let mut string_map: HashMap<String, u32> = HashMap::new();
    
    // Helper to intern strings
    let mut intern = |s: &str| -> u32 {
        if let Some(&idx) = string_map.get(s) {
            idx
        } else {
            let idx = strings.len() as u32;
            strings.push(s.to_string());
            string_map.insert(s.to_string(), idx);
            idx
        }
    };
    
    // Collect all nodes and strings
    let mut node_data: Vec<Vec<u8>> = Vec::new();
    
    for node in &graph.nodes {
        let mut nd = Vec::new();
        match node {
            Node::Literal { id, value } => {
                nd.push(0); // type = Literal
                nd.push(0); // no op
                let id_idx = intern(id);
                nd.extend_from_slice(&id_idx.to_le_bytes());
                match value {
                    Value::Int(n) => {
                        nd.push(1); // arg type = int
                        nd.extend_from_slice(&n.to_le_bytes());
                    }
                    Value::String(s) => {
                        nd.push(2); // arg type = string
                        let s_idx = intern(s);
                        nd.extend_from_slice(&s_idx.to_le_bytes());
                    }
                    Value::Float(f) => {
                        nd.push(3); // arg type = float
                        nd.extend_from_slice(&f.to_le_bytes());
                    }
                    _ => {}
                }
            }
            Node::Operation { id, op, args } => {
                nd.push(1); // type = Operation
                nd.push(op_to_byte(op));
                let id_idx = intern(id);
                nd.extend_from_slice(&id_idx.to_le_bytes());
                nd.extend_from_slice(&(args.len() as u16).to_le_bytes());
                for arg in args {
                    match arg {
                        Arg::Ref(r) => {
                            nd.push(0); // ref
                            let r_idx = intern(r);
                            nd.extend_from_slice(&r_idx.to_le_bytes());
                        }
                        Arg::Lit(Value::Int(n)) => {
                            nd.push(1); // int
                            nd.extend_from_slice(&n.to_le_bytes());
                        }
                        Arg::Lit(Value::String(s)) => {
                            nd.push(2); // string
                            let s_idx = intern(s);
                            nd.extend_from_slice(&s_idx.to_le_bytes());
                        }
                        _ => {}
                    }
                }
            }
            Node::Flow { source, dest } => {
                nd.push(2); // type = Flow
                nd.push(0);
                // Source
                match source {
                    Arg::Ref(r) => {
                        nd.push(0);
                        let r_idx = intern(r);
                        nd.extend_from_slice(&r_idx.to_le_bytes());
                    }
                    _ => {}
                }
                // Dest
                match dest {
                    FlowDest::Node(n) => {
                        nd.push(0);
                        let n_idx = intern(n);
                        nd.extend_from_slice(&n_idx.to_le_bytes());
                    }
                    FlowDest::CyclicNode(n) => {
                        nd.push(1); // cyclic
                        let n_idx = intern(n);
                        nd.extend_from_slice(&n_idx.to_le_bytes());
                    }
                    _ => {}
                }
            }
            Node::Reference { id, target } => {
                nd.push(3); // type = Reference
                nd.push(0);
                let id_idx = intern(id);
                nd.extend_from_slice(&id_idx.to_le_bytes());
                let t_idx = intern(target);
                nd.extend_from_slice(&t_idx.to_le_bytes());
            }
            _ => {}
        }
        if !nd.is_empty() {
            node_data.push(nd);
        }
    }
    
    // Build string table
    let mut string_table = Vec::new();
    string_table.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    for s in &strings {
        let bytes = s.as_bytes();
        string_table.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        string_table.extend_from_slice(bytes);
    }
    
    // Calculate offsets
    let header_size = 16;
    let nodes_size: usize = node_data.iter().map(|n| n.len() + 2).sum(); // +2 for length prefix
    let string_table_offset = header_size + nodes_size;
    
    // Write header
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&(node_data.len() as u32).to_le_bytes());
    out.extend_from_slice(&(string_table_offset as u32).to_le_bytes());
    
    // Write nodes
    for nd in &node_data {
        out.extend_from_slice(&(nd.len() as u16).to_le_bytes());
        out.extend_from_slice(nd);
    }
    
    // Write string table
    out.extend_from_slice(&string_table);
    
    out
}

/// Deserialize binary format to Graph
pub fn deserialize(data: &[u8]) -> Result<Graph, String> {
    if data.len() < 16 {
        return Err("Invalid binary: too short".to_string());
    }
    
    // Check magic
    if &data[0..4] != MAGIC {
        return Err("Invalid binary: bad magic".to_string());
    }
    
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != VERSION {
        return Err(format!("Unsupported version: {}", version));
    }
    
    let node_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let string_table_offset = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
    
    // Read string table first
    let mut strings: Vec<String> = Vec::new();
    let mut pos = string_table_offset;
    if pos + 4 > data.len() {
        return Err("Invalid string table".to_string());
    }
    let string_count = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
    pos += 4;
    
    for _ in 0..string_count {
        if pos + 2 > data.len() {
            return Err("Invalid string entry".to_string());
        }
        let len = u16::from_le_bytes([data[pos], data[pos+1]]) as usize;
        pos += 2;
        if pos + len > data.len() {
            return Err("Invalid string data".to_string());
        }
        let s = String::from_utf8_lossy(&data[pos..pos+len]).to_string();
        strings.push(s);
        pos += len;
    }
    
    // Read nodes
    let mut graph = Graph::new();
    pos = 16; // after header
    
    for _ in 0..node_count {
        if pos + 2 > string_table_offset {
            break;
        }
        let node_len = u16::from_le_bytes([data[pos], data[pos+1]]) as usize;
        pos += 2;
        
        if pos + node_len > string_table_offset || node_len < 2 {
            break;
        }
        
        let node_type = data[pos];
        let op_byte = data[pos + 1];
        let mut npos = pos + 2;
        
        match node_type {
            0 => { // Literal
                if npos + 4 > pos + node_len { continue; }
                let id_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                npos += 4;
                let id = strings.get(id_idx).cloned().unwrap_or_default();
                
                if npos < pos + node_len {
                    let arg_type = data[npos];
                    npos += 1;
                    let value = match arg_type {
                        1 => { // int
                            if npos + 8 <= pos + node_len {
                                let n = i64::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3],
                                                           data[npos+4], data[npos+5], data[npos+6], data[npos+7]]);
                                Value::Int(n)
                            } else { Value::Int(0) }
                        }
                        2 => { // string
                            if npos + 4 <= pos + node_len {
                                let s_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                                Value::String(strings.get(s_idx).cloned().unwrap_or_default())
                            } else { Value::String(String::new()) }
                        }
                        _ => Value::Int(0),
                    };
                    graph.add_node(Node::Literal { id, value });
                }
            }
            1 => { // Operation
                if npos + 6 > pos + node_len { continue; }
                let id_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                npos += 4;
                let id = strings.get(id_idx).cloned().unwrap_or_default();
                let arg_count = u16::from_le_bytes([data[npos], data[npos+1]]) as usize;
                npos += 2;
                
                let op = byte_to_op(op_byte).unwrap_or(Op::Add);
                let mut args = Vec::new();
                
                for _ in 0..arg_count {
                    if npos >= pos + node_len { break; }
                    let arg_type = data[npos];
                    npos += 1;
                    match arg_type {
                        0 => { // ref
                            if npos + 4 <= pos + node_len {
                                let r_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                                npos += 4;
                                args.push(Arg::Ref(strings.get(r_idx).cloned().unwrap_or_default()));
                            }
                        }
                        1 => { // int
                            if npos + 8 <= pos + node_len {
                                let n = i64::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3],
                                                           data[npos+4], data[npos+5], data[npos+6], data[npos+7]]);
                                npos += 8;
                                args.push(Arg::Lit(Value::Int(n)));
                            }
                        }
                        2 => { // string
                            if npos + 4 <= pos + node_len {
                                let s_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                                npos += 4;
                                args.push(Arg::Lit(Value::String(strings.get(s_idx).cloned().unwrap_or_default())));
                            }
                        }
                        _ => {}
                    }
                }
                
                graph.add_node(Node::Operation { id, op, args });
            }
            3 => { // Reference
                if npos + 8 > pos + node_len { continue; }
                let id_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                npos += 4;
                let t_idx = u32::from_le_bytes([data[npos], data[npos+1], data[npos+2], data[npos+3]]) as usize;
                let id = strings.get(id_idx).cloned().unwrap_or_default();
                let target = strings.get(t_idx).cloned().unwrap_or_default();
                graph.add_node(Node::Reference { id, target });
            }
            _ => {}
        }
        
        pos += node_len;
    }
    
    Ok(graph)
}

/// Check if data is binary format
pub fn is_binary(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == MAGIC
}
