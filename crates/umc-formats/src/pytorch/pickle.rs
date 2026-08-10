/// Minimal pickle protocol 2/4 parser for PyTorch state_dict files.
/// Handles exactly the patterns produced by torch.save(state_dict, path).
use std::collections::HashMap;
use std::io::{Cursor, Read};

// ── Pickle opcodes (subset used by PyTorch) ───────────────────────────────────

const PROTO: u8 = 0x80;
const FRAME: u8 = 0x95;
const STOP: u8 = b'.';
const POP: u8 = b'0';
const POP_MARK: u8 = b'1';
const MARK: u8 = b'(';
const BININT1: u8 = b'K';
const BININT2: u8 = b'M';
const BININT: u8 = b'J';
const LONG1: u8 = 0x8A;
const NONE: u8 = b'N';
const NEWTRUE: u8 = 0x88;
const NEWFALSE: u8 = 0x89;
const BINUNICODE: u8 = b'X';
const SHORT_BINUNICODE: u8 = 0x8C;
const BINUNICODE8: u8 = 0x8D;
const BINSTRING: u8 = b'T';
const SHORT_BINSTRING: u8 = b'U';
const EMPTY_TUPLE: u8 = b')';
const TUPLE: u8 = b't';
const TUPLE1: u8 = 0x85;
const TUPLE2: u8 = 0x86;
const TUPLE3: u8 = 0x87;
const EMPTY_LIST: u8 = b']';
const APPENDS: u8 = b'e';
const APPEND: u8 = b'a';
const EMPTY_DICT: u8 = b'}';
const DICT: u8 = b'd';
const SETITEM: u8 = b's';
const SETITEMS: u8 = b'u';
const GLOBAL: u8 = b'c';
const REDUCE: u8 = b'R';
const BUILD: u8 = b'b';
const BINGET: u8 = b'h';
const LONG_BINGET: u8 = b'j';
const BINPUT: u8 = b'q';
const LONG_BINPUT: u8 = b'r';
const MEMOIZE: u8 = 0x94;
const BINPERSID: u8 = b'Q';
const NEWOBJ: u8 = 0x81;

// ── Value type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Pv {
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),
    Str(String),
    Bool(bool),
    None,
    List(Vec<Pv>),
    Dict(Vec<(Pv, Pv)>),
    Tuple(Vec<Pv>),
    /// PyTorch storage persistent reference
    StorageRef {
        key: String,
        dtype_class: String,
        numel: usize,
    },
    /// Reconstructed PyTorch tensor descriptor
    PtTensor {
        storage_key: String,
        dtype_class: String,
        storage_offset: usize,
        shape: Vec<usize>,
        stride: Vec<usize>,
    },
    /// Fallback for unknown Global objects
    Global {
        module: String,
        name: String,
    },
    Object {
        class: Box<Pv>,
        args: Box<Pv>,
    },
}

