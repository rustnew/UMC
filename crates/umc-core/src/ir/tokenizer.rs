#[derive(Debug, Clone)]
pub enum TokenizerType {
    BPE,
    SentencePiece,
    TikToken,
    WordPiece,
    Unknown(String),
}

/// Tokenizer data preserved in the IR.
#[derive(Debug, Clone)]
pub struct TokenizerStore {
    pub tokenizer_type: TokenizerType,
    pub vocab_size: usize,
    pub bos_token: Option<String>,
    pub eos_token: Option<String>,
    pub pad_token: Option<String>,
    pub unk_token: Option<String>,
    pub chat_template: Option<String>,
    /// Raw tokenizer blob (preserved verbatim for round-trip).
    pub raw_blob: Option<Vec<u8>>,
    /// Vocabulary: token string → token ID.
    pub vocab: indexmap::IndexMap<String, u32>,
    /// Merges (BPE).
    pub merges: Vec<String>,
}

impl TokenizerStore {
    pub fn new(tokenizer_type: TokenizerType, vocab_size: usize) -> Self {
        Self {
            tokenizer_type,
            vocab_size,
            bos_token: None,
            eos_token: None,
            pad_token: None,
            unk_token: None,
            chat_template: None,
            raw_blob: None,
            vocab: indexmap::IndexMap::new(),
            merges: Vec::new(),
        }
    }
}
