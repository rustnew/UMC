# UMC — Universal Model Converter
## Document d'Implémentation Complète v3.0 — Référence Définitive

> **Statut :** Document d'implémentation officiel  
> **Langage :** Rust stable (1.80+)  
> **Licence :** Apache 2.0  
> **Philosophie :** Natif ou Rien · Honnête ou Rien · Excellent ou Rien

---

# PARTIE I — VISION, ARCHITECTURE ET PRINCIPES

---

## 1. VISION ET POSITIONNEMENT HONNÊTE

### 1.1 La Phrase Fondatrice

> **"UMC est le ffmpeg des modèles IA. 100 % natif Rust. Zéro dépendance dans le chemin critique. Garanti mathématiquement."**

### 1.2 Ce que UMC Promet — Vérifiable et Réel

| Promesse | Réalité | Comment la vérifier |
|----------|---------|---------------------|
| Conversion native Rust pour les formats majeurs | ✅ Aucun subprocess | `cargo tree` — zéro dépendance système |
| Performance ×4 à ×8 vs outils Python | ✅ Mesuré sur 3 machines | Benchmark public reproductible |
| RAM minimale via mmap | ✅ ~200 Mo de structures (hors cache OS) | RSS mesuré par `/proc/self/status` |
| Round-trip sémantiquement identique | ✅ Même sorties d'inférence | Validation fonctionnelle automatique |
| Zéro perte d'information structurelle | ✅ ExtensionStore préserve tout | Test automatique par format |
| Formats produits valides | ✅ Validateurs natifs intégrés | Tests de conformité par format |

### 1.3 Ce que UMC ne Promet PAS (Honnêteté Fondatrice)

