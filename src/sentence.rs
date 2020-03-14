use std::collections::BTreeMap;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Sentence {
    raw: String,
    tokens: Vec<Token>,
}

impl Sentence {
    pub fn new(raw: String, tokens: Vec<Token>) -> Sentence {
        Sentence { raw, tokens }
    }

    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    pub fn tokens_mut(&mut self) -> &mut [Token] {
        self.tokens.as_mut_slice()
    }

    pub fn by_loc(&self, loc: &Loc) -> String {
        self.raw
            .chars()
            .skip(loc.start_index)
            .take(loc.len)
            .collect()
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Token {
    pub val: String,
    pub loc: Loc,
    meta: BTreeMap<String, Meta>,
}

impl Token {
    pub fn new(val: String, loc: Loc) -> Token {
        Token {
            val,
            loc,
            meta: Default::default(),
        }
    }

    pub fn add_meta(&mut self, key: &str, meta: Meta) {
        self.meta.insert(key.to_owned(), meta);
    }

    pub fn get_meta(&self, key: &str) -> Option<&Meta> {
        self.meta.get(key)
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Loc {
    pub start_index: usize,
    pub len: usize,
}

impl Loc {
    pub fn new(start_index: usize, len: usize) -> Loc {
        Loc { start_index, len }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Meta {
    Bool(bool),
    Str(String),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    Usize(usize),
    I128(i128),
    U128(u128),
    Vec(Vec<Meta>),
    Map(BTreeMap<String, Meta>),
}

impl Meta {
    pub fn as_usize(&self) -> usize {
        match self {
            Meta::Usize(val) => *val,
            _ => panic!("Expected usize value. Found [{:?}]", self),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Meta::I64(val) => *val,
            _ => panic!("Expected i64 value. Found [{:?}]", self),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Meta::U64(val) => *val,
            _ => panic!("Expected u64 value. Found [{:?}]", self),
        }
    }
}