impl Pv {
    pub fn as_str(&self) -> Option<&str> {
        if let Pv::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        if let Pv::Int(n) = self {
            Some(*n)
        } else {
            None
        }
    }
    pub fn as_tuple(&self) -> Option<&[Pv]> {
        if let Pv::Tuple(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_list(&self) -> Option<&[Pv]> {
        if let Pv::List(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_dict(&self) -> Option<&[(Pv, Pv)]> {
        if let Pv::Dict(d) = self {
            Some(d)
        } else {
            None
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

struct Mark;

pub struct PickleParser<'a> {
    data: &'a [u8],
    pos: usize,
    stack: Vec<Pv>,
    mark_stack: Vec<usize>, // stack positions where MARK was placed
    memo: HashMap<u32, Pv>,
}

impl<'a> PickleParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            stack: Vec::new(),
            mark_stack: Vec::new(),
            memo: HashMap::new(),
        }
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }

    fn read_bytes(&mut self, n: usize) -> Option<&[u8]> {
        if self.pos + n <= self.data.len() {
            let s = &self.data[self.pos..self.pos + n];
            self.pos += n;
            Some(s)
        } else {
            None
        }
    }

    fn read_line(&mut self) -> Option<String> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != b'\n' {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.data[start..self.pos])
            .ok()?
            .to_string();
        if self.pos < self.data.len() {
            self.pos += 1;
        } // skip \n
        Some(s)
    }

    fn pop_mark(&mut self) -> Vec<Pv> {
        let mark_pos = self.mark_stack.pop().unwrap_or(0);
        self.stack.drain(mark_pos..).collect()
    }

    fn push(&mut self, v: Pv) {
        self.stack.push(v);
    }

    fn peek(&self) -> Option<&Pv> {
        self.stack.last()
    }

    fn pop(&mut self) -> Option<Pv> {
        self.stack.pop()
    }

    /// Main parse loop. Returns the final value.
    pub fn parse(&mut self) -> Result<Pv, String> {
        loop {
            let op = self.read_u8().ok_or("unexpected end of pickle")?;
            match op {
                PROTO => {
                    let _v = self.read_u8().ok_or("PROTO missing version")?;
                }
                FRAME => {
                    // Skip 8-byte frame size (protocol 4)
                    self.read_bytes(8).ok_or("FRAME missing size")?;
                }
                STOP => {
                    return self.pop().ok_or_else(|| "stack empty at STOP".to_string());
                }
                MARK => {
                    self.mark_stack.push(self.stack.len());
                }
                POP => {
                    self.pop();
                }
                POP_MARK => {
                    self.pop_mark();
                }
                NONE => self.push(Pv::None),
                NEWTRUE => self.push(Pv::Bool(true)),
                NEWFALSE => self.push(Pv::Bool(false)),
                BININT1 => {
                    let v = self.read_u8().ok_or("BININT1 missing byte")? as i64;
                    self.push(Pv::Int(v));
                }
                BININT2 => {
                    let b = self.read_bytes(2).ok_or("BININT2 missing bytes")?;
                    let v = u16::from_le_bytes([b[0], b[1]]) as i64;
                    self.push(Pv::Int(v));
                }
                BININT => {
                    let b = self.read_bytes(4).ok_or("BININT missing bytes")?;
                    let v = i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as i64;
                    self.push(Pv::Int(v));
                }
                LONG1 => {
                    let n = self.read_u8().ok_or("LONG1 missing len")? as usize;
                    let bytes = self.read_bytes(n).ok_or("LONG1 missing data")?;
                    // Reconstruct as i64 (enough for tensor sizes)
                    let mut val: i64 = 0;
                    for (i, &b) in bytes.iter().enumerate().take(8) {
                        val |= (b as i64) << (i * 8);
                    }
                    // Sign extend from n*8 bits
                    if n < 8 && n > 0 && bytes[n - 1] & 0x80 != 0 {
                        val |= !((1i64 << (n * 8)) - 1);
                    }
                    self.push(Pv::Int(val));
                }
                BINUNICODE => {
                    let b = self.read_bytes(4).ok_or("BINUNICODE missing len")?;
                    let n = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
                    let s_bytes = self.read_bytes(n).ok_or("BINUNICODE missing data")?;
                    let s = String::from_utf8_lossy(s_bytes).into_owned();
                    self.push(Pv::Str(s));
                }
                SHORT_BINUNICODE => {
                    let n = self.read_u8().ok_or("SHORT_BINUNICODE missing len")? as usize;
                    let s_bytes = self.read_bytes(n).ok_or("SHORT_BINUNICODE missing data")?;
                    let s = String::from_utf8_lossy(s_bytes).into_owned();
                    self.push(Pv::Str(s));
                }
                BINUNICODE8 => {
                    let b = self.read_bytes(8).ok_or("BINUNICODE8 missing len")?;
                    let n = u64::from_le_bytes(b.try_into().unwrap()) as usize;
                    let s_bytes = self.read_bytes(n).ok_or("BINUNICODE8 missing data")?;
                    let s = String::from_utf8_lossy(s_bytes).into_owned();
                    self.push(Pv::Str(s));
                }
                BINSTRING => {
                    let b = self.read_bytes(4).ok_or("BINSTRING missing len")?;
                    let n = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
                    let data = self.read_bytes(n).ok_or("BINSTRING missing data")?.to_vec();
                    self.push(Pv::Bytes(data));
                }
                SHORT_BINSTRING => {
                    let n = self.read_u8().ok_or("SHORT_BINSTRING missing len")? as usize;
                    let data = self
                        .read_bytes(n)
                        .ok_or("SHORT_BINSTRING missing data")?
                        .to_vec();
                    self.push(Pv::Bytes(data));
                }
                EMPTY_TUPLE => self.push(Pv::Tuple(vec![])),
                TUPLE1 => {
                    let v = self.pop().ok_or("TUPLE1 stack empty")?;
                    self.push(Pv::Tuple(vec![v]));
                }
                TUPLE2 => {
                    let b = self.pop().ok_or("TUPLE2 stack empty (2)")?;
                    let a = self.pop().ok_or("TUPLE2 stack empty (1)")?;
                    self.push(Pv::Tuple(vec![a, b]));
                }
                TUPLE3 => {
                    let c = self.pop().ok_or("TUPLE3 stack empty (3)")?;
                    let b = self.pop().ok_or("TUPLE3 stack empty (2)")?;
                    let a = self.pop().ok_or("TUPLE3 stack empty (1)")?;
                    self.push(Pv::Tuple(vec![a, b, c]));
                }
                TUPLE => {
                    let items = self.pop_mark();
                    self.push(Pv::Tuple(items));
                }
                EMPTY_LIST => self.push(Pv::List(vec![])),
                APPEND => {
                    let v = self.pop().ok_or("APPEND: empty stack")?;
                    if let Some(Pv::List(ref mut lst)) = self.stack.last_mut() {
                        lst.push(v);
                    }
                }
                APPENDS => {
                    let items = self.pop_mark();
                    if let Some(Pv::List(ref mut lst)) = self.stack.last_mut() {
                        lst.extend(items);
                    }
                }
                EMPTY_DICT => self.push(Pv::Dict(vec![])),
                DICT => {
                    let mut items = self.pop_mark();
                    let mut dict: Vec<(Pv, Pv)> = Vec::with_capacity(items.len() / 2);
                    let mut i = 0;
                    while i + 1 < items.len() {
                        let k = items.remove(i);
                        let v = items.remove(i);
                        dict.push((k, v));
                    }
                    self.push(Pv::Dict(dict));
                }
                SETITEM => {
                    let v = self.pop().ok_or("SETITEM missing value")?;
                    let k = self.pop().ok_or("SETITEM missing key")?;
                    if let Some(Pv::Dict(ref mut d)) = self.stack.last_mut() {
                        d.push((k, v));
                    }
                }
                SETITEMS => {
                    let mut items = self.pop_mark();
                    if let Some(Pv::Dict(ref mut d)) = self.stack.last_mut() {
                        let mut i = 0;
                        while i + 1 < items.len() {
                            let k = items.remove(i);
                            let v = items.remove(i);
                            d.push((k, v));
                        }
                    }
                }
                GLOBAL => {
                    let module = self.read_line().ok_or("GLOBAL missing module")?;
                    let name = self.read_line().ok_or("GLOBAL missing name")?;
                    self.push(Pv::Global { module, name });
                }
                REDUCE => {
                    let args = self.pop().ok_or("REDUCE missing args")?;
                    let callable = self.pop().ok_or("REDUCE missing callable")?;
                    // Handle PyTorch-specific reducers
                    let result = self.apply_reduce(callable, args);
                    self.push(result);
                }
                NEWOBJ => {
                    let args = self.pop().ok_or("NEWOBJ missing args")?;
                    let cls = self.pop().ok_or("NEWOBJ missing class")?;
                    self.push(Pv::Object {
                        class: Box::new(cls),
                        args: Box::new(args),
                    });
                }
                BUILD => {
                    let state = self.pop().ok_or("BUILD missing state")?;
                    // For OrderedDict: the top of stack now has the base,
                    // BUILD fills it with the state dict content.
                    // We handle this by replacing the top with the state if it's a dict.
                    if let Pv::Dict(_) = &state {
                        if let Some(top) = self.stack.last_mut() {
                            // OrderedDict with state dict → take state as the result
                            if matches!(top, Pv::Object { .. } | Pv::Global { .. }) {
                                *top = state;
                            }
                        }
                    }
                    // else: no-op for objects we don't handle
                }
                BINGET => {
                    let key = self.read_u8().ok_or("BINGET missing key")? as u32;
                    let v = self
                        .memo
                        .get(&key)
                        .cloned()
                        .ok_or_else(|| format!("BINGET: key {} not in memo", key))?;
                    self.push(v);
                }
                LONG_BINGET => {
                    let b = self.read_bytes(4).ok_or("LONG_BINGET missing key")?;
                    let key = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                    let v = self
                        .memo
                        .get(&key)
                        .cloned()
                        .ok_or_else(|| format!("LONG_BINGET: key {} not in memo", key))?;
                    self.push(v);
                }
                BINPUT => {
                    let key = self.read_u8().ok_or("BINPUT missing key")? as u32;
                    if let Some(v) = self.stack.last() {
                        self.memo.insert(key, v.clone());
                    }
                }
                LONG_BINPUT => {
                    let b = self.read_bytes(4).ok_or("LONG_BINPUT missing key")?;
                    let key = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
                    if let Some(v) = self.stack.last() {
                        self.memo.insert(key, v.clone());
                    }
                }
                MEMOIZE => {
                    let key = self.memo.len() as u32;
                    if let Some(v) = self.stack.last() {
                        self.memo.insert(key, v.clone());
                    }
                }
                BINPERSID => {
                    let pid = self.pop().ok_or("BINPERSID missing id")?;
                    let resolved = self.resolve_persistent_id(pid);
                    self.push(resolved);
                }
                _ => {
                    // Silently skip unknown opcodes to be lenient
                }
            }
        }
    }

    /// Handle PyTorch REDUCE patterns for known classes.
    fn apply_reduce(&self, callable: Pv, args: Pv) -> Pv {
        if let Pv::Global {
            ref module,
            ref name,
        } = callable
        {
            if (module == "torch._utils" || module == "torch") && name == "_rebuild_tensor_v2" {
                if let Pv::Tuple(ref a) = args {
                    return self.rebuild_tensor_v2(a);
                }
            }
            if module == "torch" && name == "_rebuild_parameter" {
                if let Pv::Tuple(ref a) = args {
                    if let Some(tensor) = a.first() {
                        return tensor.clone();
                    }
                }
            }
            if (module == "collections" && name == "OrderedDict")
                || (module == "_codecs" && name == "encode")
            {
                if let Pv::Tuple(ref items) = args {
                    if items.len() == 1 {
                        if let Pv::Dict(_) | Pv::List(_) = &items[0] {
                            return items[0].clone();
                        }
                    }
                }
            }
        }
        Pv::Object {
            class: Box::new(callable),
            args: Box::new(args),
        }
    }

    /// Reconstruct a tensor from _rebuild_tensor_v2 arguments.
    /// Args: (storage, storage_offset, size, stride, requires_grad, backward_hooks, metadata?)
    fn rebuild_tensor_v2(&self, args: &[Pv]) -> Pv {
        if args.len() < 5 {
            return Pv::None;
        }
        let (storage_key, dtype_class) = match &args[0] {
            Pv::StorageRef {
                key, dtype_class, ..
            } => (key.clone(), dtype_class.clone()),
            _ => return Pv::None,
        };
        let storage_offset = args[1].as_i64().unwrap_or(0) as usize;
        let shape: Vec<usize> = match &args[2] {
            Pv::Tuple(v) | Pv::List(v) => v
                .iter()
                .filter_map(|x| x.as_i64())
                .map(|x| x as usize)
                .collect(),
            _ => vec![],
        };
        let stride: Vec<usize> = match &args[3] {
            Pv::Tuple(v) | Pv::List(v) => v
                .iter()
                .filter_map(|x| x.as_i64())
                .map(|x| x as usize)
                .collect(),
            _ => vec![],
        };
        Pv::PtTensor {
            storage_key,
            dtype_class,
            storage_offset,
            shape,
            stride,
        }
    }

    /// Resolve a BINPERSID tuple into a StorageRef.
    /// Format: ('storage', storage_class_instance_or_type, key, device, numel)
    fn resolve_persistent_id(&self, pid: Pv) -> Pv {
        if let Pv::Tuple(items) = pid {
            if items.len() >= 5 {
                let kind = items[0].as_str().unwrap_or("").to_string();
                if kind == "storage" {
                    // items[1] is storage class (could be Global or Object)
                    let dtype_class = match &items[1] {
                        Pv::Global { name, .. } => name.clone(),
                        Pv::Str(s) => s.clone(),
                        _ => "FloatStorage".to_string(),
                    };
                    let key = items[2].as_str().unwrap_or("0").to_string();
                    let numel = items[4].as_i64().unwrap_or(0) as usize;
                    return Pv::StorageRef {
                        key,
                        dtype_class,
                        numel,
                    };
                }
            }
        }
        Pv::None
    }
}

/// Parse a pickle byte stream, returning the top-level value.
pub fn parse(data: &[u8]) -> Result<Pv, String> {
    let mut p = PickleParser::new(data);
    p.parse()
}