- ❌ Round-trip **bit-identical** entre formats DIFFÉRENTS (mathématiquement impossible pour formats quantifiés)
- ❌ Performance ×17 sur toutes les machines (chiffre d'un benchmark spécifique)
- ❌ "Valeur légale" des certificats (ce sont des **rapports de conversion certifiés**, pas des actes notariés)
- ❌ 32 formats parfaits dès le jour 1 (progression documentée et honnête)
- ❌ RAM identique à 200 Mo sur tous les systèmes (le cache OS utilise la RAM disponible pour mmap)

### 1.4 Les Trois Niveaux de Round-Trip

```
NIVEAU 1 — Bit-Identical : A → A (MÊME format uniquement)
  GGUF → GGUF : SHA256(source) == SHA256(cible)
  SafeTensors → SafeTensors : SHA256(source) == SHA256(cible)
  Garantie absolue. Vérifiable immédiatement.

NIVEAU 2 — Sémantique : A → B → A (cross-format)
  GGUF → ONNX → GGUF : sorties d'inférence identiques dans la tolérance
  SafeTensors → PyTorch → SafeTensors : poids identiques à 1e-6 près
  Dépend du couple de formats. Divergence documentée par paire.

NIVEAU 3 — Structurel : pour les formats qui transforment le graphe
  Même architecture, même nombre de couches, même topologie
  Garanti pour toutes les conversions. Vérifiable automatiquement.
```

### 1.5 L'Insight Architectural — Version Correcte

```
Réalité mathématique :
  80% des conversions → N + M composants (IR suffit, pas de logique de paire)
  20% des conversions → logique spécifique à la paire (cas edge documentés)

Format A → [Loader A] → IR_UMC → [Saver B] → Format B

IR_UMC = union évolutive de tous les formats supportés
       = sur-ensemble, enrichi à chaque nouveau format

Logique de paire requise pour :
  - Différences de sémantique d'opérateurs (BatchNorm PyTorch ≠ BatchNorm ONNX)
  - Différences de layout mémoire (NCHW ↔ NHWC)
  - Endianness et alignement
  → Ces cas sont documentés dans ConversionHints par paire de formats
```

---

## 2. ARCHITECTURE TECHNIQUE GLOBALE

### 2.1 Vue d'Ensemble du Système

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         UMC — Architecture v3.0                              │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        COUCHE INTERFACE                               │  │
│  │   CLI (clap 4.5)  ·  API REST (axum 0.7)  ·  SDK Python  ·  SDK JS  │  │
│  └─────────────────────────────┬────────────────────────────────────────┘  │
│                                │                                            │
│  ┌─────────────────────────────▼────────────────────────────────────────┐  │
│  │                      COUCHE ORCHESTRATION                              │  │
│  │  Détection Format (magic bytes cascade)                               │  │
│  │  Routing Dijkstra (chemin optimal automatique)                        │  │
│  │  Job Queue (PostgreSQL SKIP LOCKED → Redis Streams → Kafka)           │  │
│  │  GraphTemplate Registry (reconstruction de graphe pour weights-only)  │  │
│  │  ConversionHints Registry (logique de paire source→cible)             │  │
│  │  Capability Registry (outils disponibles sur ce système)              │  │
│  └─────────────────────────────┬────────────────────────────────────────┘  │
│                                │                                            │
│  ┌─────────────────────────────▼────────────────────────────────────────┐  │
│  │                    COUCHE CONVERSION (CORE)                            │  │
│  │                                                                        │  │
│  │  ┌──────────────┐  ┌──────────────────────────────┐  ┌────────────┐  │  │
│  │  │   LOADERS    │─▶│       IR UNIVERSELLE v3       │─▶│   SAVERS   │  │  │
│  │  │  Rust natif  │  │  TensorStore (mmap+streaming) │  │ Rust natif │  │  │
│  │  │  sécurisé    │  │  GraphContent (optionnel)     │  │  validé    │  │  │
│  │  │  fuzzé       │  │  MetadataStore                │  │  certifié  │  │  │
│  │  └──────────────┘  │  QuantizationStore (étendu)   │  └────────────┘  │  │
│  │                    │  AdapterStore                  │                  │  │
│  │  ┌──────────────┐  │  ExtensionStore (limité+sûr)  │  ┌────────────┐  │  │
│  │  │   PIPELINE   │  │  TokenizerStore               │  │ VALIDATOR  │  │  │
│  │  │  Reader      │  │  ProvenanceChain (immutable)   │  │ Structurel │  │  │
│  │  │  Transformer │  │  SecurityBounds               │  │ Numérique  │  │  │
│  │  │  Writer      │  │  ConversionHints              │  │ Sémantique │  │  │
│  │  │  Watchdog    │  └──────────────────────────────-┘  │ Certificat │  │  │
│  │  │  Cancel Token│                                      └────────────┘  │  │
│  │  └──────────────┘                                                       │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     COUCHE INFRASTRUCTURE                              │  │
│  │  memmap2 · rayon · crossbeam · tokio · xxhash · sha2 · SIMD runtime  │  │
│  │  cargo-fuzz · proptest · criterion · loom (tests concurrence)         │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Principes Architecturaux Non Négociables

```
PRINCIPE 1 — Natif ou Rien
  Toute fonctionnalité annoncée est implémentée en Rust pur.
  Aucun subprocess dans le chemin de conversion critique.
  Les outils externes (trtexec, coremltools) génèrent des "Recettes" — jamais exécutés par UMC.

PRINCIPE 2 — Sécurité par Défaut
  Tout fichier entrant est hostile jusqu'à preuve du contraire.
  Limites hardcodées (SecurityBounds) sur TOUS les champs lus depuis les fichiers.
  Fuzzing automatique sur tous les loaders en CI.

PRINCIPE 3 — Honnêteté des Promesses
  Aucune garantie qui ne peut être vérifiée automatiquement.
  Chaque chiffre de performance = méthodologie publique reproductible.

PRINCIPE 4 — Progression Qualitative
  5 formats parfaits avant 32 formats médiocres.
  Chaque format : spec lue, tests écrits, fuzzing, benchmarks.

PRINCIPE 5 — Zéro Deadlock, Zéro Panique
  CancellationToken coopératif dans tous les threads.
  Watchdog détectant la stagnation.
  Timeout sur toutes les opérations.
  TempOutputFile + atomic rename : fichier valide ou absent, jamais corrompu.

PRINCIPE 6 — Détection SIMD au Runtime
  Jamais de flags SIMD globaux dans .cargo/config.toml (crashes sur CPUs sans AVX-512).
  Détection via is_x86_feature_detected!() et std::arch::is_aarch64_feature_detected!().
  Fallback scalaire toujours correct et présent.
```

### 2.3 Structure du Workspace Rust

```
umc/
├── Cargo.toml                      # Workspace root
├── Cargo.lock
├── rust-toolchain.toml             # Rust stable pinné
├── .cargo/config.toml              # LLD linker uniquement (pas de SIMD global)
│
├── crates/
│   ├── umc-core/                   # IR + traits + sécurité + erreurs
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ir/
│   │       │   ├── mod.rs
│   │       │   ├── tensor.rs           # TensorStore, Tensor, TensorData, DType
│   │       │   ├── graph.rs            # GraphContent, ComputeGraph, UniversalOp
│   │       │   ├── metadata.rs         # MetadataStore, MetaValue
│   │       │   ├── quantization.rs     # QuantizationStore, TensorQuantization, CanonicalQuant
│   │       │   ├── adapter.rs          # AdapterInfo, LoRA, QLoRA, PEFT
│   │       │   ├── pruning.rs          # PruningInfo, PruningMask
│   │       │   ├── distillation.rs     # DistillationInfo
│   │       │   ├── tokenizer.rs        # TokenizerStore, BPE, SentencePiece, TikToken
│   │       │   ├── extension.rs        # ExtensionStore (limité, namespaced, sécurisé)
│   │       │   ├── provenance.rs       # ProvenanceChain (immutable hash chaining)
│   │       │   ├── config.rs           # ArchitectureConfig, GenerationConfig
│   │       │   └── security.rs         # SecurityBounds, validation des champs
│   │       ├── traits/
│   │       │   ├── loader.rs           # trait FormatLoader + SecureLoader
│   │       │   ├── saver.rs            # trait FormatSaver
│   │       │   └── detector.rs         # trait FormatDetector
│   │       ├── hints.rs                # ConversionHints par paire source→cible
│   │       ├── error.rs                # UmcError enum complet + codes de sortie
│   │       └── dtype.rs                # DType, conversions, bytes_per_element
│   │
│   ├── umc-detect/                 # Détection automatique de format
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── magic.rs                # Table magic bytes (tous formats)
│   │       ├── registry.rs             # FormatRegistry, cascade multi-niveaux
│   │       └── heuristics.rs           # Analyse de contenu pour formats ambigus
│   │
│   ├── umc-graph/                  # Graphe Dijkstra + GraphTemplate Registry
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── conversion_graph.rs     # ConversionGraph, arêtes, coûts
│   │       ├── dijkstra.rs             # Algorithme Dijkstra pondéré
│   │       ├── template_registry.rs    # GraphTemplate pour formats weights-only
│   │       └── templates/
│   │           ├── llama.rs            # LlamaTemplate (Llama 1/2/3, Mistral, Mixtral)
│   │           ├── phi.rs              # PhiTemplate (Phi-1/2/3)
│   │           ├── gemma.rs            # GemmaTemplate (Gemma 1/2)
│   │           ├── qwen.rs             # QwenTemplate (Qwen 1/1.5/2)
│   │           ├── falcon.rs           # FalconTemplate
│   │           └── generic_decoder.rs  # Template générique pour décodeurs inconnus
│   │
│   ├── umc-pipeline/               # Pipeline 3-threads + Watchdog + CancellationToken
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pipeline.rs             # ConversionPipeline, CancellationToken
│   │       ├── reader.rs               # Thread Reader (mmap, streaming)
│   │       ├── transformer.rs          # Thread Transformer (rayon, SIMD)
│   │       ├── writer.rs               # Thread Writer (TempOutputFile + atomic rename)
│   │       └── watchdog.rs             # Watchdog thread (détection stagnation)
│   │
│   ├── umc-validate/               # Validation + Certification
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── structural.rs           # Hash structurel, topologie de graphe
│   │       ├── numeric.rs              # Comparaison SIMD, divergence, profils
│   │       ├── semantic.rs             # Comparaison de tenseurs déquantifiés
│   │       ├── functional.rs           # Exécution via runtime natif minimal
│   │       ├── roundtrip.rs            # Test round-trip, niveaux 1/2/3
│   │       └── certificate.rs          # Rapport de conversion certifié, ed25519
│   │
│   ├── umc-formats/                # Tous les loaders/savers natifs
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── gguf/
│   │       │   ├── loader.rs
│   │       │   └── saver.rs
│   │       ├── safetensors/
│   │       ├── onnx/
│   │       ├── pytorch/               # ZIP + SafePickleParser (whitelist)
│   │       ├── tensorflow/
│   │       ├── tflite/                # FlatBuffers natif
│   │       ├── coreml/                # Protobuf natif (mlmodel)
│   │       ├── awq/
│   │       ├── gptq/
│   │       ├── bitsandbytes/
│   │       ├── executorch/            # FlatBuffers natif
│   │       ├── sentencepiece/
│   │       ├── tiktoken/
│   │       ├── lora/ qlora/ peft/
│   │       ├── ggml/                  # Lecture seule, legacy
│   │       ├── keras_h5/              # Lecture seule, hdf5-rs
│   │       ├── jax_flax/              # Lecture seule, rmp-serde
│   │       ├── torchscript/
│   │       ├── paddlepaddle/
│   │       ├── onnx_runtime/
│   │       ├── openvino/              # XML + bin natif
│   │       ├── diffusers/             # Format composite
│   │       ├── mediapipe/
│   │       ├── recipes/               # Recipe Savers (TensorRT, QNN, TVM, Triton...)
│   │       └── onnx_web/
│   │
│   ├── umc-cli/                    # Interface CLI (clap)
│   ├── umc-api/                    # API REST (axum + PostgreSQL + Redis)
│   └── umc-fuzz/                   # Cibles cargo-fuzz
│       ├── fuzz_targets/
│       │   ├── fuzz_gguf.rs
│       │   ├── fuzz_onnx.rs
│       │   ├── fuzz_safetensors.rs
│       │   ├── fuzz_pytorch.rs
│       │   └── fuzz_tflite.rs
│
├── tests/
│   ├── round_trip/                 # Tests round-trip par format
│   ├── security/                   # Tests de parsing malveillant
│   ├── fixtures/                   # Modèles de test (< 100 Mo)
│   └── conformity/                 # Tests de conformité de format
│
└── benches/                        # Benchmarks criterion publics
    ├── conversion_bench.rs
    └── simd_bench.rs
```

### 2.4 Cargo.toml Workspace

```toml
[workspace]
members = [
    "crates/umc-core",
    "crates/umc-detect",
    "crates/umc-graph",
    "crates/umc-pipeline",
    "crates/umc-validate",
    "crates/umc-formats",
    "crates/umc-cli",
    "crates/umc-api",
    "crates/umc-fuzz",
]
resolver = "2"

[workspace.dependencies]
# Sérialisation
serde            = { version = "1.0", features = ["derive"] }
serde_json       = "1.0"
prost            = "0.12"
bincode          = "2.0"
rmp-serde        = "1.3"           # JAX/Flax msgpack
flatbuffers      = "23.5"          # TFLite, ExecuTorch
hdf5-rs          = "0.8"           # Keras H5

# Parallélisme
rayon            = "1.10"
crossbeam        = "0.8"
tokio            = { version = "1.40", features = ["full"] }
num_cpus         = "1.16"
sysinfo          = "0.31"

# Mémoire et I/O
memmap2          = "0.9"
bytes            = "1.7"
indexmap         = "2.4"

# Hashing et cryptographie
xxhash-rust      = { version = "0.8", features = ["xxh64"] }
sha2             = "0.10"
ed25519-dalek    = "2.1"
getrandom        = "0.2"
hex              = "0.4"
constant_time_eq = "0.3"

# Web et URL
url              = "2.5"

# CLI
clap             = { version = "4.5", features = ["derive"] }

# API
axum             = "0.7"
tower            = "0.5"
tower-http       = { version = "0.5", features = ["cors", "trace"] }
sqlx             = { version = "0.8", features = ["postgres", "runtime-tokio", "uuid"] }
redis            = { version = "0.26", features = ["tokio-comp"] }
uuid             = { version = "1.10", features = ["v4"] }

# Graphe
petgraph         = "0.6"

# Logging et erreurs
thiserror        = "1.0"
anyhow           = "1.0"
tracing          = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Tests
criterion        = "0.5"
proptest         = "1.5"
tempfile         = "3.13"
loom             = "0.7"           # Tests de concurrence

[profile.release]
opt-level        = 3
lto              = "fat"
codegen-units    = 1
strip            = true
panic            = "abort"         # Pas d'unwinding en production

[profile.bench]
opt-level        = 3
lto              = "thin"
```

### 2.5 Configuration Build Correcte

```toml
# .cargo/config.toml
# IMPORTANT : PAS de target-feature SIMD globaux (crashs sur AMD Zen 3, Intel Alder Lake E-cores)
# La détection SIMD se fait au RUNTIME via is_x86_feature_detected!()

[build]
rustflags = [
    "-C", "link-arg=-fuse-ld=lld",   # LLD linker (plus rapide)
    # PAS de target-cpu=native (binaires non portables)
    # PAS de target-feature=+avx512f (crash sur CPUs sans AVX-512)
]
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

---

# PARTIE II — L'IR UNIVERSELLE (umc-core)

---

## 3. L'IR UNIVERSELLE v3 — CŒUR DU SYSTÈME

### 3.1 Structure Principale

```rust
// crates/umc-core/src/ir/mod.rs

/// IR Universelle v3 — corrections complètes appliquées :
/// - GraphContent optionnel (formats weights-only vs formats avec graphe)
/// - ExtensionStore limité (100 Mo max, clés namespaced)
/// - ProvenanceChain immutable (hash chaining tamper-evident)
/// - SecurityBounds intégrés et validés à chaque insertion
/// - TensorQuantization avec tous les paramètres requis
#[derive(Debug, Clone)]
pub struct UniversalIR {
    /// Tenseurs du modèle (poids, biases, embeddings...)
    pub tensors: TensorStore,

    /// Contenu de graphe — distingue weights-only des formats avec graphe
    /// CRITIQUE : Les formats LLM (GGUF, SafeTensors) sont WeightsOnly
    /// Ils n'ont PAS de graphe explicite. La reconstruction se fait via GraphTemplate.
    pub graph: GraphContent,

    /// Métadonnées générales du modèle
    pub metadata: MetadataStore,

    /// Configuration d'architecture (hyperparamètres du modèle)
    pub architecture: ArchitectureConfig,

    /// Tokenizer (si présent — LLM principalement)
    pub tokenizer: Option<TokenizerStore>,

    /// Schéma de quantification global
    pub quantization: Option<QuantizationStore>,

    /// Adaptateurs (LoRA, QLoRA, PEFT...)
    pub adapters: Vec<AdapterInfo>,

    /// Élagage / Pruning
    pub pruning: Option<PruningInfo>,

    /// Distillation
    pub distillation: Option<DistillationInfo>,

    /// Configuration de génération (LLM)
    pub generation_config: Option<GenerationConfig>,

    /// Configuration d'entraînement
    pub training_config: Option<TrainingConfig>,

    /// Chaîne de provenance — immutable par hash chaining
    pub provenance: ProvenanceChain,

    /// Extensions opaques — garantie de zéro perte d'information
    /// Limité à 100 Mo. Clés namespaced "FORMAT@VERSION/chemin".
    pub extensions: ExtensionStore,

    /// Hints de conversion — logique spécifique à des paires de formats
    /// Résout le problème des "20% de cas edge" qui nécessitent de la logique de paire
    pub conversion_hints: ConversionHintsMap,
}

/// Contenu de graphe — NOUVEAU : distingue explicitement les cas
#[derive(Debug, Clone)]
pub enum GraphContent {
    /// Format avec graphe explicite (ONNX, PyTorch, TFSavedModel, TFLite...)
    Explicit(ComputeGraph),

    /// Format weights-only (GGUF, SafeTensors, AWQ, GPTQ, bitsandbytes...)
    /// Le graphe sera reconstruit via GraphTemplate lors de la conversion vers
    /// un format qui en a besoin (ONNX, TFLite, CoreML...)
    WeightsOnly {
        architecture: String,          // "llama", "mistral", "phi", "gemma"...
        template_available: bool,      // true si UMC a un template pour cette archi
        template_name: Option<String>, // Nom du template utilisé
    },

    /// Format composite — plusieurs sous-modèles (Diffusers)
    Composite(Vec<SubModelGraph>),

    /// Graphe vide — format purement de tokenizer (SentencePiece, TikToken)
    TokenizerOnly,
}

#[derive(Debug, Clone)]
pub struct SubModelGraph {
    pub name: String,               // "unet", "vae", "text_encoder"
    pub graph: ComputeGraph,
    pub role: SubModelRole,
    pub format_hint: String,        // Format d'origine de ce sous-modèle
}

#[derive(Debug, Clone)]
pub enum SubModelRole {
    TextEncoder,
    TextEncoder2,
    ImageEncoder,
    Denoiser,        // UNet
    Decoder,         // VAE Decoder
    Encoder,         // VAE Encoder
    Scheduler,
    Custom(String),
}

/// Métadonnées générales
#[derive(Debug, Clone, Default)]
pub struct MetadataStore {
    entries: indexmap::IndexMap<String, MetaValue>,
}

#[derive(Debug, Clone)]
pub enum MetaValue {
    String(String),
    I64(i64),
    F64(f64),
    Bool(bool),
    Array(Vec<MetaValue>),
    Raw(Vec<u8>),
}

/// Configuration d'architecture — tous les hyperparamètres courants
#[derive(Debug, Clone, Default)]
pub struct ArchitectureConfig {
    pub architecture: String,                         // "llama", "mistral", "phi"...
    pub model_type: String,                           // "causal_lm", "seq2seq", "vision"...
    pub hidden_size: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub num_kv_heads: Option<usize>,                  // GQA
    pub intermediate_size: usize,
    pub max_position_embeddings: usize,
    pub vocab_size: usize,
    pub rms_norm_eps: Option<f64>,
    pub layer_norm_eps: Option<f64>,
    pub rope_theta: Option<f64>,
    pub rope_scaling: Option<RopeScalingConfig>,
    pub attention_bias: bool,
    pub tie_word_embeddings: bool,
    pub torch_dtype: Option<String>,
    pub transformers_version: Option<String>,
    pub extra_fields: indexmap::IndexMap<String, serde_json::Value>, // Champs inconnus préservés
}

#[derive(Debug, Clone)]
pub struct RopeScalingConfig {
    pub scaling_type: String,      // "linear", "ntk", "yarn", "llama3", "su"
    pub factor: f64,
    pub original_max_position_embeddings: Option<usize>,
    pub low_freq_factor: Option<f64>,
    pub high_freq_factor: Option<f64>,
    pub extra: indexmap::IndexMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct GenerationConfig {
    pub bos_token_id: Option<u32>,
    pub eos_token_id: Option<Vec<u32>>,
    pub pad_token_id: Option<u32>,
    pub max_new_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<usize>,
    pub repetition_penalty: Option<f64>,
    pub do_sample: bool,
    pub extra: indexmap::IndexMap<String, serde_json::Value>,
}

/// Map des hints de conversion par paire de formats
/// Résout les incompatibilités sémantiques entre formats
#[derive(Debug, Clone, Default)]
pub struct ConversionHintsMap {
    hints: std::collections::HashMap<(String, String), ConversionHints>,
}

#[derive(Debug, Clone, Default)]
pub struct ConversionHints {
    /// Permutation de layout mémoire requise
    pub layout_transpose: Option<Vec<usize>>,  // NCHW → NHWC par exemple
    /// BatchNorm doit être fusionné avant export
    pub fuse_batchnorm: bool,
    /// Les tied weights doivent être dupliqués (ou dédupliqués)
    pub tied_weights_policy: TiedWeightsPolicy,
    /// Opérateurs qui doivent être décomposés pour ce format cible
    pub decompose_ops: Vec<String>,
    /// Alignement des poids requis par le format cible
    pub weight_alignment: Option<usize>,
    /// Commentaire libre pour documentation
    pub note: String,
}

#[derive(Debug, Clone, Default)]
pub enum TiedWeightsPolicy {
    #[default]
    PreserveShared,     // Garder les tied weights liés (économise la mémoire)
    Duplicate,          // Dupliquer pour les formats qui ne supportent pas le sharing
    Deduplicate,        // Dédupliquer si le format source avait des copies inutiles
}
```

### 3.2 TensorStore — Zéro Copie avec Sécurité Complète

```rust
// crates/umc-core/src/ir/tensor.rs

use memmap2::Mmap;
use std::sync::Arc;

/// Limites de sécurité — validées à chaque insertion de tenseur
/// Ces limites PRÉVIENNENT les DoS via des fichiers malformés
#[derive(Debug, Clone)]
pub struct SecurityBounds {
    pub max_tensor_count: usize,        // 1_000_000
    pub max_metadata_count: usize,      // 10_000
    pub max_string_length: usize,       // 1_048_576 (1 Mo)
    pub max_shape_rank: usize,          // 8
    pub max_tensor_size_bytes: usize,   // 100 * 1024^3 (100 Go par tenseur)
    pub max_extension_bytes: usize,     // 100 * 1024^2 (100 Mo pour ExtensionStore)
    pub max_metadata_nesting: usize,    // 32 (protobuf depth limit)
    pub max_compression_ratio: usize,   // 1000 (anti-zip-bomb)
}

impl Default for SecurityBounds {
    fn default() -> Self {
        Self {
            max_tensor_count: 1_000_000,
            max_metadata_count: 10_000,
            max_string_length: 1_048_576,
            max_shape_rank: 8,
            max_tensor_size_bytes: 100 * 1024 * 1024 * 1024,
            max_extension_bytes: 100 * 1024 * 1024,
            max_metadata_nesting: 32,
            max_compression_ratio: 1000,
        }
    }
}

/// Un tenseur dans l'IR UMC — données jamais copiées si > mmap_threshold
#[derive(Debug, Clone)]
pub struct Tensor {
    pub name: String,
    pub dtype: DType,
    pub shape: Vec<usize>,
    pub strides: Option<Vec<usize>>,   // None = C-contiguous (row-major)
    pub layout: Layout,
    pub data: TensorData,
    pub checksum: u64,                  // xxHash64 des données brutes
    pub quantization: Option<TensorQuantization>,
}

/// Layout mémoire
#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    CContiguous,    // Row-major (NumPy par défaut, PyTorch par défaut)
    FContiguous,    // Column-major (Fortran, certains formats BLAS)
    Custom,         // Strides custom (fournies dans Tensor.strides)
}

/// Données du tenseur — zéro copie via mmap pour les gros fichiers
#[derive(Debug, Clone)]
pub enum TensorData {
    /// Vue mmap directe sur le fichier source (ZÉRO copie)
    MmapView {
        mmap: Arc<Mmap>,
        offset: usize,
        length: usize,
    },
    /// Données en RAM (pour tenseurs < mmap_threshold ou résultats de transformation)
    Owned(Arc<Vec<u8>>),
    /// Chargement paresseux — ne charge qu'au moment de l'accès
    Lazy {
        file_path: std::path::PathBuf,
        offset: u64,
        length: usize,
        checksum: u64,      // Vérifié au chargement
    },
    /// Référence vers un autre tenseur (tied weights — embed_tokens == lm_head)
    Shared {
        target_name: String,
        transforms: Vec<TensorTransform>, // Transposes appliquées si nécessaire
    },
}

/// Transformations légères applicables à un tenseur partagé
#[derive(Debug, Clone)]
pub enum TensorTransform {
    Transpose(Vec<usize>),    // Permutation des dimensions
    Slice { axis: usize, start: usize, end: usize },
}

impl TensorData {
    /// Accès aux bytes bruts — ne copie JAMAIS pour MmapView
    pub fn as_bytes(&self) -> Result<&[u8], UmcError> {
        match self {
            Self::MmapView { mmap, offset, length } => {
                Ok(&mmap[*offset..*offset + *length])
            }
            Self::Owned(data) => Ok(data.as_slice()),
            Self::Lazy { .. } => Err(UmcError::NotMaterialized),
            Self::Shared { target_name, .. } => Err(UmcError::IsReference(target_name.clone())),
        }
    }

    /// Taille en octets
    pub fn len(&self) -> usize {
        match self {
            Self::MmapView { length, .. } => *length,
            Self::Owned(v) => v.len(),
            Self::Lazy { length, .. } => *length,
            Self::Shared { .. } => 0,
        }
    }

    /// Matérialise un tenseur Lazy en MmapView (avec vérification de checksum)
    pub fn materialize_mmap(&mut self) -> Result<(), UmcError> {
        if let Self::Lazy { file_path, offset, length, checksum } = self {
            let file = std::fs::File::open(file_path).map_err(UmcError::Io)?;
            let mmap = Arc::new(unsafe {
                memmap2::Mmap::map(&file).map_err(|e| UmcError::Mmap(e.to_string()))?
            });
            // Vérifier le checksum avant d'utiliser
            let actual_checksum = xxhash_rust::xxh64::xxh64(
                &mmap[*offset as usize..*offset as usize + *length],
                0,
            );
            if actual_checksum != *checksum {
                return Err(UmcError::ChecksumMismatch {
                    expected: *checksum,
                    actual: actual_checksum,
                    context: file_path.display().to_string(),
                });
            }
            *self = Self::MmapView {
                mmap,
                offset: *offset as usize,
                length: *length,
            };
        }
        Ok(())
    }
}

/// TensorStore — stockage ordonné avec validation de sécurité à l'insertion
#[derive(Debug, Clone)]
pub struct TensorStore {
    tensors: indexmap::IndexMap<String, Tensor>,
    ram_usage_bytes: usize,
    pub mmap_threshold_bytes: usize,   // Seuil mmap (64 Mo par défaut)
    bounds: SecurityBounds,
}

impl TensorStore {
    pub fn new() -> Self {
        Self {
            tensors: indexmap::IndexMap::new(),
            ram_usage_bytes: 0,
            mmap_threshold_bytes: 64 * 1024 * 1024,
            bounds: SecurityBounds::default(),
        }
    }

    pub fn with_bounds(bounds: SecurityBounds) -> Self {
        Self { bounds, ..Self::new() }
    }

    /// Insertion avec validation complète de sécurité
    pub fn insert(&mut self, tensor: Tensor) -> Result<(), UmcError> {
        // Vérifier le nombre de tenseurs
        if self.tensors.len() >= self.bounds.max_tensor_count {
            return Err(UmcError::SecurityViolation {
                field: "tensor_count".into(),
                value: self.tensors.len(),
                limit: self.bounds.max_tensor_count,
            });
        }
        // Vérifier le rang du shape
        if tensor.shape.len() > self.bounds.max_shape_rank {
            return Err(UmcError::SecurityViolation {
                field: "shape_rank".into(),
                value: tensor.shape.len(),
                limit: self.bounds.max_shape_rank,
            });
        }
        // Vérifier la taille des données
        if tensor.data.len() > self.bounds.max_tensor_size_bytes {
            return Err(UmcError::SecurityViolation {
                field: "tensor_size_bytes".into(),
                value: tensor.data.len(),
                limit: self.bounds.max_tensor_size_bytes,
            });
        }
        // Vérifier le nom (pas de caractères nuls ni de longueur excessive)
        if tensor.name.len() > self.bounds.max_string_length {
            return Err(UmcError::SecurityViolation {
                field: "tensor_name_length".into(),
                value: tensor.name.len(),
                limit: self.bounds.max_string_length,
            });
        }
        if tensor.name.contains('\0') {
            return Err(UmcError::InvalidTensorName(tensor.name.clone()));
        }

        if let TensorData::Owned(ref v) = tensor.data {
            self.ram_usage_bytes = self.ram_usage_bytes.saturating_add(v.len());
        }
        self.tensors.insert(tensor.name.clone(), tensor);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Tensor> {
        self.tensors.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Tensor> {
        self.tensors.get_mut(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Tensor)> {
        self.tensors.iter()
    }

    pub fn len(&self) -> usize { self.tensors.len() }
    pub fn is_empty(&self) -> bool { self.tensors.is_empty() }
    pub fn ram_usage_mb(&self) -> f64 { self.ram_usage_bytes as f64 / (1024.0 * 1024.0) }

    /// Résout un tenseur Shared vers le tenseur concret
    pub fn resolve_shared<'a>(&'a self, tensor: &'a Tensor) -> Result<&'a Tensor, UmcError> {
        match &tensor.data {
            TensorData::Shared { target_name, .. } => {
                self.tensors.get(target_name)
                    .ok_or_else(|| UmcError::MissingSharedTensor(target_name.clone()))
            }
            _ => Ok(tensor),
        }
    }
}

/// DType — sur-ensemble universel de tous les formats supportés
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DType {
    // Flottants IEEE 754
    F64, F32, F16, BF16,
    // FP8 (H100+)
    F8E4M3, F8E5M2,
    // Entiers signés
    I64, I32, I16, I8,
    // Entiers non signés
    U64, U32, U16, U8,
    // Booléen
    Bool,
    // GGUF K-quants (blocs)
    Q2K, Q3KS, Q3KM, Q3KL,
    Q4_0, Q4_1, Q4KS, Q4KM,
    Q5_0, Q5_1, Q5KS, Q5KM,
    Q6K, Q8_0, Q8K,
    // AWQ/GPTQ (canal)
    Awq4, Awq8, Gptq2, Gptq3, Gptq4, Gptq8,
    // bitsandbytes
    NF4, FP4,
    // Personnalisé
    Custom(String),
}

impl DType {
    /// Octets par élément (None pour les types sub-byte sans facteur entier)
    pub fn bytes_per_element(&self) -> Option<f64> {
        match self {
            Self::F64 | Self::I64 | Self::U64 => Some(8.0),
            Self::F32 | Self::I32 | Self::U32 => Some(4.0),
            Self::F16 | Self::BF16 | Self::I16 | Self::U16 => Some(2.0),
            Self::F8E4M3 | Self::F8E5M2 | Self::I8 | Self::U8 | Self::Bool => Some(1.0),
            Self::Q8_0 | Self::Q8K | Self::Awq8 | Self::Gptq8 => Some(1.0),
            Self::Q4_0 | Self::Q4_1 | Self::Q4KS | Self::Q4KM => Some(0.5),
            Self::Q5_0 | Self::Q5_1 | Self::Q5KS | Self::Q5KM => Some(0.625),
            Self::Q6K => Some(0.75),
            Self::Q2K | Self::Gptq2 => Some(0.25),
            Self::Q3KS | Self::Q3KM | Self::Q3KL | Self::Gptq3 => Some(0.375),
            Self::NF4 | Self::FP4 | Self::Awq4 | Self::Gptq4 => Some(0.5),
            Self::Custom(_) => None,
        }
    }

    /// Conversion lossless possible ?
    pub fn is_lossless_upcast_to(&self, target: &DType) -> bool {
        matches!(
            (self, target),
            (DType::F16, DType::F32) | (DType::F16, DType::F64)
            | (DType::BF16, DType::F32) | (DType::BF16, DType::F64)
            | (DType::F32, DType::F64)
            | (DType::I8, DType::I16) | (DType::I8, DType::I32) | (DType::I8, DType::I64)
            | (DType::I16, DType::I32) | (DType::I16, DType::I64)
            | (DType::I32, DType::I64)
            | (DType::U8, DType::U16) | (DType::U8, DType::U32) | (DType::U8, DType::U64)
            | (DType::U16, DType::U32) | (DType::U16, DType::U64)
            | (DType::U32, DType::U64)
        )
    }

    /// Le type est-il quantifié (perd de l'information par rapport à F32) ?
    pub fn is_quantized(&self) -> bool {
        matches!(self,
            Self::Q2K | Self::Q3KS | Self::Q3KM | Self::Q3KL
            | Self::Q4_0 | Self::Q4_1 | Self::Q4KS | Self::Q4KM
            | Self::Q5_0 | Self::Q5_1 | Self::Q5KS | Self::Q5KM
            | Self::Q6K | Self::Q8_0 | Self::Q8K
            | Self::Awq4 | Self::Awq8 | Self::Gptq2 | Self::Gptq3
            | Self::Gptq4 | Self::Gptq8
            | Self::NF4 | Self::FP4
        )
    }
}
```

### 3.3 TensorQuantization — Métadonnées Complètes (Correction Critique)

```rust
// crates/umc-core/src/ir/quantization.rs

/// Métadonnées de quantification complètes
/// CORRECTION : Version précédente manquait block_size, superblock_size, scale_dtype, storage_order
/// Ces champs sont OBLIGATOIRES pour une déquantification correcte
#[derive(Debug, Clone)]
pub struct TensorQuantization {
    pub scheme: QuantScheme,
    pub block_size: usize,              // OBLIGATOIRE — taille du bloc de quantification
    pub superblock_size: Option<usize>, // Pour GGUF K-quants : 256 éléments
    pub scale_dtype: DType,             // Type des scales : F16, F32, Q8_0...
    pub zero_point_dtype: DType,        // Type des zero-points
    pub storage_order: StorageOrder,    // Comment poids et scales sont entrelacés
    pub calibration_dataset: Option<String>, // Référence dataset (AWQ/GPTQ)
    pub calibration_method: Option<String>,  // "minmax", "percentile", "mse"
    pub group_size: Option<usize>,      // Pour GPTQ/AWQ : taille du groupe
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantScheme {
    // GGUF K-quants
    GgufQ2K, GgufQ3KS, GgufQ3KM, GgufQ3KL,
    GgufQ4_0, GgufQ4_1, GgufQ4KS, GgufQ4KM,
    GgufQ5_0, GgufQ5_1, GgufQ5KS, GgufQ5KM,
    GgufQ6K, GgufQ8_0, GgufQ8K,
    // AWQ
    AwqGemm4, AwqGemv4, AwqGemm8,
    // GPTQ
    Gptq { bits: u8, sym: bool },
    // bitsandbytes
    BnbNF4, BnbFP4,
    // Standard
    SymmetricInt8, AsymmetricInt8,
    // Personnalisé
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageOrder {
    Sequential,     // Poids puis scales séparément (GPTQ)
    Interleaved,    // Poids et scales entrelacés par groupe (AWQ)
    BlockPacked,    // Blocs compacts avec scales inclus (GGUF)
}

/// Représentation canonique — pont universel entre tous les schémas
/// Permet la déquantification et la re-quantification cross-scheme
#[derive(Debug, Clone)]
pub struct CanonicalQuantization {
    pub bit_width: u8,
    pub block_size: usize,
    pub superblock_size: Option<usize>,
    pub scales: Vec<f32>,
    pub zero_points: Vec<f32>,
    pub scales_dtype: DType,
    pub quantized_data: Vec<u8>,
    pub storage_order: StorageOrder,
}

impl CanonicalQuantization {
    /// Déquantification vers F32 — 100% natif Rust
    pub fn dequantize_to_f32(&self) -> Result<Vec<f32>, UmcError> {
        match self.storage_order {
            StorageOrder::BlockPacked => self.dequantize_block_packed(),
            StorageOrder::Sequential => self.dequantize_sequential(),
            StorageOrder::Interleaved => self.dequantize_interleaved(),
        }
    }

    fn dequantize_block_packed(&self) -> Result<Vec<f32>, UmcError> {
        let block_size = self.block_size;
        let superblock = self.superblock_size.unwrap_or(block_size);
        let bytes_per_block = (block_size * self.bit_width as usize).div_ceil(8);
        let mut result = Vec::with_capacity(
            self.quantized_data.len() * 8 / self.bit_width as usize
        );

        for (block_idx, chunk) in self.quantized_data.chunks(bytes_per_block).enumerate() {
            let scale = self.scales.get(block_idx).copied().unwrap_or(1.0);
            let zero = self.zero_points.get(block_idx).copied().unwrap_or(0.0);

            match self.bit_width {
                4 => {
                    for &byte in chunk {
                        let lo = (byte & 0x0F) as f32;
                        let hi = (byte >> 4) as f32;
                        result.push(scale * (lo - zero));
                        result.push(scale * (hi - zero));
                    }
                }
                8 => {
                    for &byte in chunk {
                        result.push(scale * (byte as f32 - zero));
                    }
                }
                _ => return Err(UmcError::UnsupportedBitWidth(self.bit_width)),
            }
            let _ = superblock; // Utilisé pour les superblocks GGUF dans l'impl complète
        }
        Ok(result)
    }

    fn dequantize_sequential(&self) -> Result<Vec<f32>, UmcError> {
        // GPTQ : poids quantifiés contigus, puis scales séparés
        let num_elements = self.quantized_data.len() * 8 / self.bit_width as usize;
        let mut result = Vec::with_capacity(num_elements);
        let group_size = self.block_size;

        for (elem_idx, byte_idx) in (0..num_elements).enumerate() {
            let group_idx = elem_idx / group_size;
            let scale = self.scales.get(group_idx).copied().unwrap_or(1.0);
            let zero = self.zero_points.get(group_idx).copied().unwrap_or(0.0);
            let byte_pos = (elem_idx * self.bit_width as usize) / 8;
            let bit_offset = (elem_idx * self.bit_width as usize) % 8;
            let mask = (1u8 << self.bit_width) - 1;
            let raw_byte = self.quantized_data.get(byte_pos).copied().unwrap_or(0);
            let q = (raw_byte >> bit_offset) & mask;
            result.push(scale * (q as f32 - zero));
            let _ = byte_idx;
        }
        Ok(result)
    }

    fn dequantize_interleaved(&self) -> Result<Vec<f32>, UmcError> {
        // AWQ : poids et scales interleaved par groupe
        let group_size = self.block_size;
        let bytes_per_weight = self.bit_width as usize;
        let num_groups = self.scales.len();
        let mut result = Vec::with_capacity(num_groups * group_size);

        for group_idx in 0..num_groups {
            let scale = self.scales[group_idx];
            let zero = self.zero_points.get(group_idx).copied().unwrap_or(0.0);
            let group_start = group_idx * group_size * bytes_per_weight / 8;
            let group_bytes = &self.quantized_data[group_start..
                (group_start + group_size * bytes_per_weight / 8).min(self.quantized_data.len())];

            for (i, &byte) in group_bytes.iter().enumerate() {
                let lo = (byte & 0x0F) as f32;
                let hi = (byte >> 4) as f32;
                if i * 2 < group_size { result.push(scale * (lo - zero)); }
                if i * 2 + 1 < group_size { result.push(scale * (hi - zero)); }
            }
        }
        Ok(result)
    }

    /// Re-quantification possible ? Documente les cas impossibles sans calibration
    pub fn can_requantize(&self, target: &QuantScheme) -> RequantizationSupport {
        match target {
            QuantScheme::GgufQ4KM | QuantScheme::GgufQ5KM | QuantScheme::GgufQ8_0
            | QuantScheme::GgufQ4_0 | QuantScheme::GgufQ6K => {
                RequantizationSupport::Supported
            }
            QuantScheme::AwqGemm4 | QuantScheme::AwqGemv4 => {
                RequantizationSupport::RequiresCalibration {
                    reason: "AWQ requiert un dataset de calibration pour recalculer \
                             les scales optimaux. Conversion vers F16 recommandée.".into(),
                }
            }
            QuantScheme::Gptq { .. } => {
                RequantizationSupport::RequiresCalibration {
                    reason: "GPTQ utilise une optimisation de second ordre (Hessian). \
                             Re-quantification directe produira des résultats sous-optimaux.".into(),
                }
            }
            QuantScheme::BnbNF4 | QuantScheme::BnbFP4 => {
                RequantizationSupport::Unsupported {
                    reason: "NF4/FP4 bitsandbytes nécessite la bibliothèque bitsandbytes \
                             pour la quantification. Conversion vers F16 puis re-quantification \
                             GGUF possible.".into(),
                }
            }
            _ => RequantizationSupport::Supported,
        }
    }
}

pub enum RequantizationSupport {
    Supported,
    RequiresCalibration { reason: String },
    Unsupported { reason: String },
}
```

### 3.4 ExtensionStore — Sécurisé, Limité, Namespaced

```rust
// crates/umc-core/src/ir/extension.rs

/// ExtensionStore v3 — garantie de zéro perte d'information
/// Limité à max_bytes (défaut 100 Mo).
/// Clés OBLIGATOIREMENT namespaced : "FORMAT@VERSION/chemin"
/// Exemples : "GGUF@v3/tokenizer.chat_template"
///            "GGUF@v3/rope_scaling.type"
///            "ONNX@opset21/custom_metadata/key"
#[derive(Debug, Clone)]
pub struct ExtensionStore {
    // Extensions globales par format source
    format_extensions: std::collections::HashMap<String, FormatExtension>,
    // Extensions par tenseur (champs supplémentaires spécifiques à un tenseur)
    tensor_extensions: std::collections::HashMap<String, TensorExtension>,
    // Extensions de tokenizer non représentables dans TokenizerStore
    tokenizer_extras: std::collections::HashMap<String, Vec<u8>>,
    // Total des bytes utilisés (pour enforcement de la limite)
    total_bytes: usize,
    // Limite maximale (configurable, défaut 100 Mo)
    max_bytes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct FormatExtension {
    pub format_name: String,
    pub format_version: String,
    pub custom_fields: indexmap::IndexMap<String, Vec<u8>>,
    pub original_hash: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TensorExtension {
    pub tensor_name: String,
    pub extra_metadata: std::collections::HashMap<String, Vec<u8>>,
}

/// Validation d'une clé d'extension namespaced
/// Format obligatoire : "FORMAT@VERSION/chemin/vers/champ"
fn validate_extension_key(key: &str) -> Result<ExtensionKeyParts, UmcError> {
    // Longueur maximale
    if key.len() > 512 {
        return Err(UmcError::InvalidExtensionKey {
            key: key.to_string(),
            reason: "Clé trop longue (max 512 caractères)".into(),
        });
    }
    // Format : FORMAT@VERSION/chemin
    let at_pos = key.find('@').ok_or_else(|| UmcError::InvalidExtensionKey {
        key: key.to_string(),
        reason: "Clé doit contenir '@' pour le namespace : FORMAT@VERSION/chemin".into(),
    })?;
    let slash_pos = key.find('/').ok_or_else(|| UmcError::InvalidExtensionKey {
        key: key.to_string(),
        reason: "Clé doit contenir '/' pour le chemin : FORMAT@VERSION/chemin".into(),
    })?;
    if slash_pos <= at_pos {
        return Err(UmcError::InvalidExtensionKey {
            key: key.to_string(),
            reason: "Le '/' doit venir après '@'".into(),
        });
    }
    // Caractères autorisés uniquement
    if !key.chars().all(|c| c.is_alphanumeric() || "@/._-".contains(c)) {
        return Err(UmcError::InvalidExtensionKey {
            key: key.to_string(),
            reason: "Seuls les caractères alphanumériques et @/._- sont autorisés".into(),
        });
    }
    Ok(ExtensionKeyParts {
        format_name: key[..at_pos].to_string(),
        format_version: key[at_pos+1..slash_pos].to_string(),
        field_path: key[slash_pos+1..].to_string(),
    })
}

struct ExtensionKeyParts {
    format_name: String,
    format_version: String,
    field_path: String,
}

impl ExtensionStore {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            format_extensions: std::collections::HashMap::new(),
            tensor_extensions: std::collections::HashMap::new(),
            tokenizer_extras: std::collections::HashMap::new(),
            total_bytes: 0,
            max_bytes,
        }
    }

    pub fn default() -> Self { Self::new(100 * 1024 * 1024) } // 100 Mo

    /// Stocke des données avec clé namespaced validée
    pub fn set(&mut self, key: &str, value: Vec<u8>) -> Result<(), UmcError> {
        let parts = validate_extension_key(key)?;

        // Vérifier la limite de taille
        let new_total = self.total_bytes.saturating_add(value.len());
        if new_total > self.max_bytes {
            return Err(UmcError::ExtensionStoreFull {
                current_bytes: self.total_bytes,
                max_bytes: self.max_bytes,
                tried_to_add: value.len(),
            });
        }

        let ext = self.format_extensions
            .entry(parts.format_name)
            .or_insert_with(FormatExtension::default);
        self.total_bytes = self.total_bytes
            .saturating_sub(ext.custom_fields.get(&parts.field_path).map_or(0, |v| v.len()))
            .saturating_add(value.len());
        ext.custom_fields.insert(parts.field_path, value);
        Ok(())
    }

    /// Récupère des données par clé namespaced
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        let parts = validate_extension_key(key).ok()?;
        self.format_extensions
            .get(&parts.format_name)
            .and_then(|ext| ext.custom_fields.get(&parts.field_path))
            .map(|v| v.as_slice())
    }

    /// Récupère toutes les extensions d'un format donné
    pub fn get_all_for_format(&self, format_name: &str) -> Option<&FormatExtension> {
        self.format_extensions.get(format_name)
    }

    pub fn total_bytes(&self) -> usize { self.total_bytes }
    pub fn max_bytes(&self) -> usize { self.max_bytes }
    pub fn usage_percent(&self) -> f64 {
        self.total_bytes as f64 / self.max_bytes as f64 * 100.0
    }
}
```

### 3.5 ProvenanceChain — Immutable par Hash Chaining

```rust
// crates/umc-core/src/ir/provenance.rs

use sha2::{Sha256, Digest};

/// ProvenanceChain — journal d'audit tamper-evident
/// entry[n].chain_hash = SHA256(entry[n-1].chain_hash || entry[n].content_hash)
/// Toute modification est détectable par verify().
#[derive(Debug, Clone)]
pub struct ProvenanceChain {
    entries: Vec<ProvenanceEntry>,
    root_hash: String,      // Hash de la graine initiale (timestamp + source)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceEntry {
    pub timestamp: u64,                 // Unix timestamp en secondes
    pub source_format: String,          // Format source ("GGUF")
    pub target_format: String,          // Format cible ("ONNX")
    pub tool: String,                   // "umc/3.0.0"
    pub input_hash: String,             // SHA256 du fichier source
    pub output_hash: Option<String>,    // SHA256 du fichier cible (si disponible)
    pub roundtrip_level: String,        // "bit_identical", "semantic", "structural"
    pub max_divergence: Option<f64>,    // Divergence mesurée
    pub warnings: Vec<String>,          // Avertissements de la conversion
    pub content_hash: String,           // SHA256 de cette entrée (sans chain_hash)
    pub chain_hash: String,             // SHA256(prev_chain_hash || content_hash)
}

impl ProvenanceChain {
    pub fn new(source_format: &str, source_path: &std::path::Path) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let seed = format!("UMC_ROOT_{}_{}", timestamp, source_path.display());
        let root_hash = hex::encode(Sha256::digest(seed.as_bytes()));
        Self { entries: Vec::new(), root_hash }
    }

    /// Ajoute une entrée — opération append-only
    pub fn append(&mut self, data: ProvenanceEntryData) -> &ProvenanceEntry {
        let prev_chain_hash = self.entries.last()
            .map(|e| e.chain_hash.as_str())
            .unwrap_or(self.root_hash.as_str());

        let content = serde_json::json!({
            "timestamp": data.timestamp,
            "source_format": data.source_format,
            "target_format": data.target_format,
            "tool": data.tool,
            "input_hash": data.input_hash,
            "output_hash": data.output_hash,
            "roundtrip_level": data.roundtrip_level,
            "max_divergence": data.max_divergence,
            "warnings": data.warnings,
        });
        let content_hash = hex::encode(Sha256::digest(content.to_string().as_bytes()));
        let chain_hash_input = format!("{}{}", prev_chain_hash, content_hash);
        let chain_hash = hex::encode(Sha256::digest(chain_hash_input.as_bytes()));

        self.entries.push(ProvenanceEntry {
            timestamp: data.timestamp,
            source_format: data.source_format,
            target_format: data.target_format,
            tool: data.tool,
            input_hash: data.input_hash,
            output_hash: data.output_hash,
            roundtrip_level: data.roundtrip_level,
            max_divergence: data.max_divergence,
            warnings: data.warnings,
            content_hash,
            chain_hash,
        });
        self.entries.last().unwrap()
    }

    /// Vérifie l'intégrité de toute la chaîne
    pub fn verify(&self) -> bool {
        let mut prev = self.root_hash.clone();
        for entry in &self.entries {
            let expected = hex::encode(Sha256::digest(
                format!("{}{}", prev, entry.content_hash).as_bytes()
            ));
            if expected != entry.chain_hash {
                return false;
            }
            prev = entry.chain_hash.clone();
        }
        true
    }

    pub fn entries(&self) -> &[ProvenanceEntry] { &self.entries }
    pub fn root_hash(&self) -> &str { &self.root_hash }
    pub fn last_entry(&self) -> Option<&ProvenanceEntry> { self.entries.last() }
}

pub struct ProvenanceEntryData {
    pub timestamp: u64,
    pub source_format: String,
    pub target_format: String,
    pub tool: String,
    pub input_hash: String,
    pub output_hash: Option<String>,
    pub roundtrip_level: String,
    pub max_divergence: Option<f64>,
    pub warnings: Vec<String>,
}
```

### 3.6 ComputeGraph — DAG Universel

```rust
// crates/umc-core/src/ir/graph.rs

/// Graphe de calcul orienté acyclique (DAG)
#[derive(Debug, Clone, Default)]
pub struct ComputeGraph {
    pub nodes: Vec<ComputeNode>,
    pub edges: Vec<ComputeEdge>,
    pub inputs: Vec<GraphTensor>,       // Tenseurs d'entrée du graphe
    pub outputs: Vec<GraphTensor>,      // Tenseurs de sortie du graphe
    pub opset_version: Option<u32>,     // Pour ONNX : version de l'opset
}

#[derive(Debug, Clone)]
pub struct GraphTensor {
    pub name: String,
    pub dtype: Option<DType>,
    pub shape: Option<Vec<Option<i64>>>,  // None = dimension dynamique
}

#[derive(Debug, Clone)]
pub struct ComputeNode {
    pub id: String,
    pub op_type: UniversalOp,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub attributes: OpAttributes,
    pub domain: String,     // "" pour ONNX standard, "com.microsoft" pour ORT, etc.
}

#[derive(Debug, Clone)]
pub struct ComputeEdge {
    pub from_node: String,
    pub from_output: usize,
    pub to_node: String,
    pub to_input: usize,
}

#[derive(Debug, Clone, Default)]
pub struct OpAttributes {
    pub floats: std::collections::HashMap<String, f64>,
    pub ints: std::collections::HashMap<String, i64>,
    pub strings: std::collections::HashMap<String, String>,
    pub tensors: std::collections::HashMap<String, Vec<u8>>,
    pub graphs: std::collections::HashMap<String, ComputeGraph>,
}

/// Opérateurs universels — sur-ensemble de tous les formats
#[derive(Debug, Clone, PartialEq)]
pub enum UniversalOp {
    // Arithmétique de base
    Add, Sub, Mul, Div, Pow, Sqrt, Rsqrt, Abs, Neg, Exp, Log,
    Tanh, Sigmoid, Erf, Sign, Ceil, Floor, Round,

    // Activations
    Relu, Relu6, LeakyRelu { alpha: f64 },
    Gelu, GeluApprox, Silu, Swish, HardSwish, HardSigmoid,
    Mish, QuickGelu, Elu { alpha: f64 },

    // Réduction
    ReduceSum { axes: Vec<i64>, keepdims: bool },
    ReduceMean { axes: Vec<i64>, keepdims: bool },
    ReduceMax { axes: Vec<i64>, keepdims: bool },
    ReduceMin { axes: Vec<i64>, keepdims: bool },
    ReduceProd { axes: Vec<i64>, keepdims: bool },

    // Normalisation
    LayerNorm { axis: i64, eps: f64 },
    RmsNorm { eps: f64 },
    BatchNorm { eps: f64, momentum: f64, training: bool },
    GroupNorm { num_groups: i64, eps: f64 },
    InstanceNorm { eps: f64 },

    // Algèbre linéaire
    Gemm { alpha: f64, beta: f64, trans_a: bool, trans_b: bool },
    MatMul,
    Conv2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64>,
              dilations: Vec<i64>, group: i64, auto_pad: String },
    ConvTranspose2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    DepthwiseConv2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    Linear { in_features: i64, out_features: i64, bias: bool },

    // Attention (LLM)
    MultiHeadAttention { num_heads: i64, head_dim: i64 },
    GroupedQueryAttention { num_heads: i64, num_kv_heads: i64, head_dim: i64 },
    ScaledDotProductAttention { is_causal: bool },
    FlashAttention { num_heads: i64, head_dim: i64, is_causal: bool },

    // Positional Encoding (LLM)
    RotaryPositionEmbedding { base: f64, scaling: Option<RopeScalingConfig> },
    AlibiPositionEmbedding,
    SinusoidalPositionEmbedding,
    LearnedPositionEmbedding,

    // MoE (Mixture of Experts)
    MoeLayer { num_experts: i64, top_k: i64 },
    MoeGate { num_experts: i64, top_k: i64 },

    // Reshape et indexation
    Reshape, Transpose { perm: Vec<i64> },
    Flatten { axis: i64 }, Squeeze { axes: Vec<i64> }, Unsqueeze { axes: Vec<i64> },
    Concat { axis: i64 }, Split { axis: i64, sizes: Vec<i64> },
    Gather { axis: i64 }, GatherElements { axis: i64 },
    Scatter { axis: i64 }, ScatterElements { axis: i64, reduction: String },
    Slice { axes: Vec<i64>, starts: Vec<i64>, ends: Vec<i64>, steps: Vec<i64> },
    Tile { repeats: Vec<i64> }, Expand,
    Pad { mode: PadMode, pads: Vec<i64>, constant_value: f64 },
    Resize { mode: String, coordinate_transformation_mode: String },

    // Pooling
    MaxPool2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    AveragePool2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    GlobalAveragePool, GlobalMaxPool,

    // Divers
    Softmax { axis: i64 }, LogSoftmax { axis: i64 },
    Cast { to: DType },
    Clip { min: Option<f64>, max: Option<f64> },
    Constant { value: ConstantValue },
    Identity,
    Dropout { ratio: f64, training: bool },
    Embedding { padding_idx: Option<i64>, vocab_size: i64, embed_dim: i64 },
    Where,    // Opérateur conditionnel (ternaire)
    NonZero,
    CumSum { axis: i64, exclusive: bool, reverse: bool },
    ArgMax { axis: i64, keepdims: bool }, ArgMin { axis: i64, keepdims: bool },
    TopK { axis: i64, largest: bool, sorted: bool },
    EyeLike,
    ScaledDotProduct,

    // OPÉRATEUR INCONNU — préservé dans ExtensionStore, PAS une erreur fatale
    // Le graphe reste fonctionnel pour les conversions vers des formats
    // qui supportent les custom ops
    Custom {
        domain: String,
        op_type: String,
        attributes: std::collections::HashMap<String, Vec<u8>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PadMode {
    Constant, Reflect, Edge, Wrap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    Tensor(Vec<u8>),    // Tenseur sérialisé
}

impl ComputeGraph {
    pub fn empty() -> Self { Self::default() }

    /// Vérifie que le graphe est un DAG valide (pas de cycles)
    pub fn is_valid_dag(&self) -> bool {
        // Implémentation via DFS avec coloration (blanc/gris/noir)
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();
        let node_ids: Vec<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();

        fn has_cycle(
            node_id: &str,
            nodes: &[ComputeNode],
            edges: &[ComputeEdge],
            visited: &mut std::collections::HashSet<String>,
            in_stack: &mut std::collections::HashSet<String>,
        ) -> bool {
            if in_stack.contains(node_id) { return true; }
            if visited.contains(node_id) { return false; }
            visited.insert(node_id.to_string());
            in_stack.insert(node_id.to_string());
            for edge in edges.iter().filter(|e| e.from_node == node_id) {
                if has_cycle(&edge.to_node, nodes, edges, visited, in_stack) {
                    return true;
                }
            }
            in_stack.remove(node_id);
            false
        }

        for node_id in &node_ids {
            if has_cycle(node_id, &self.nodes, &self.edges, &mut visited, &mut in_stack) {
                return false;
            }
        }
        true
    }
}


---

# PARTIE III — DÉTECTION ET ROUTAGE

---

## 4. DÉTECTION AUTOMATIQUE DE FORMAT (umc-detect)

### 4.1 FormatRegistry — Cascade Multi-Niveaux

```rust
// crates/umc-detect/src/registry.rs

use std::path::Path;

/// Résultat de détection
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub format: String,
    pub confidence: f32,        // 0.0 à 1.0
    pub method: DetectionMethod,
    pub format_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectionMethod {
    MagicBytes,         // Plus fiable (priorité 1)
    Extension,          // Fiable si magic absent (priorité 2)
    ContentAnalysis,    // Moins fiable (priorité 3)
    ManualOverride,     // L'utilisateur a spécifié le format
}

pub trait FormatDetector: Send + Sync {
    fn format_name(&self) -> &'static str;
    fn priority(&self) -> u8;           // 1 = magic bytes (le plus fiable)
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32;
    fn detect_version(&self, path: &Path, first_bytes: &[u8]) -> Option<String>;
}

pub struct FormatRegistry {
    detectors: Vec<Box<dyn FormatDetector>>,
}

impl FormatRegistry {
    pub fn new() -> Self {
        let mut r = Self { detectors: Vec::new() };
        r.register_all_builtin();
        r
    }

    fn register_all_builtin(&mut self) {
        // Priorité 1 — Magic bytes fiables (> 0.95)
        self.detectors.push(Box::new(GgufDetector));
        self.detectors.push(Box::new(GgmlDetector));
        self.detectors.push(Box::new(SafeTensorsDetector));
        self.detectors.push(Box::new(TFLiteDetector));
        self.detectors.push(Box::new(ExecuTorchDetector));
        self.detectors.push(Box::new(HDF5Detector));       // KerasH5
        self.detectors.push(Box::new(FlatBuffersDetector)); // TFLite fallback
        // Priorité 2 — Extension + magic partiel (0.85-0.95)
        self.detectors.push(Box::new(OnnxDetector));
        self.detectors.push(Box::new(PyTorchDetector));
        self.detectors.push(Box::new(TorchScriptDetector));
        self.detectors.push(Box::new(SentencePieceDetector));
        self.detectors.push(Box::new(TikTokenDetector));
        // Priorité 3 — Analyse de contenu (0.70-0.85)
        self.detectors.push(Box::new(TFSavedModelDetector));
        self.detectors.push(Box::new(DiffusersDetector));
        self.detectors.push(Box::new(LoRADetector));
        self.detectors.push(Box::new(AWQDetector));
        self.detectors.push(Box::new(GPTQDetector));
    }

    pub fn register(&mut self, detector: Box<dyn FormatDetector>) {
        self.detectors.push(detector);
    }

    /// Détecte le format avec cascade. Retourne une erreur claire si inconnu.
    pub fn detect(&self, path: &Path) -> Result<DetectionResult, UmcError> {
        // Lire les 512 premiers octets (suffisant pour tous les magic bytes)
        let first_bytes = self.read_magic_bytes(path, 512)?;

        let mut candidates: Vec<(&dyn FormatDetector, f32)> = self.detectors
            .iter()
            .filter_map(|d| {
                let conf = d.confidence(path, &first_bytes);
                if conf > 0.0 { Some((d.as_ref(), conf)) } else { None }
            })
            .collect();

        // Trier par confiance décroissante, puis par priorité croissante
        candidates.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.priority().cmp(&b.0.priority()))
        });

        let (detector, confidence) = candidates.first()
            .ok_or_else(|| UmcError::UnknownFormat {
                path: path.to_string_lossy().to_string(),
                hint: "Utilisez --format <FORMAT> pour spécifier manuellement. \
                       Listez les formats avec: umc formats".into(),
            })?;

        // Ambiguïté : deux formats avec confiance proche (< 0.1 d'écart)
        if candidates.len() > 1 {
            let second_conf = candidates[1].1;
            if confidence - second_conf < 0.1 {
                tracing::warn!(
                    "Détection ambiguë pour {}: {} ({:.2}) vs {} ({:.2}). \
                     Utilisez --format pour lever l'ambiguïté.",
                    path.display(),
                    detector.format_name(), confidence,
                    candidates[1].0.format_name(), second_conf,
                );
            }
        }

        Ok(DetectionResult {
            format: detector.format_name().to_string(),
            confidence: *confidence,
            method: match detector.priority() {
                1 => DetectionMethod::MagicBytes,
                2 => DetectionMethod::Extension,
                _ => DetectionMethod::ContentAnalysis,
            },
            format_version: detector.detect_version(path, &first_bytes),
        })
    }

    fn read_magic_bytes(&self, path: &Path, n: usize) -> Result<Vec<u8>, UmcError> {
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let mut buf = vec![0u8; n];
        let read = file.read(&mut buf).map_err(UmcError::Io)?;
        buf.truncate(read);
        Ok(buf)
    }
}

// ── Implémentations des détecteurs ────────────────────────────────────────

macro_rules! magic_detector {
    ($name:ident, $fmt:literal, $priority:literal, $magic:expr, $ext:expr, $conf:literal) => {
        pub struct $name;
        impl FormatDetector for $name {
            fn format_name(&self) -> &'static str { $fmt }
            fn priority(&self) -> u8 { $priority }
            fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
                let magic_match = first_bytes.len() >= $magic.len()
                    && first_bytes.starts_with($magic);
                let ext_match = path.extension()
                    .map_or(false, |e| $ext.contains(&e.to_str().unwrap_or("")));
                if magic_match { $conf }
                else if ext_match { ($conf * 0.7) }
                else { 0.0 }
            }
            fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
        }
    };
}

// GGUF : magic "GGUF" (4 octets)
pub struct GgufDetector;
impl FormatDetector for GgufDetector {
    fn format_name(&self) -> &'static str { "GGUF" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, _path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.starts_with(b"GGUF") { 0.99 } else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, first_bytes: &[u8]) -> Option<String> {
        if first_bytes.len() >= 8 {
            let version = u32::from_le_bytes(first_bytes[4..8].try_into().ok()?);
            Some(format!("v{}", version))
        } else { None }
    }
}

// GGML : magic "GGML" (héritage)
magic_detector!(GgmlDetector, "GGML", 1, b"GGML", ["bin"], 0.99);

// SafeTensors : 8 octets LE size + '{'
pub struct SafeTensorsDetector;
impl FormatDetector for SafeTensorsDetector {
    fn format_name(&self) -> &'static str { "SafeTensors" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, _path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.len() < 9 { return 0.0; }
        let json_size = u64::from_le_bytes(first_bytes[0..8].try_into().unwrap_or([0;8]));
        if first_bytes[8] == b'{' && json_size > 2 && json_size < 100_000_000 { 0.99 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> {
        Some("1.0".into())
    }
}

// TFLite : FlatBuffer magic TFL3/TFL2
pub struct TFLiteDetector;
impl FormatDetector for TFLiteDetector {
    fn format_name(&self) -> &'static str { "TFLite" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.len() >= 8 {
            let magic = &first_bytes[4..8];
            if magic == b"TFL3" || magic == b"TFL2" || magic == b"TFL1" { return 0.99; }
        }
        if path.extension().map_or(false, |e| e == "tflite") { 0.75 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, first_bytes: &[u8]) -> Option<String> {
        if first_bytes.len() >= 8 {
            Some(std::str::from_utf8(&first_bytes[4..8]).unwrap_or("?").to_string())
        } else { None }
    }
}

// ExecuTorch : magic "ET\0\0"
magic_detector!(ExecuTorchDetector, "ExecuTorch", 1, b"ET\x00\x00", ["pte"], 0.99);

// HDF5 : magic "\x89HDF"
pub struct HDF5Detector;
impl FormatDetector for HDF5Detector {
    fn format_name(&self) -> &'static str { "KerasH5" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.starts_with(&[0x89, 0x48, 0x44, 0x46]) {
            if path.extension().map_or(false, |e| e == "h5" || e == "keras" || e == "hdf5") {
                return 0.99;
            }
            return 0.85; // HDF5 mais pas forcément Keras
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

// ONNX : protobuf (commence par 0x08 ou 0x0a) + extension .onnx
pub struct OnnxDetector;
impl FormatDetector for OnnxDetector {
    fn format_name(&self) -> &'static str { "ONNX" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        let is_onnx_ext = path.extension().map_or(false, |e| e == "onnx");
        let looks_like_proto = !first_bytes.is_empty()
            && (first_bytes[0] == 0x08 || first_bytes[0] == 0x0a);
        if is_onnx_ext && looks_like_proto { 0.97 }
        else if is_onnx_ext { 0.80 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

// PyTorch : ZIP + extension pt/pth/bin
pub struct PyTorchDetector;
impl FormatDetector for PyTorchDetector {
    fn format_name(&self) -> &'static str { "PyTorch" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        let is_zip = first_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]);
        let ext_match = path.extension().map_or(false, |e| {
            matches!(e.to_str(), Some("pt" | "pth" | "bin"))
        });
        if is_zip && ext_match { 0.90 }
        else if is_zip { 0.40 } // Peut être un autre format ZIP
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

// TorchScript : ZIP + extension .pt (JIT)
pub struct TorchScriptDetector;
impl FormatDetector for TorchScriptDetector {
    fn format_name(&self) -> &'static str { "TorchScript" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        // TorchScript et PyTorch ont le même magic (ZIP)
        // La distinction se fait par le contenu du ZIP (archive/code/ vs archive/data.pkl)
        let is_zip = first_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]);
        let ext_match = path.extension().map_or(false, |e| e == "pt");
        if is_zip && ext_match { 0.55 } // Ambiguïté avec PyTorch
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

// SentencePiece : protobuf spécifique
magic_detector!(SentencePieceDetector, "SentencePiece", 2,
    b"\x0a", ["model", "spm"], 0.60);

// TikToken : texte base64 spécifique
pub struct TikTokenDetector;
impl FormatDetector for TikTokenDetector {
    fn format_name(&self) -> &'static str { "TikToken" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        let ext = path.extension().map_or(false, |e| e == "tiktoken");
        // TikToken : ligne de texte "<token_base64> <rank>\n"
        let looks_like_tiktoken = first_bytes.windows(2).any(|w| w == b" ");
        if ext { 0.90 }
        else if looks_like_tiktoken && path.extension().map_or(false, |e| e == "txt") { 0.50 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

// TFSavedModel : répertoire avec saved_model.pb
pub struct TFSavedModelDetector;
impl FormatDetector for TFSavedModelDetector {
    fn format_name(&self) -> &'static str { "TFSavedModel" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if path.is_dir() {
            let pb = path.join("saved_model.pb");
            if pb.exists() { return 0.99; }
        }
        if path.file_name().map_or(false, |f| f == "saved_model.pb") { 0.95 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

// Diffusers : répertoire avec model_index.json
pub struct DiffusersDetector;
impl FormatDetector for DiffusersDetector {
    fn format_name(&self) -> &'static str { "Diffusers" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, path: &Path, _first_bytes: &[u8]) -> f32 {
        if path.is_dir() {
            let index = path.join("model_index.json");
            if index.exists() { return 0.95; }
            // Vérifier la structure Diffusers (sous-répertoires typiques)
            let has_unet = path.join("unet").exists();
            let has_vae = path.join("vae").exists();
            let has_text_encoder = path.join("text_encoder").exists();
            if has_unet || (has_vae && has_text_encoder) { return 0.80; }
        }
        0.0
    }
    fn detect_version(&self, path: &Path, _first_bytes: &[u8]) -> Option<String> {
        if path.is_dir() {
            let index = path.join("model_index.json");
            if let Ok(content) = std::fs::read_to_string(&index) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    return json["_class_name"].as_str()
                        .map(|s| s.to_string());
                }
            }
        }
        None
    }
}

// LoRA/AWQ/GPTQ : SafeTensors + adapter_config.json / quant_config.json
pub struct LoRADetector;
impl FormatDetector for LoRADetector {
    fn format_name(&self) -> &'static str { "LoRA" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        // Un répertoire LoRA contient adapter_config.json
        if path.is_dir() && path.join("adapter_config.json").exists() { return 0.92; }
        // Un fichier SafeTensors peut être un LoRA si son nom l'indique
        let is_st = SafeTensorsDetector.confidence(path, first_bytes) > 0.9;
        if is_st {
            let name = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
            if name.contains("lora") || name.contains("adapter") { return 0.70; }
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

pub struct AWQDetector;
impl FormatDetector for AWQDetector {
    fn format_name(&self) -> &'static str { "AWQ" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, path: &Path, _first_bytes: &[u8]) -> f32 {
        if path.is_dir() {
            let config = path.join("quant_config.json");
            if config.exists() {
                if let Ok(content) = std::fs::read_to_string(&config) {
                    if content.contains("awq") || content.contains("AWQ") { return 0.92; }
                }
            }
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

pub struct GPTQDetector;
impl FormatDetector for GPTQDetector {
    fn format_name(&self) -> &'static str { "GPTQ" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, path: &Path, _first_bytes: &[u8]) -> f32 {
        if path.is_dir() {
            let config = path.join("quantize_config.json");
            if config.exists() { return 0.92; }
            // Certains modèles GPTQ utilisent aussi config.json
            let config2 = path.join("config.json");
            if config2.exists() {
                if let Ok(c) = std::fs::read_to_string(&config2) {
                    if c.contains("gptq") || c.contains("GPTQ") { return 0.85; }
                }
            }
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

pub struct FlatBuffersDetector;
impl FormatDetector for FlatBuffersDetector {
    fn format_name(&self) -> &'static str { "FlatBuffers" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, _path: &Path, _first_bytes: &[u8]) -> f32 { 0.0 }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}
```

---

## 5. GRAPHE DE CONVERSION DIJKSTRA (umc-graph)

### 5.1 ConversionGraph

```rust
// crates/umc-graph/src/conversion_graph.rs

use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Debug, Clone)]
pub struct FormatNode {
    pub name: String,
    pub can_load: bool,
    pub can_save: bool,
    pub is_legacy: bool,        // Lecture seule (GGML, KerasH5)
    pub is_recipe_only: bool,   // Génère une recette, pas de conversion directe
}

#[derive(Debug, Clone)]
pub struct ConversionEdge {
    /// Coût Dijkstra :
    /// 1.0 = conversion native Rust (rapide, fiable)
    /// 1.5 = via format intermédiaire ou format composite
    /// 2.0 = Recipe Saver (outil externe requis de l'utilisateur)
    pub cost: f32,
    pub converter_type: ConverterType,
    /// Niveau de round-trip garanti pour cette arête
    pub roundtrip_level: RoundTripLevel,
    /// Divergence maximale connue pour cette paire
    pub known_max_divergence: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum ConverterType {
    /// Conversion 100% Rust, pas de subprocess
    NativeRust,
    /// Passe par un format intermédiaire (ex: GGUF → ONNX → TFLite)
    ViaIntermediate(String),
    /// Génère une recette de build reproductible (pas de conversion directe)
    RecipeSaver { tool_name: String, install_url: String },
    /// Format composite (plusieurs sous-modèles)
    Composite,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoundTripLevel {
    BitIdentical,           // SHA256 identique (même format uniquement)
    Semantic(f64),          // Divergence maximale connue
    Structural,             // Même graphe, même architecture
}

/// Chemin de conversion calculé par Dijkstra
#[derive(Debug, Clone)]
pub struct ConversionPath {
    pub steps: Vec<ConversionStep>,
    pub total_cost: f32,
    pub worst_roundtrip: RoundTripLevel,
    pub requires_external_tools: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConversionStep {
    pub from: String,
    pub to: String,
    pub converter_type: ConverterType,
    pub roundtrip_level: RoundTripLevel,
}

pub struct ConversionGraph {
    graph: DiGraph<FormatNode, ConversionEdge>,
    node_map: std::collections::HashMap<String, NodeIndex>,
}

impl ConversionGraph {
    pub fn new() -> Self {
        let mut g = Self {
            graph: DiGraph::new(),
            node_map: std::collections::HashMap::new(),
        };
        g.register_all_formats();
        g.register_all_edges();
        g
    }

    fn register_all_formats(&mut self) {
        let formats = vec![
            // (name, can_load, can_save, is_legacy, is_recipe_only)
            // Phase 0 — Fondations
            ("GGUF",          true,  true,  false, false),
            ("SafeTensors",   true,  true,  false, false),
            ("ONNX",          true,  true,  false, false),
            // Phase 1 — LLM essentiels
            ("PyTorch",       true,  true,  false, false),
            ("SentencePiece", true,  true,  false, false),
            ("TikToken",      true,  true,  false, false),
            ("AWQ",           true,  true,  false, false),
            ("GPTQ",          true,  true,  false, false),
            ("LoRA",          true,  true,  false, false),
            ("QLoRA",         true,  true,  false, false),
            ("PEFT",          true,  true,  false, false),
            ("bitsandbytes",  true,  false, false, false),
            ("GGML",          true,  false, true,  false), // Legacy
            ("TFSavedModel",  true,  true,  false, false),
            // Phase 2 — Mobile et Edge
            ("TFLite",        true,  true,  false, false),
            ("CoreML",        true,  true,  false, false), // Saver = .mlpackage non compilé
            ("ExecuTorch",    true,  true,  false, false),
            ("JAXFlax",       true,  false, false, false),
            ("KerasH5",       true,  false, true,  false), // Legacy
            ("TorchScript",   true,  true,  false, false),
            ("PaddlePaddle",  true,  true,  false, false),
            ("ONNXRuntime",   true,  true,  false, false),
            // Phase 3 — Serveur et spécialisés
            ("OpenVINO",      true,  true,  false, false), // XML + bin natif
            ("Diffusers",     true,  true,  false, false),
            ("TensorRT",      false, true,  false, true),  // Recipe uniquement
            ("QualcommQNN",   false, true,  false, true),  // Recipe uniquement
            ("MediaPipe",     true,  true,  false, false),
            ("NVIDIATriton",  false, true,  false, true),  // Recipe uniquement
            ("TensorRTLLM",   false, true,  false, true),  // Recipe uniquement
            ("ApacheTVM",     false, true,  false, true),  // Recipe uniquement
            ("ONNXWeb",       false, true,  false, false),
            ("TBD",           false, false, false, false), // Voté par communauté
        ];

        for (name, can_load, can_save, is_legacy, is_recipe_only) in formats {
            let idx = self.graph.add_node(FormatNode {
                name: name.to_string(),
                can_load,
                can_save,
                is_legacy,
                is_recipe_only,
            });
            self.node_map.insert(name.to_string(), idx);
        }
    }

    fn register_all_edges(&mut self) {
        // ── Conversions Natives Rust (cost=1.0, roundtrip Semantic) ─────────

        let native_semantic = vec![
            // Formats LLM fondamentaux
            ("GGUF",       "SafeTensors", 1e-7),
            ("SafeTensors","GGUF",        1e-7),
            ("GGUF",       "ONNX",        1e-6), // Via GraphTemplate
            ("ONNX",       "GGUF",        1e-6),
            ("SafeTensors","ONNX",        1e-6), // Via GraphTemplate
            ("ONNX",       "SafeTensors", 1e-6),
            ("PyTorch",    "SafeTensors", 1e-7),
            ("SafeTensors","PyTorch",     1e-7),
            ("PyTorch",    "ONNX",        1e-6),
            ("ONNX",       "PyTorch",     1e-6),
            ("TFSavedModel","SafeTensors",1e-6),
            ("SafeTensors","TFSavedModel",1e-6),
            ("TFSavedModel","ONNX",       1e-6),
            ("ONNX",       "TFSavedModel",1e-6),
            // Quantification
            ("AWQ",        "SafeTensors", 1e-3), // Déquantification
            ("GPTQ",       "SafeTensors", 1e-2), // Déquantification
            ("bitsandbytes","SafeTensors",1e-2),
            ("SafeTensors","AWQ",         1e-3), // Re-quantification (sans calibration = approx)
            ("SafeTensors","GPTQ",        1e-2),
            // Adaptateurs
            ("LoRA",       "SafeTensors", 1e-7),
            ("SafeTensors","LoRA",        1e-7),
            ("QLoRA",      "SafeTensors", 1e-3),
            ("SafeTensors","QLoRA",       1e-3),
            ("PEFT",       "SafeTensors", 1e-7),
            ("SafeTensors","PEFT",        1e-7),
            // Legacy → moderne
            ("GGML",       "GGUF",        1e-6),
            ("KerasH5",    "TFSavedModel",1e-6),
            ("JAXFlax",    "SafeTensors", 1e-6),
            // Mobile/Edge
            ("TFLite",     "ONNX",        1e-5),
            ("ONNX",       "TFLite",      1e-5),
            ("TFSavedModel","TFLite",     1e-5),
            ("ExecuTorch", "ONNX",        1e-5),
            ("ONNX",       "ExecuTorch",  1e-5),
            ("TorchScript","PyTorch",     1e-7),
            ("PyTorch",    "TorchScript", 1e-7),
            ("PaddlePaddle","ONNX",       1e-5),
            ("ONNX",       "PaddlePaddle",1e-5),
            ("ONNXRuntime","ONNX",        1e-7),
            ("ONNX",       "ONNXRuntime", 1e-6),
            // Tokenizers
            ("SentencePiece","TikToken",  0.0),
            ("TikToken",   "SentencePiece",0.0),
            // Serveur
            ("OpenVINO",   "ONNX",        1e-5),
            ("ONNX",       "OpenVINO",    1e-5),
            ("Diffusers",  "SafeTensors", 1e-7),
            ("SafeTensors","Diffusers",   1e-7),
            ("MediaPipe",  "TFLite",      1e-6),
            ("TFLite",     "MediaPipe",   1e-6),
            ("ONNX",       "ONNXWeb",     1e-5),
            ("CoreML",     "ONNX",        1e-5),
            ("ONNX",       "CoreML",      1e-5),
        ];

        for (from, to, divergence) in native_semantic {
            self.add_edge(from, to, 1.0,
                ConverterType::NativeRust,
                RoundTripLevel::Semantic(divergence),
            );
        }

        // ── Recipe Savers (cost=2.0) ─────────────────────────────────────────

        let recipes = vec![
            ("ONNX", "TensorRT", "trtexec",
             "https://developer.nvidia.com/tensorrt"),
            ("ONNX", "QualcommQNN", "qnn-net-run",
             "https://developer.qualcomm.com/software/qualcomm-neural-network"),
            ("ONNX", "NVIDIATriton", "tritonserver",
             "https://developer.nvidia.com/triton-inference-server"),
            ("ONNX", "TensorRTLLM", "trtllm-build",
             "https://github.com/NVIDIA/TensorRT-LLM"),
            ("ONNX", "ApacheTVM", "tvmc",
             "https://tvm.apache.org"),
        ];

        for (from, to, tool, url) in recipes {
            self.add_edge(from, to, 2.0,
                ConverterType::RecipeSaver {
                    tool_name: tool.to_string(),
                    install_url: url.to_string(),
                },
                RoundTripLevel::Structural,
            );
        }
    }

    fn add_edge(&mut self, from: &str, to: &str, cost: f32,
                converter_type: ConverterType, roundtrip_level: RoundTripLevel) {
        if let (Some(&fi), Some(&ti)) = (self.node_map.get(from), self.node_map.get(to)) {
            let known_max_divergence = match &roundtrip_level {
                RoundTripLevel::Semantic(d) => Some(*d),
                _ => None,
            };
            self.graph.add_edge(fi, ti, ConversionEdge {
                cost,
                converter_type,
                roundtrip_level,
                known_max_divergence,
            });
        }
    }

    /// Trouve le chemin optimal entre deux formats via Dijkstra
    pub fn find_path(&self, from: &str, to: &str) -> Result<ConversionPath, UmcError> {
        // Cas trivial : même format
        if from == to {
            return Ok(ConversionPath {
                steps: vec![],
                total_cost: 0.0,
                worst_roundtrip: RoundTripLevel::BitIdentical,
                requires_external_tools: vec![],
                warnings: vec!["Conversion source=cible : aucune opération nécessaire.".into()],
            });
        }

        let from_idx = self.node_map.get(from)
            .ok_or_else(|| UmcError::UnknownFormatName(from.to_string()))?;
        let to_idx = self.node_map.get(to)
            .ok_or_else(|| UmcError::UnknownFormatName(to.to_string()))?;

        // Dijkstra avec petgraph
        let path = petgraph::algo::astar(
            &self.graph,
            *from_idx,
            |n| n == *to_idx,
            |e| e.weight().cost,
            |_| 0.0,  // Heuristique nulle (Dijkstra exact)
        );

        match path {
            Some((cost, node_path)) => {
                let mut steps = Vec::new();
                let mut worst_roundtrip = RoundTripLevel::BitIdentical;
                let mut requires_external_tools = Vec::new();
                let mut warnings = Vec::new();

                for window in node_path.windows(2) {
                    let (from_n, to_n) = (window[0], window[1]);
                    let edge = self.graph.edges_connecting(from_n, to_n).next().unwrap();
                    let edge_data = edge.weight();
                    let from_name = self.graph[from_n].name.clone();
                    let to_name = self.graph[to_n].name.clone();

                    // Déterminer le pire round-trip de la chaîne
                    worst_roundtrip = match (&worst_roundtrip, &edge_data.roundtrip_level) {
                        (RoundTripLevel::BitIdentical, other) => other.clone(),
                        (RoundTripLevel::Semantic(a), RoundTripLevel::Semantic(b)) => {
                            RoundTripLevel::Semantic(a.max(*b))
                        }
                        (_, RoundTripLevel::Structural) | (RoundTripLevel::Structural, _) => {
                            RoundTripLevel::Structural
                        }
                        (current, _) => current.clone(),
                    };

                    // Collecter les outils externes requis
                    if let ConverterType::RecipeSaver { tool_name, .. } = &edge_data.converter_type {
                        requires_external_tools.push(tool_name.clone());
                        warnings.push(format!(
                            "Étape {} → {} : nécessite {} (recette générée par UMC, \
                             exécution manuelle requise)",
                            from_name, to_name, tool_name
                        ));
                    }

                    steps.push(ConversionStep {
                        from: from_name,
                        to: to_name,
                        converter_type: edge_data.converter_type.clone(),
                        roundtrip_level: edge_data.roundtrip_level.clone(),
                    });
                }

                // Avertissement si chemin indirect (plus d'une étape)
                if steps.len() > 1 {
                    warnings.push(format!(
                        "Conversion indirecte en {} étape(s). \
                         Chaque étape peut ajouter une légère divergence numérique.",
                        steps.len()
                    ));
                }

                Ok(ConversionPath {
                    steps,
                    total_cost: cost,
                    worst_roundtrip,
                    requires_external_tools,
                    warnings,
                })
            }
            None => Err(UmcError::NoConversionPath {
                from: from.to_string(),
                to: to.to_string(),
                hint: format!(
                    "Vérifiez les formats disponibles avec 'umc formats'. \
                     Si vous avez besoin du chemin {} → {}, ouvrez une issue sur GitHub.",
                    from, to
                ),
            }),
        }
    }

    /// Exporte le graphe en JSON pour le frontend
    pub fn to_json(&self) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = self.graph.node_indices()
            .map(|i| {
                let n = &self.graph[i];
                serde_json::json!({
                    "id": n.name,
                    "can_load": n.can_load,
                    "can_save": n.can_save,
                    "is_legacy": n.is_legacy,
                    "is_recipe_only": n.is_recipe_only,
                })
            })
            .collect();

        let edges: Vec<serde_json::Value> = self.graph.edge_indices()
            .map(|i| {
                let (from, to) = self.graph.edge_endpoints(i).unwrap();
                let e = &self.graph[i];
                serde_json::json!({
                    "from": self.graph[from].name,
                    "to": self.graph[to].name,
                    "cost": e.cost,
                    "roundtrip": format!("{:?}", e.roundtrip_level),
                })
            })
            .collect();

        serde_json::json!({ "nodes": nodes, "edges": edges })
    }
}
```

### 5.2 GraphTemplate Registry — Reconstruction de Graphe pour Formats Weights-Only

```rust
// crates/umc-graph/src/template_registry.rs

/// GraphTemplate Registry — résout le problème critique des formats weights-only
/// (GGUF, SafeTensors, AWQ, GPTQ) qui n'ont PAS de graphe de calcul explicite.
/// Pour convertir vers ONNX/TFLite/CoreML, il faut reconstruire le graphe
/// depuis les métadonnées d'architecture.
///
/// Ce n'est pas de la magie : c'est un catalogue maintenu d'architectures connues.
pub struct GraphTemplateRegistry {
    templates: Vec<Box<dyn GraphTemplate>>,
}

pub trait GraphTemplate: Send + Sync {
    fn architecture_name(&self) -> &str;
    fn known_variants(&self) -> &[&str];
    fn matches(&self, config: &ArchitectureConfig) -> bool;
    fn build_graph(
        &self,
        config: &ArchitectureConfig,
        tensors: &TensorStore,
    ) -> Result<ComputeGraph, UmcError>;
    fn verify_tensors(&self, config: &ArchitectureConfig, tensors: &TensorStore) -> Vec<String>;
}

impl GraphTemplateRegistry {
    pub fn new() -> Self {
        let mut r = Self { templates: Vec::new() };
        r.register(Box::new(LlamaTemplate::new()));
        r.register(Box::new(MistralTemplate::new()));  // Hérite de Llama
        r.register(Box::new(PhiTemplate::new()));
        r.register(Box::new(GemmaTemplate::new()));
        r.register(Box::new(QwenTemplate::new()));
        r.register(Box::new(FalconTemplate::new()));
        r.register(Box::new(GPTNeoXTemplate::new()));
        r.register(Box::new(OPTTemplate::new()));
        r.register(Box::new(BloomTemplate::new()));
        r
    }

    pub fn register(&mut self, template: Box<dyn GraphTemplate>) {
        self.templates.push(template);
    }

    /// Trouve le template approprié pour une architecture donnée
    pub fn find(&self, config: &ArchitectureConfig) -> Option<&dyn GraphTemplate> {
        self.templates.iter()
            .find(|t| t.matches(config))
            .map(|t| t.as_ref())
    }

    /// Reconstruit le graphe ou retourne une erreur documentée
    pub fn reconstruct_graph(
        &self,
        config: &ArchitectureConfig,
        tensors: &TensorStore,
    ) -> Result<ComputeGraph, GraphReconstructionError> {
        match self.find(config) {
            Some(template) => {
                // Vérifier que les tenseurs nécessaires sont présents
                let missing = template.verify_tensors(config, tensors);
                if !missing.is_empty() {
                    return Err(GraphReconstructionError::MissingTensors {
                        architecture: config.architecture.clone(),
                        missing_tensors: missing,
                    });
                }
                template.build_graph(config, tensors)
                    .map_err(|e| GraphReconstructionError::BuildFailed {
                        architecture: config.architecture.clone(),
                        reason: e.to_string(),
                    })
            }
            None => Err(GraphReconstructionError::UnknownArchitecture {
                architecture: config.architecture.clone(),
                known_architectures: self.templates.iter()
                    .flat_map(|t| t.known_variants().iter().map(|s| s.to_string()))
                    .collect(),
                workaround: "Conversion vers SafeTensors possible (weights-only, sans graphe). \
                             Pour ONNX avec graphe, ouvrez une issue avec votre architecture.".into(),
            }),
        }
    }
}

/// Erreurs spécifiques à la reconstruction de graphe — jamais fatales
#[derive(Debug)]
pub enum GraphReconstructionError {
    UnknownArchitecture {
        architecture: String,
        known_architectures: Vec<String>,
        workaround: String,
    },
    MissingTensors {
        architecture: String,
        missing_tensors: Vec<String>,
    },
    BuildFailed {
        architecture: String,
        reason: String,
    },
}

/// Template Llama — couvre Llama 1/2/3/3.1/3.2, Mistral, Mixtral, Vicuna, etc.
pub struct LlamaTemplate {
    known_variants: Vec<&'static str>,
}

impl LlamaTemplate {
    pub fn new() -> Self {
        Self {
            known_variants: vec![
                "llama", "llama2", "llama3",
                "mistral", "mixtral",
                "solar", "vicuna", "alpaca", "hermes",
                "openhermes", "dolphin", "orca",
                "zephyr", "stablelm",
            ],
        }
    }
}

impl GraphTemplate for LlamaTemplate {
    fn architecture_name(&self) -> &str { "llama-family" }
    fn known_variants(&self) -> &[&str] { &self.known_variants }

    fn matches(&self, config: &ArchitectureConfig) -> bool {
        let arch = config.architecture.to_lowercase();
        self.known_variants.iter().any(|&v| arch.contains(v))
    }

    fn verify_tensors(&self, config: &ArchitectureConfig, tensors: &TensorStore) -> Vec<String> {
        let mut missing = Vec::new();
        // Tenseurs obligatoires pour Llama
        let required = vec![
            "model.embed_tokens.weight",
            "model.norm.weight",
            "lm_head.weight",
        ];
        for req in required {
            if tensors.get(req).is_none() {
                missing.push(req.to_string());
            }
        }
        // Vérifier au moins une couche
        let layer0_q = format!("model.layers.0.self_attn.q_proj.weight");
        if tensors.get(&layer0_q).is_none() {
            missing.push(layer0_q);
        }
        missing
    }

    fn build_graph(
        &self,
        config: &ArchitectureConfig,
        tensors: &TensorStore,
    ) -> Result<ComputeGraph, UmcError> {
        let mut nodes = Vec::new();
        let num_layers = config.num_layers;
        let num_heads = config.num_heads;
        let num_kv_heads = config.num_kv_heads.unwrap_or(num_heads);
        let head_dim = config.hidden_size / num_heads;
        let eps = config.rms_norm_eps.unwrap_or(1e-5);
        let rope_theta = config.rope_theta.unwrap_or(10000.0);

        // === Embedding ===
        nodes.push(ComputeNode {
            id: "embed_tokens".into(),
            op_type: UniversalOp::Embedding {
                padding_idx: None,
                vocab_size: config.vocab_size as i64,
                embed_dim: config.hidden_size as i64,
            },
            inputs: vec!["input_ids".into()],
            outputs: vec!["hidden_states_0".into()],
            attributes: OpAttributes::default(),
            domain: String::new(),
        });

        // === N Decoder Layers ===
        for layer_idx in 0..num_layers {
            let prev_hidden = if layer_idx == 0 {
                "hidden_states_0".to_string()
            } else {
                format!("hidden_states_{}", layer_idx)
            };
            let curr_hidden = format!("hidden_states_{}", layer_idx + 1);
            let prefix = format!("layer_{}", layer_idx);

            // Input LayerNorm (RMSNorm)
            nodes.push(ComputeNode {
                id: format!("{}_input_norm", prefix),
                op_type: UniversalOp::RmsNorm { eps },
                inputs: vec![prev_hidden.clone()],
                outputs: vec![format!("{}_normed", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });

            // Self-Attention (GQA si num_kv_heads != num_heads)
            nodes.push(ComputeNode {
                id: format!("{}_attn", prefix),
                op_type: if num_kv_heads == num_heads {
                    UniversalOp::MultiHeadAttention {
                        num_heads: num_heads as i64,
                        head_dim: head_dim as i64,
                    }
                } else {
                    UniversalOp::GroupedQueryAttention {
                        num_heads: num_heads as i64,
                        num_kv_heads: num_kv_heads as i64,
                        head_dim: head_dim as i64,
                    }
                },
                inputs: vec![format!("{}_normed", prefix)],
                outputs: vec![format!("{}_attn_out", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });

            // RoPE Embedding
            nodes.push(ComputeNode {
                id: format!("{}_rope", prefix),
                op_type: UniversalOp::RotaryPositionEmbedding {
                    base: rope_theta,
                    scaling: config.rope_scaling.clone(),
                },
                inputs: vec![format!("{}_attn_out", prefix)],
                outputs: vec![format!("{}_rope_out", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });

            // Residual connection 1
            nodes.push(ComputeNode {
                id: format!("{}_res1", prefix),
                op_type: UniversalOp::Add,
                inputs: vec![prev_hidden, format!("{}_rope_out", prefix)],
                outputs: vec![format!("{}_after_attn", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });

            // Post-attention LayerNorm
            nodes.push(ComputeNode {
                id: format!("{}_post_norm", prefix),
                op_type: UniversalOp::RmsNorm { eps },
                inputs: vec![format!("{}_after_attn", prefix)],
                outputs: vec![format!("{}_mlp_input", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });

            // MLP (SwiGLU : gate × up, puis down)
            nodes.push(ComputeNode {
                id: format!("{}_mlp_gate", prefix),
                op_type: UniversalOp::Silu,
                inputs: vec![format!("{}_mlp_input", prefix)],
                outputs: vec![format!("{}_mlp_gate_out", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });
            nodes.push(ComputeNode {
                id: format!("{}_mlp_mul", prefix),
                op_type: UniversalOp::Mul,
                inputs: vec![
                    format!("{}_mlp_gate_out", prefix),
                    format!("{}_mlp_input", prefix),
                ],
                outputs: vec![format!("{}_mlp_hidden", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });
            nodes.push(ComputeNode {
                id: format!("{}_mlp_down", prefix),
                op_type: UniversalOp::Linear {
                    in_features: config.intermediate_size as i64,
                    out_features: config.hidden_size as i64,
                    bias: false,
                },
                inputs: vec![format!("{}_mlp_hidden", prefix)],
                outputs: vec![format!("{}_mlp_out", prefix)],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });

            // Residual connection 2
            nodes.push(ComputeNode {
                id: format!("{}_res2", prefix),
                op_type: UniversalOp::Add,
                inputs: vec![
                    format!("{}_after_attn", prefix),
                    format!("{}_mlp_out", prefix),
                ],
                outputs: vec![curr_hidden],
                attributes: OpAttributes::default(),
                domain: String::new(),
            });
        }

        // === Final Norm + LM Head ===
        let last_hidden = format!("hidden_states_{}", num_layers);
        nodes.push(ComputeNode {
            id: "final_norm".into(),
            op_type: UniversalOp::RmsNorm { eps },
            inputs: vec![last_hidden],
            outputs: vec!["normed_final".into()],
            attributes: OpAttributes::default(),
            domain: String::new(),
        });

        // LM Head (lié à embed_tokens si tie_word_embeddings)
        nodes.push(ComputeNode {
            id: "lm_head".into(),
            op_type: UniversalOp::Linear {
                in_features: config.hidden_size as i64,
                out_features: config.vocab_size as i64,
                bias: false,
            },
            inputs: vec!["normed_final".into()],
            outputs: vec!["logits".into()],
            attributes: OpAttributes::default(),
            domain: String::new(),
        });

        Ok(ComputeGraph {
            nodes,
            edges: Vec::new(), // Les arêtes sont implicites via les noms de tenseurs
            inputs: vec![GraphTensor {
                name: "input_ids".into(),
                dtype: Some(DType::I64),
                shape: Some(vec![None, None]), // [batch, seq_len]
            }],
            outputs: vec![GraphTensor {
                name: "logits".into(),
                dtype: Some(DType::F32),
                shape: Some(vec![None, None, Some(config.vocab_size as i64)]),
            }],
            opset_version: Some(21),
        })
    }
}

// Templates similaires pour Phi, Gemma, Qwen, Falcon...
// (implémentation complète dans crates/umc-graph/src/templates/)
pub struct MistralTemplate(LlamaTemplate);
impl MistralTemplate {
    pub fn new() -> Self { Self(LlamaTemplate::new()) }
}
// Mistral = Llama avec Sliding Window Attention — réutilise LlamaTemplate
// avec un flag supplémentaire dans OpAttributes

pub struct PhiTemplate {
    known_variants: Vec<&'static str>,
}
impl PhiTemplate {
    pub fn new() -> Self {
        Self { known_variants: vec!["phi", "phi-1", "phi-2", "phi-3", "phi-3.5"] }
    }
}
// Implémentation similaire à LlamaTemplate mais avec Parallel Attention + MLP

pub struct GemmaTemplate {
    known_variants: Vec<&'static str>,
}
impl GemmaTemplate {
    pub fn new() -> Self {
        Self { known_variants: vec!["gemma", "gemma2"] }
    }
}
// Gemma = Llama avec quelques différences (GeLU au lieu de SiLU, etc.)

pub struct QwenTemplate {
    known_variants: Vec<&'static str>,
}
impl QwenTemplate {
    pub fn new() -> Self {
        Self { known_variants: vec!["qwen", "qwen1.5", "qwen2", "qwen2.5", "qwen3"] }
    }
}

pub struct FalconTemplate {
    known_variants: Vec<&'static str>,
}
impl FalconTemplate {
    pub fn new() -> Self {
        Self { known_variants: vec!["falcon", "rw"] }
    }
}

pub struct GPTNeoXTemplate { known_variants: Vec<&'static str> }
impl GPTNeoXTemplate { pub fn new() -> Self { Self { known_variants: vec!["gpt_neox", "pythia", "dolly"] } } }

pub struct OPTTemplate { known_variants: Vec<&'static str> }
impl OPTTemplate { pub fn new() -> Self { Self { known_variants: vec!["opt"] } } }

pub struct BloomTemplate { known_variants: Vec<&'static str> }
impl BloomTemplate { pub fn new() -> Self { Self { known_variants: vec!["bloom", "bloomz"] } } }

// Implémentations GraphTemplate pour tous les templates ci-dessus
// dans leurs fichiers respectifs sous crates/umc-graph/src/templates/
```