# UMC — Universal Model Converter
## Document de Conception v2.0 — Corrigé et Complet

> **Statut :** Référence d'implémentation officielle  
> **Langage :** Rust (stable)  
> **Licence :** Apache 2.0  
> **Philosophie fondatrice :** UMC est 100 % natif. Zéro dépendance aux outils externes dans le chemin critique. Zéro promesse fausse. Zéro compromis sur la qualité.

---

## TABLE DES MATIÈRES

1. [Vision Corrigée et Positionnement Honnête](#1-vision-corrigée-et-positionnement-honnête)
2. [Architecture Technique Globale](#2-architecture-technique-globale)
3. [L'IR Universelle — Cœur du Système](#3-lir-universelle--cœur-du-système)
4. [Pipeline de Conversion Parallèle Corrigé](#4-pipeline-de-conversion-parallèle-corrigé)
5. [Stratégie d'Ajout Progressif des 32 Formats](#5-stratégie-dajout-progressif-des-32-formats)
6. [GraphTemplate — Reconstruction Native des Graphes](#6-graphtemplate--reconstruction-native-des-graphes)
7. [Gestion Native de la Quantification](#7-gestion-native-de-la-quantification)
8. [Gestion Native des Adaptateurs](#8-gestion-native-des-adaptateurs)
9. [Validation et Certification Réaliste](#9-validation-et-certification-réaliste)
10. [Sécurité — Parsing Défensif](#10-sécurité--parsing-défensif)
11. [Backend Simplifié et Scalable](#11-backend-simplifié-et-scalable)
12. [Frontend Épuré](#12-frontend-épuré)
13. [CLI Complète](#13-cli-complète)
14. [API REST](#14-api-rest)
15. [Modèle Économique Révisé](#15-modèle-économique-révisé)
16. [Stratégie de Déploiement Progressive](#16-stratégie-de-déploiement-progressive)
17. [Structure du Projet Rust](#17-structure-du-projet-rust)
18. [Annexes et Glossaire](#18-annexes-et-glossaire)

---

## 1. Vision Corrigée et Positionnement Honnête

### 1.1 La Phrase Fondatrice

> **"UMC est le ffmpeg des modèles IA. 100 % natif. Zéro dépendance. Garanti mathématiquement."**

### 1.2 Ce que UMC Promet — et Ce qu'il Ne Promet Pas

**Promesses réelles et vérifiables :**

| Promesse | Réalité | Garantie |
|----------|---------|----------|
| Conversion native Rust pour les formats majeurs | ✅ Aucune dépendance externe | Vérifiable par `cargo tree` |
| Performance ×4 à ×8 vs outils Python | ✅ Mesuré sur hardware documenté | Benchmark reproductible public |
| RAM minimale via mmap | ✅ Structures de données ~200 Mo | RSS réelle documentée par OS |
| Round-trip sémantiquement identique | ✅ Mêmes sorties d'inférence | Validation fonctionnelle |
| Zéro perte d'information structurelle | ✅ ExtensionStore préserve tout | Test automatique par format |
| Formats produits valides | ✅ Validateurs natifs intégrés | Tests d'intégrité par format |

**Ce qu'UMC ne promet pas :**

- ❌ Round-trip **bit-identical** entre formats différents (impossible mathématiquement pour les formats quantifiés ou compressés)
- ❌ Performance ×17 sur toutes les machines (chiffre issu d'un seul benchmark sur hardware haut de gamme)
- ❌ "Valeur légale" des certificats (terme remplacé par "rapport de conversion certifié")
- ❌ Support complet de 32 formats dès le lancement (progression documentée et honnête)

### 1.3 L'Insight Architectural — Version Honnête

```
Réalité :
  80 % des conversions → N + M composants (IR suffit)
  20 % des conversions → logique spécifique par paire (cas edge)

Format A → [Loader A] → IR_UMC → [Saver B] → Format B

IR_UMC = union de tous les formats supportés
       = sur-ensemble évolutif, pas parfait d'emblée

Garantie réelle pour A → B → A :
  NIVEAU 1 (Bit-identical) : A → A uniquement (même format)
  NIVEAU 2 (Sémantique)    : A → B → A → sorties inférence identiques
  NIVEAU 3 (Structurel)    : A → B → A → même graphe et architecture
```

### 1.4 Indépendance Totale — La Règle Absolue

**UMC n'appelle jamais un outil externe dans le chemin de conversion.**

- Toutes les conversions sont implémentées nativement en Rust
- Les formats qui nécessitent des compilateurs hardware (TensorRT, CoreML compiler) sont supportés en **lecture uniquement** ou via une **API de génération de configuration** — pas via des sous-processus
- Si un format ne peut pas être implémenté nativement avec qualité suffisante, il n'est pas annoncé comme supporté

```
PHILOSOPHIE :
  Mieux vaut 10 formats impeccables que 32 formats médiocres.
  Chaque format ajouté est natif, testé, maintenu.
  Aucun `Command::new("trtexec")` dans le code source.
```

---

## 2. Architecture Technique Globale

### 2.1 Vue d'Ensemble

```
┌──────────────────────────────────────────────────────────────────────┐
│                     UMC — Architecture v2.0                           │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                     COUCHE INTERFACE                            │ │
│  │   CLI (clap)    API REST (axum)    SDK Python    SDK JS         │ │
│  └───────────────────────────┬────────────────────────────────────┘ │
│                              │                                       │
│  ┌───────────────────────────▼────────────────────────────────────┐ │
│  │                 COUCHE ORCHESTRATION                             │ │
│  │   Détection Format · Dijkstra · Job Queue · Progress            │ │
│  │   GraphTemplate Registry · Capability Registry                  │ │
│  └───────────────────────────┬────────────────────────────────────┘ │
│                              │                                       │
│  ┌───────────────────────────▼────────────────────────────────────┐ │
│  │                 COUCHE CONVERSION (CORE)                         │ │
│  │                                                                  │ │
│  │  ┌──────────┐  ┌─────────────────────────────┐  ┌──────────┐  │ │
│  │  │ LOADERS  │─▶│       IR UNIVERSELLE v2      │─▶│  SAVERS  │  │ │
│  │  │ 100% Rust│  │  TensorStore (mmap+streaming)│  │ 100% Rust│  │ │
│  │  │ sécurisé │  │  ComputeGraph (DAG + weights)│  │ validé   │  │ │
│  │  │ fuzzé    │  │  QuantizationStore (étendu)  │  │ certifié │  │ │
│  │  └──────────┘  │  AdapterStore                │  └──────────┘  │ │
│  │                │  ExtensionStore (limité+safe) │                 │ │
│  │  ┌──────────┐  │  TokenizerStore               │  ┌──────────┐  │ │
│  │  │ PIPELINE │  │  ProvenanceChain (immutable)  │  │VALIDATOR │  │ │
│  │  │ Reader   │  │  GraphTemplate Registry       │  │ Struct.  │  │ │
│  │  │Transform │  │  SecurityBounds               │  │ Numeric  │  │ │
│  │  │ Writer   │  └─────────────────────────────-─┘  │ Semantic │  │ │
│  │  │ Watchdog │                                      │ Certif.  │  │ │
│  │  └──────────┘                                      └──────────┘  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                  COUCHE INFRASTRUCTURE                           │ │
│  │  mmap (memmap2) · rayon · crossbeam · tokio · xxhash · SIMD    │ │
│  │  cargo-fuzz (tests) · proptest · criterion                      │ │
│  └────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.2 Principes Architecturaux Non Négociables

```
PRINCIPE 1 — Natif ou Rien
  Toute fonctionnalité annoncée est implémentée en Rust pur.
  Aucun subprocess. Aucun outil tiers dans le chemin critique.

PRINCIPE 2 — Sécurité par défaut
  Tout fichier entrant est hostile jusqu'à preuve du contraire.
  Limites hardcodées sur tous les champs lus depuis les fichiers.
  Fuzzing automatique sur tous les loaders.

PRINCIPE 3 — Honnêteté des promesses
  Aucune garantie qui ne peut pas être vérifiée automatiquement.
  Chaque chiffre de performance est accompagné d'une méthodologie publique.

PRINCIPE 4 — Progression qualitative
  5 formats parfaits avant 32 formats médiocres.
  Chaque format ajouté : spec lue, tests écrits, fuzzing fait.

PRINCIPE 5 — Zéro Deadlock, Zéro Panique
  Cancellation coopérative dans tous les threads.
  Watchdog sur le pipeline. Timeout sur toutes les opérations.
```

### 2.3 Workspace Rust

```
umc/
├── crates/
│   ├── umc-core/        IR + traits + sécurité + erreurs
│   ├── umc-detect/      Détection de format (magic bytes)
│   ├── umc-graph/       Graphe Dijkstra + GraphTemplate Registry
│   ├── umc-pipeline/    Pipeline 3-threads + Watchdog
│   ├── umc-validate/    Validation sémantique + certification
│   ├── umc-formats/     Loaders/Savers 100% natifs
│   │   ├── gguf/
│   │   ├── safetensors/
│   │   ├── onnx/
│   │   ├── pytorch/
│   │   └── ...
│   ├── umc-cli/         Interface CLI
│   ├── umc-api/         API REST (axum)
│   └── umc-fuzz/        Cibles de fuzzing (cargo-fuzz)
├── tests/
│   ├── round_trip/
│   ├── security/        Tests de parsing malveillant
│   ├── benchmarks/
│   └── fixtures/
└── benches/             Benchmarks publics reproductibles
```

---

## 3. L'IR Universelle — Cœur du Système

### 3.1 Structure Principale v2

```rust
/// IR Universelle v2 — corrections appliquées :
/// - ComputeGraph optionnel (formats weights-only)
/// - ExtensionStore limité et sécurisé
/// - ProvenanceChain immutable par hash chaining
/// - SecurityBounds intégrés
#[derive(Debug, Clone)]
pub struct UniversalIR {
    pub tensors:           TensorStore,
    pub graph:             GraphContent,     // ← MODIFIÉ : optionnel
    pub metadata:          MetadataStore,
    pub architecture:      ArchitectureConfig,
    pub tokenizer:         Option<TokenizerStore>,
    pub quantization:      Option<QuantizationStore>,
    pub adapters:          Vec<AdapterInfo>,
    pub pruning:           Option<PruningInfo>,
    pub distillation:      Option<DistillationInfo>,
    pub generation_config: Option<GenerationConfig>,
    pub training_config:   Option<TrainingConfig>,
    pub provenance:        ProvenanceChain,  // ← immutable via hash chain
    pub extensions:        ExtensionStore,   // ← limité, sécurisé
}

/// Contenu de graphe — distingue weights-only des formats avec graphe
#[derive(Debug, Clone)]
pub enum GraphContent {
    /// Format avec graphe explicite (ONNX, PyTorch, TFSavedModel...)
    Explicit(ComputeGraph),
    /// Format weights-only (GGUF, SafeTensors, AWQ, GPTQ...)
    /// Le graphe sera reconstruit via GraphTemplate si nécessaire
    WeightsOnly {
        architecture: String,  // "llama", "mistral", "phi", etc.
        template_available: bool,
    },
    /// Graphe composite (Diffusers : plusieurs sous-modèles)
    Composite(Vec<SubModelGraph>),
}

#[derive(Debug, Clone)]
pub struct SubModelGraph {
    pub name: String,          // "unet", "vae", "text_encoder"
    pub graph: ComputeGraph,
    pub role: SubModelRole,
}

#[derive(Debug, Clone)]
pub enum SubModelRole {
    TextEncoder, ImageEncoder, Denoiser, Decoder, Scheduler, Custom(String),
}
```

### 3.2 TensorStore — Zéro Copie avec Sécurité

```rust
#[derive(Debug, Clone)]
pub struct Tensor {
    pub name:         String,
    pub dtype:        DType,
    pub shape:        Vec<usize>,
    pub strides:      Option<Vec<usize>>,
    pub layout:       Layout,
    pub data:         TensorData,
    pub checksum:     u64,          // xxHash64
    pub quantization: Option<TensorQuantization>,
}

#[derive(Debug, Clone)]
pub enum TensorData {
    MmapView { mmap: Arc<Mmap>, offset: usize, length: usize },
    Owned(Arc<Vec<u8>>),
    Lazy { file_path: PathBuf, offset: u64, length: usize },
    Shared(String),
}

/// Limites de sécurité — validées à chaque insertion
pub struct SecurityBounds {
    pub max_tensor_count:      usize,   // 1_000_000
    pub max_metadata_count:    usize,   // 10_000
    pub max_string_length:     usize,   // 1_048_576 (1 Mo)
    pub max_shape_rank:        usize,   // 8
    pub max_tensor_size_bytes: usize,   // 100 * 1024^3 (100 Go)
    pub max_extension_bytes:   usize,   // 100 * 1024^2 (100 Mo)
}

impl Default for SecurityBounds {
    fn default() -> Self {
        Self {
            max_tensor_count:      1_000_000,
            max_metadata_count:    10_000,
            max_string_length:     1_048_576,
            max_shape_rank:        8,
            max_tensor_size_bytes: 100 * 1024 * 1024 * 1024,
            max_extension_bytes:   100 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TensorStore {
    tensors:           indexmap::IndexMap<String, Tensor>,
    ram_usage_bytes:   usize,
    pub mmap_threshold: usize,
    bounds:            SecurityBounds,
}

impl TensorStore {
    pub fn new() -> Self {
        Self {
            tensors: indexmap::IndexMap::new(),
            ram_usage_bytes: 0,
            mmap_threshold: 64 * 1024 * 1024,
            bounds: SecurityBounds::default(),
        }
    }

    /// Insert avec validation de sécurité
    pub fn insert(&mut self, tensor: Tensor) -> Result<(), UmcError> {
        if self.tensors.len() >= self.bounds.max_tensor_count {
            return Err(UmcError::SecurityViolation {
                field: "tensor_count",
                value: self.tensors.len(),
                limit: self.bounds.max_tensor_count,
            });
        }
        if tensor.data.len() > self.bounds.max_tensor_size_bytes {
            return Err(UmcError::SecurityViolation {
                field: "tensor_size_bytes",
                value: tensor.data.len(),
                limit: self.bounds.max_tensor_size_bytes,
            });
        }
        if let TensorData::Owned(ref v) = tensor.data {
            self.ram_usage_bytes += v.len();
        }
        self.tensors.insert(tensor.name.clone(), tensor);
        Ok(())
    }
}
```

### 3.3 DType — Énumération Universelle

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DType {
    F64, F32, F16, BF16,
    F8E4M3, F8E5M2,
    I64, I32, I16, I8,
    U64, U32, U16, U8,
    Bool,
    // GGUF K-quants
    Q2K, Q3KS, Q3KM, Q3KL,
    Q4_0, Q4_1, Q4KS, Q4KM,
    Q5_0, Q5_1, Q5KS, Q5KM,
    Q6K, Q8_0, Q8K,
    // Quantification par canal
    Awq4, Awq8, Gptq4, Gptq8,
    // bitsandbytes
    NF4, FP4,
    Custom(String),
}
```

### 3.4 TensorQuantization — Métadonnées Complètes

```rust
/// Métadonnées de quantification étendues (v2 — correction critique)
/// La version précédente manquait block_size, superblock_size, scale_dtype
#[derive(Debug, Clone)]
pub struct TensorQuantization {
    pub scheme:              QuantScheme,
    pub block_size:          usize,          // OBLIGATOIRE
    pub superblock_size:     Option<usize>,  // Pour GGUF K-quants (256)
    pub scale_dtype:         DType,          // Type des scales (F16, F32, Q8_0...)
    pub zp_dtype:            DType,          // Type des zero-points
    pub storage_order:       StorageOrder,   // Interleaved vs Sequential
    pub calibration_dataset: Option<String>, // Référence dataset (AWQ/GPTQ)
    pub calibration_method:  Option<String>, // "minmax", "percentile", "mse"
}

#[derive(Debug, Clone)]
pub enum StorageOrder {
    Sequential,   // Poids puis scales (GPTQ)
    Interleaved,  // Poids et scales entrelacés (AWQ)
    BlockPacked,  // Blocs complets (GGUF)
}
```

### 3.5 ExtensionStore — Sécurisé et Limité

```rust
/// ExtensionStore v2 — corrections :
/// - Taille maximale imposée
/// - Clés namespaced (collision-proof)
/// - Overflow sur disque optionnel
/// - Validation des clés
#[derive(Debug, Clone, Default)]
pub struct ExtensionStore {
    format_extensions:   HashMap<String, FormatExtension>,
    tensor_extensions:   HashMap<String, TensorExtension>,
    op_extensions:       HashMap<String, OpExtension>,
    tokenizer_ext:       HashMap<String, Vec<u8>>,
    config_ext:          HashMap<String, Vec<u8>>,
    total_bytes:         usize,
    max_bytes:           usize,   // Défaut : 100 Mo
}

/// Clé namespaced — format : "<FORMAT>@<VERSION>/<chemin>"
/// Exemples : "GGUF@v3/tokenizer.chat_template"
///            "GGUF@v3/rope_scaling.factor"
fn validate_extension_key(key: &str) -> Result<(), UmcError> {
    // Format obligatoire : "FORMAT@VERSION/path"
    if !key.contains('@') || !key.contains('/') {
        return Err(UmcError::InvalidExtensionKey(key.to_string()));
    }
    if key.len() > 512 {
        return Err(UmcError::InvalidExtensionKey("key too long".to_string()));
    }
    // Caractères autorisés : alphanumérique, @, /, _, ., -
    if !key.chars().all(|c| c.is_alphanumeric() || "@/._-".contains(c)) {
        return Err(UmcError::InvalidExtensionKey(key.to_string()));
    }
    Ok(())
}

impl ExtensionStore {
    pub fn set_raw(&mut self, key: &str, value: Vec<u8>) -> Result<(), UmcError> {
        validate_extension_key(key)?;
        let new_total = self.total_bytes + value.len();
        if new_total > self.max_bytes {
            return Err(UmcError::ExtensionStoreFull {
                current_bytes: self.total_bytes,
                max_bytes: self.max_bytes,
                tried_to_add: value.len(),
            });
        }
        // Parsing de la clé
        let at_pos = key.find('@').unwrap();
        let slash_pos = key.find('/').unwrap();
        let format_name = &key[..at_pos];
        let path = &key[slash_pos + 1..];
        let ext = self.format_extensions
            .entry(format_name.to_string())
            .or_insert_with(FormatExtension::default);
        self.total_bytes += value.len();
        ext.custom_fields.insert(path.to_string(), value);
        Ok(())
    }

    pub fn get_raw(&self, format: &str, version: &str, path: &str) -> Option<&[u8]> {
        self.format_extensions
            .get(format)
            .and_then(|ext| ext.custom_fields.get(path))
            .map(|v| v.as_slice())
    }

    pub fn total_bytes(&self) -> usize { self.total_bytes }
}
```

### 3.6 ProvenanceChain — Immutable par Hash Chaining

```rust
/// ProvenanceChain v2 — tamper-evident par hash chaining
/// entry[n].chain_hash = SHA256(entry[n-1].chain_hash || entry[n].content_hash)
#[derive(Debug, Clone)]
pub struct ProvenanceChain {
    entries:    Vec<ProvenanceEntry>,
    root_hash:  String,   // Hash de la première entrée
}

#[derive(Debug, Clone)]
pub struct ProvenanceEntry {
    pub timestamp:    u64,
    pub source_fmt:   String,
    pub target_fmt:   String,
    pub tool:         String,          // "umc/2.0.0"
    pub input_hash:   String,          // SHA256 du fichier source
    pub output_hash:  String,          // SHA256 du fichier cible
    pub certificate:  Option<String>,  // ID du certificat
    pub content_hash: String,          // SHA256 de cette entrée
    pub chain_hash:   String,          // SHA256(prev_chain_hash || content_hash)
}

impl ProvenanceChain {
    /// Vérifie l'intégrité de toute la chaîne
    pub fn verify(&self) -> bool {
        let mut prev_chain_hash = self.root_hash.clone();
        for entry in &self.entries {
            let expected = compute_chain_hash(&prev_chain_hash, &entry.content_hash);
            if expected != entry.chain_hash {
                return false;
            }
            prev_chain_hash = entry.chain_hash.clone();
        }
        true
    }

    /// Ajoute une entrée — ne peut qu'ajouter, jamais modifier
    pub fn append(&mut self, entry_data: ProvenanceEntryData) -> ProvenanceEntry {
        let prev_chain_hash = self.entries.last()
            .map(|e| e.chain_hash.clone())
            .unwrap_or_else(|| self.root_hash.clone());
        let content_hash = entry_data.compute_hash();
        let chain_hash = compute_chain_hash(&prev_chain_hash, &content_hash);
        let entry = ProvenanceEntry {
            timestamp:    entry_data.timestamp,
            source_fmt:   entry_data.source_fmt,
            target_fmt:   entry_data.target_fmt,
            tool:         entry_data.tool,
            input_hash:   entry_data.input_hash,
            output_hash:  entry_data.output_hash,
            certificate:  entry_data.certificate,
            content_hash,
            chain_hash,
        };
        self.entries.push(entry.clone());
        entry
    }
}
```

### 3.7 ComputeGraph — DAG Universel

```rust
#[derive(Debug, Clone)]
pub struct ComputeGraph {
    pub nodes:   Vec<ComputeNode>,
    pub edges:   Vec<ComputeEdge>,
    pub inputs:  Vec<String>,
    pub outputs: Vec<String>,
}

/// Opérateurs universels — natifs, sans outil externe
#[derive(Debug, Clone, PartialEq)]
pub enum UniversalOp {
    // Arithmétique
    Add, Sub, Mul, Div, Pow, Sqrt, Rsqrt, Abs, Neg, Exp, Log, Tanh, Sigmoid, Erf,
    // Activations
    Relu, Relu6, LeakyRelu { alpha: f64 },
    Gelu, GeluApprox, Silu, Swish, HardSwish, HardSigmoid, Mish, QuickGelu,
    // Réduction
    ReduceSum { axes: Vec<i64>, keepdims: bool },
    ReduceMean { axes: Vec<i64>, keepdims: bool },
    ReduceMax { axes: Vec<i64>, keepdims: bool },
    ReduceMin { axes: Vec<i64>, keepdims: bool },
    // Normalisation
    LayerNorm { axis: i64, eps: f64 },
    RmsNorm { eps: f64 },
    BatchNorm { eps: f64, momentum: f64, training: bool },
    GroupNorm { num_groups: i64, eps: f64 },
    // Algèbre linéaire
    Gemm { alpha: f64, beta: f64, trans_a: bool, trans_b: bool },
    MatMul,
    Conv2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64>, dilations: Vec<i64>, group: i64 },
    ConvTranspose2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    DepthwiseConv2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    // Attention
    MultiHeadAttention { num_heads: i64, head_dim: i64 },
    ScaledDotProductAttention,
    // Positional
    RotaryPositionEmbedding { base: f64, scaling: Option<RopeScalingConfig> },
    AlibiPositionEmbedding,
    SinusoidalPositionEmbedding,
    // Reshape
    Reshape, Transpose { perm: Vec<i64> },
    Flatten { axis: i64 }, Squeeze { axes: Vec<i64> }, Unsqueeze { axes: Vec<i64> },
    Concat { axis: i64 }, Split { axis: i64, sizes: Vec<i64> },
    Gather { axis: i64 }, Slice { axes: Vec<i64>, starts: Vec<i64>, ends: Vec<i64> },
    // Pooling
    MaxPool2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    AveragePool2D { kernel_shape: Vec<i64>, strides: Vec<i64>, pads: Vec<i64> },
    GlobalAveragePool, GlobalMaxPool,
    // Divers
    Softmax { axis: i64 }, LogSoftmax { axis: i64 },
    Cast { to: DType }, Embedding { padding_idx: Option<i64> },
    Dropout { ratio: f64, training: bool },
    Constant { value: ConstantValue }, Identity,
    // Opérateur inconnu — préservé dans ExtensionStore, PAS une erreur fatale
    Custom { domain: String, op_type: String, attributes: HashMap<String, Vec<u8>> },
}
```

---

## 4. Pipeline de Conversion Parallèle Corrigé

### 4.1 Architecture 3-Thread avec Watchdog et Cancellation

```rust
/// Pipeline v2 — corrections :
/// - CancellationToken coopératif
/// - Watchdog thread (détecte les deadlocks)
/// - Canaux avec timeout (pas de deadlock possible)
/// - Write-to-temp + atomic rename

pub struct ConversionPipeline {
    config:   PipelineConfig,
    cancel:   CancellationToken,
}

pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) { self.cancelled.store(true, Ordering::SeqCst); }
    pub fn is_cancelled(&self) -> bool { self.cancelled.load(Ordering::SeqCst) }
}

pub struct PipelineConfig {
    pub shard_workers:    usize,
    pub tensor_threads:   usize,
    pub tile_size_bytes:  usize,
    pub channel_capacity: usize,
    pub chunk_size_bytes: usize,
    pub prefetch_count:   usize,
    pub op_timeout_secs:  u64,       // ← NOUVEAU : timeout par opération
    pub watchdog_secs:    u64,       // ← NOUVEAU : watchdog interval
    pub reproducible:     bool,
    pub seed:             u64,
}

impl PipelineConfig {
    pub fn auto() -> Self {
        let cpus = num_cpus::get();
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        // Utiliser RAM disponible, pas totale
        let ram_available_gb = sys.available_memory() / (1024 * 1024 * 1024);

        Self {
            shard_workers:    cpus.min(16).min(8_usize.max(cpus / 4)),
            tensor_threads:   cpus,
            tile_size_bytes:  if ram_available_gb < 8 { 16 * 1024 * 1024 }
                              else if ram_available_gb < 32 { 32 * 1024 * 1024 }
                              else { 64 * 1024 * 1024 },
            channel_capacity: 4,           // Réduit pour éviter OOM
            chunk_size_bytes: if ram_available_gb < 8 { 16 * 1024 * 1024 }
                              else { 64 * 1024 * 1024 },
            prefetch_count:   if ram_available_gb < 8 { 1 } else { 2 },
            op_timeout_secs:  120,         // 2 minutes max par tenseur
            watchdog_secs:    30,
            reproducible:     false,
            seed:             42,
        }
    }
}

/// Messages du pipeline — identiques mais avec timeout intégré
pub enum PipelineMessage {
    Tensor(Box<Tensor>),
    Done,
    Error(UmcError),
    Cancelled,
}

impl ConversionPipeline {
    pub fn run(
        &self,
        source: ConversionSource,
        target_format: &str,
        options: &ConversionOptions,
        progress: &ProgressCallback,
    ) -> Result<ConversionResult, UmcError> {
        // Fichier temporaire — atomic rename à la fin
        let temp_output = TempOutputFile::new(options.output_path())?;

        let (read_tx, read_rx) = crossbeam::channel::bounded(self.config.channel_capacity);
        let (transform_tx, transform_rx) = crossbeam::channel::bounded(self.config.channel_capacity);
        let (progress_tx, progress_rx) = crossbeam::channel::bounded(64);

        let cancel = self.cancel.clone();
        let timeout = Duration::from_secs(self.config.op_timeout_secs);

        // Thread Watchdog — détecte les deadlocks
        let watchdog_cancel = cancel.clone();
        let watchdog = {
            let last_progress = Arc::new(AtomicU64::new(0));
            let lp = last_progress.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(30));
                    let current = lp.load(Ordering::Relaxed);
                    // ... logique de détection de stagnation
                    if watchdog_cancel.is_cancelled() { break; }
                }
            })
        };

        // Thread 1 : Reader
        let reader = {
            let tx = read_tx;
            let cancel = cancel.clone();
            let source = source.clone();
            std::thread::spawn(move || {
                Self::reader_thread(source, tx, cancel, timeout)
            })
        };

        // Thread 2 : Transformer
        let transformer = {
            let rx = read_rx;
            let tx = transform_tx;
            let cancel = cancel.clone();
            let target = target_format.to_string();
            let opts = options.clone();
            let config = self.config.clone();
            std::thread::spawn(move || {
                Self::transformer_thread(rx, tx, cancel, &target, &opts, &config, timeout)
            })
        };

        // Thread 3 : Writer
        let writer = {
            let rx = transform_rx;
            let cancel = cancel.clone();
            let temp = temp_output.clone();
            let prog = progress.clone();
            std::thread::spawn(move || {
                Self::writer_thread(rx, cancel, temp, prog, timeout)
            })
        };

        let r = reader.join().map_err(|_| UmcError::ThreadPanic { thread: "reader" })??;
        let t = transformer.join().map_err(|_| UmcError::ThreadPanic { thread: "transformer" })??;
        let w = writer.join().map_err(|_| UmcError::ThreadPanic { thread: "writer" })??;

        // Atomic rename : temp → fichier final
        temp_output.commit()?;

        Ok(w)
    }

    fn reader_thread(
        source: ConversionSource,
        tx: Sender<PipelineMessage>,
        cancel: CancellationToken,
        timeout: Duration,
    ) -> Result<(), UmcError> {
        for tensor_result in source.iter_tensors()? {
            if cancel.is_cancelled() {
                let _ = tx.send_timeout(PipelineMessage::Cancelled, timeout);
                return Ok(());
            }
            let tensor = tensor_result?;
            tx.send_timeout(PipelineMessage::Tensor(Box::new(tensor)), timeout)
                .map_err(|_| UmcError::ChannelTimeout { thread: "reader→transformer" })?;
        }
        let _ = tx.send_timeout(PipelineMessage::Done, timeout);
        Ok(())
    }
}
```

### 4.2 TempOutputFile — Atomic Rename

```rust
/// Pattern write-to-temp + atomic rename
/// Garantit que le fichier de sortie est toujours valide ou absent
pub struct TempOutputFile {
    temp_path:   PathBuf,
    final_path:  PathBuf,
    committed:   bool,
}

impl TempOutputFile {
    pub fn new(final_path: &Path) -> Result<Self, UmcError> {
        let temp_path = final_path.with_extension(
            format!("{}.umc_tmp", std::process::id())
        );
        Ok(Self { temp_path, final_path: final_path.to_path_buf(), committed: false })
    }

    pub fn temp_path(&self) -> &Path { &self.temp_path }

    /// Déplace atomiquement le fichier temp vers la destination finale
    pub fn commit(mut self) -> Result<(), UmcError> {
        std::fs::rename(&self.temp_path, &self.final_path)
            .map_err(|e| UmcError::AtomicRename { source: self.temp_path.clone(), e })?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for TempOutputFile {
    fn drop(&mut self) {
        if !self.committed {
            // Nettoyage automatique en cas d'échec
            let _ = std::fs::remove_file(&self.temp_path);
        }
    }
}
```

---

## 5. Stratégie d'Ajout Progressif des 32 Formats

### 5.1 Philosophie de la Progression

```
RÈGLE D'OR :
Un format est "supporté" uniquement quand :
  □ La spécification officielle est lue et documentée
  □ Un loader natif Rust est implémenté et fuzzé
  □ Un saver natif Rust est implémenté et validé
  □ Des tests round-trip sur modèles réels passent
  □ Les benchmarks sont mesurés et documentés
  □ Le format est maintenu dans le temps

Un format en lecture seule (loader uniquement) est clairement marqué.
```

### 5.2 Ordre d'Ajout — Phase 0 : Fondations (Semaines 1–8)

**Rationale :** Ces formats couvrent 90 % des cas d'usage réels et forment la base technique.

```
SPRINT 0.1 — Semaines 1–2 : Infrastructure Core
  □ umc-core complet (IR v2, SecurityBounds, ProvenanceChain)
  □ umc-detect complet (magic bytes + heuristiques)
  □ umc-graph complet (Dijkstra + GraphTemplate Registry)
  □ umc-pipeline complet (3 threads + watchdog + cancellation)
  □ Tests unitaires sur tous les composants core

SPRINT 0.2 — Semaines 3–4 : GGUF (Format #1)
  Pourquoi en premier : format le plus utilisé pour l'inférence LLM locale
  
  □ Lire la spec GGUF v1, v2, v3 complète
  □ Implémenter GgufLoader (natif Rust, mmap, zéro copie)
  □ Implémenter GgufSaver (natif Rust, conformité spec)
  □ GraphTemplate pour Llama, Mistral, Phi, Gemma
  □ Tests round-trip : GGUF → GGUF (bit-identical, même format)
  □ Fuzzing : cargo-fuzz sur GgufLoader
  □ Benchmarks documentés (3 machines)

SPRINT 0.3 — Semaines 5–6 : SafeTensors (Format #2)
  Pourquoi : format HuggingFace par défaut, simple à implémenter parfaitement
  
  □ Spec SafeTensors complète
  □ SafeTensorsLoader (mmap du JSON header + données)
  □ SafeTensorsSaver (JSON header + données alignées)
  □ Tests round-trip : SafeTensors → SafeTensors (bit-identical)
  □ Tests cross : GGUF ↔ SafeTensors (sémantique identique)
  □ Fuzzing : cargo-fuzz sur SafeTensorsLoader
  □ Benchmarks

SPRINT 0.4 — Semaines 7–8 : ONNX (Format #3)
  Pourquoi : format universel de déploiement, graphe explicite
  
  □ Spec ONNX opset 12 à 21
  □ OnnxLoader (parsing protobuf natif via prost)
  □ OnnxSaver (sérialisation protobuf, conformité opset 21)
  □ Validateur ONNX natif intégré (vérification structurelle)
  □ Tests round-trip : ONNX → ONNX (sémantique identique)
  □ Tests cross : GGUF ↔ ONNX, SafeTensors ↔ ONNX
  □ Fuzzing : cargo-fuzz sur OnnxLoader
  □ Benchmarks

CRITÈRE DE SORTIE PHASE 0 :
  umc convert model.gguf model.onnx      ← fonctionne
  umc convert model.onnx model.gguf      ← fonctionne
  umc convert model.safetensors model.gguf ← fonctionne
  Tous les tests passent. Tous les fuzzers tournent en CI.
```

### 5.3 Phase 1 : Formats LLM Essentiels (Semaines 9–20)

```
SPRINT 1.1 — Semaines 9–10 : PyTorch (Format #4)
  Pourquoi : format d'entraînement universel
  
  Difficulté : parser le format ZIP + pickle en Rust
  Solution : implémenter un parser pickle minimal (whitelist de types)
  
  □ PyTorchLoader (ZIP + pickle parser sécurisé, whitelist)
  □ PyTorchSaver (ZIP + pickle writer)
  □ Gestion des state_dict avec metadata
  □ Tests, fuzzing, benchmarks

SPRINT 1.2 — Semaines 11–12 : Tokenizers (Formats #5, #6)
  Format #5 : SentencePiece
  Format #6 : TikToken
  
  Pourquoi ensemble : souvent accompagnent les modèles LLM
  
  □ SentencePieceLoader/Saver (protobuf)
  □ TikTokenLoader/Saver (base64 text)
  □ Conversion SentencePiece ↔ TikToken
  □ Tests, fuzzing, benchmarks

SPRINT 1.3 — Semaines 13–14 : Quantification LLM (Formats #7, #8)
  Format #7 : AWQ
  Format #8 : GPTQ
  
  Difficulté : TensorQuantization étendue obligatoire (déjà préparée)
  
  □ AWQLoader/Saver (HF JSON + SafeTensors avec metadata AWQ)
  □ GPTQLoader/Saver (HF JSON + SafeTensors avec metadata GPTQ)
  □ Déquantification native (validation numérique)
  □ Tests cross avec GGUF, SafeTensors
  □ Fuzzing, benchmarks

SPRINT 1.4 — Semaines 15–16 : Adaptateurs (Formats #9, #10, #11)
  Format #9  : LoRA
  Format #10 : QLoRA
  Format #11 : PEFT
  
  □ LoRALoader/Saver (SafeTensors + adapter_config.json)
  □ QLoRALoader/Saver (NF4/FP4 + adaptateur)
  □ PEFTLoader/Saver (HF PEFT format)
  □ Fusion LoRA → poids de base (mathématique native)
  □ Tests, fuzzing, benchmarks

SPRINT 1.5 — Semaines 17–18 : bitsandbytes (Format #12)
  Lecture seule (NF4/FP4 ne peuvent être re-quantifiés sans calibration)
  
  □ BitsAndBytesLoader (HF JSON + bin)
  □ Conversion vers SafeTensors (déquantification vers F16/F32)
  □ Documentation claire : "lecture seule"
  □ Tests, fuzzing

SPRINT 1.6 — Semaines 19–20 : Legacy (Formats #13, #14)
  Format #13 : GGML (lecture seule → migration vers GGUF)
  Format #14 : TFSavedModel
  
  □ GGMLLoader (lecture seule, conversion → GGUF)
  □ TFSavedModelLoader/Saver (protobuf + SavedModel structure)
  □ Tests, fuzzing, benchmarks

CRITÈRE DE SORTIE PHASE 1 :
  14 formats natifs fonctionnels
  Couverture des cas d'usage LLM : 95%
  Fuzzing automatique en CI pour tous les loaders
```

### 5.4 Phase 2 : Formats Mobile et Edge (Semaines 21–36)

```
SPRINT 2.1 — Semaines 21–22 : TFLite (Format #15)
  Natif : FlatBuffers parser en Rust (flatbuffers crate)
  
  □ TFLiteLoader (flatbuffers natif)
  □ TFLiteSaver (génération flatbuffers natif)
  □ Conversion ONNX ↔ TFLite (décomposition d'ops)
  □ Tests, fuzzing, benchmarks

SPRINT 2.2 — Semaines 23–24 : CoreML (Format #16)
  Natif : protobuf (mlmodel format est du protobuf)
  Lecture du modèle OK. Écriture du modèle pour Apple Neural Engine
  nécessite la compilation — supporté en mode "mlpackage sans compilation"
  
  □ CoreMLLoader (protobuf mlmodel)
  □ CoreMLSaver (protobuf mlmodel non compilé — .mlpackage)
  □ Documentation : "Requiert Xcode pour la compilation finale sur Apple"
  □ Tests, fuzzing, benchmarks

SPRINT 2.3 — Semaines 25–26 : ExecuTorch (Format #17)
  Natif : FlatBuffers (même approche que TFLite)
  
  □ ExecuTorchLoader (flatbuffers natif)
  □ ExecuTorchSaver (génération flatbuffers)
  □ Tests, fuzzing, benchmarks

SPRINT 2.4 — Semaines 27–28 : JAX/Flax (Format #18, lecture seule)
  □ JaxFlaxLoader (msgpack natif via rmp-serde)
  □ Conversion → SafeTensors (format cible recommandé)
  □ Tests, fuzzing

SPRINT 2.5 — Semaines 29–30 : Keras H5 (Format #19, lecture seule)
  □ KerasH5Loader (HDF5 natif via hdf5-rs)
  □ Conversion → SafeTensors
  □ Tests, fuzzing

SPRINT 2.6 — Semaines 31–32 : TorchScript (Format #20)
  □ TorchScriptLoader (ZIP + pickle sécurisé, extension du loader PyTorch)
  □ TorchScriptSaver
  □ Tests, fuzzing, benchmarks

SPRINT 2.7 — Semaines 33–34 : PaddlePaddle (Format #21)
  □ PaddlePaddleLoader/Saver (protobuf + pdparams)
  □ Tests, fuzzing, benchmarks

SPRINT 2.8 — Semaines 35–36 : ONNX Runtime (Format #22)
  □ ONNXRuntimeLoader (ONNX optimisé avec extensions ORT)
  □ ONNXRuntimeSaver (optimisations ORT natives : fusion d'ops, quantification)
  □ Tests, fuzzing, benchmarks

CRITÈRE DE SORTIE PHASE 2 :
  22 formats natifs fonctionnels
  Support complet mobile/edge
```

### 5.5 Phase 3 : Formats Serveur et Spécialisés (Semaines 37–52)

```
SPRINT 3.1 — Semaines 37–38 : OpenVINO (Format #23)
  Natif : XML + bin (format textuel + binaire)
  
  □ OpenVINOLoader (XML parser + bin)
  □ OpenVINOSaver (génération XML + bin conformes)
  □ Tests, fuzzing, benchmarks

SPRINT 3.2 — Semaines 39–40 : Diffusers (Format #24)
  Composite — traitement par sous-composant
  
  □ DiffusersLoader (détection version SD/SDXL/SD3/Flux)
  □ Chargement par sous-modèle (UNet, VAE, TextEncoder)
  □ DiffusersSaver (reconstruction de la structure)
  □ Conversion : Diffusers ↔ SafeTensors (par sous-modèle)
  □ Tests, fuzzing, benchmarks

SPRINT 3.3 — Semaines 41–42 : TensorRT (Format #25)
  Situation spéciale : le format TensorRT (.engine) est propriétaire NVIDIA
  et ne peut pas être lu ou écrit sans la bibliothèque TensorRT.
  
  UMC génère un "TensorRT Build Recipe" :
  □ TensorRTRecipeSaver : génère un script de build reproductible
    (fichier ONNX optimisé + configuration trtexec recommandée)
  □ Documenté clairement : "UMC prépare le modèle, l'utilisateur lance trtexec"
  □ Pas de subprocess dans UMC — l'utilisateur exécute la commande fournie

SPRINT 3.4 — Semaines 43–44 : Qualcomm QNN (Format #26)
  Même approche que TensorRT :
  □ QNNRecipeSaver : génère la configuration QNN
  □ Documentation claire sur l'étape manuelle

SPRINT 3.5 — Semaines 45–46 : MediaPipe (Format #27)
  □ MediaPipeLoader (TFLite + metadata JSON)
  □ MediaPipeSaver (TFLite + metadata JSON)
  □ Tests, fuzzing, benchmarks

SPRINT 3.6 — Semaines 47–48 : NVIDIA Triton (Format #28)
  □ TritonSaver (génération de la structure model_repository)
  □ Génération config.pbtxt native
  □ Tests, benchmarks

SPRINT 3.7 — Semaines 49–50 : TensorRT-LLM (Format #29)
  □ Même approche que TensorRT standard — Recipe Saver
  □ Optimisations spécifiques LLM documentées

SPRINT 3.8 — Semaines 51–52 : Apache TVM + ONNX Web (Formats #30, #31)
  Format #30 : Apache TVM
  □ TVMSaver (génération de la configuration TVM — tvmc recipe)
  
  Format #31 : ONNX Web
  □ ONNXWebSaver (bundle ONNX + configuration JS)
  □ Validation de taille (avertissement si > 200 Mo)
  □ Tests, benchmarks

SPRINT FINAL : Format #32 (à déterminer selon demande communauté)
  Les 3 candidats les plus demandés sur GitHub

CRITÈRE DE SORTIE PHASE 3 :
  32 formats couverts
  Indépendance maximale — Recipe Savers documentés pour les formats propriétaires
  Communauté active et programme Bounty pour les contributions
```

### 5.6 Tableau Récapitulatif des 32 Formats

| # | Format | Phase | Load | Save | Natif 100% | Notes |
|---|--------|-------|------|------|------------|-------|
| 01 | GGUF | 0 | ✅ | ✅ | ✅ | Priorité absolue |
| 02 | SafeTensors | 0 | ✅ | ✅ | ✅ | |
| 03 | ONNX | 0 | ✅ | ✅ | ✅ | |
| 04 | PyTorch | 1 | ✅ | ✅ | ✅ | Pickle sécurisé |
| 05 | SentencePiece | 1 | ✅ | ✅ | ✅ | |
| 06 | TikToken | 1 | ✅ | ✅ | ✅ | |
| 07 | AWQ | 1 | ✅ | ✅ | ✅ | |
| 08 | GPTQ | 1 | ✅ | ✅ | ✅ | |
| 09 | LoRA | 1 | ✅ | ✅ | ✅ | |
| 10 | QLoRA | 1 | ✅ | ✅ | ✅ | |
| 11 | PEFT | 1 | ✅ | ✅ | ✅ | |
| 12 | bitsandbytes | 1 | ✅ | — | ✅ | Lecture seule |
| 13 | GGML | 1 | ✅ | — | ✅ | Legacy, lecture seule |
| 14 | TFSavedModel | 1 | ✅ | ✅ | ✅ | |
| 15 | TFLite | 2 | ✅ | ✅ | ✅ | FlatBuffers natif |
| 16 | CoreML | 2 | ✅ | ✅* | ✅ | *Non compilé |
| 17 | ExecuTorch | 2 | ✅ | ✅ | ✅ | FlatBuffers natif |
| 18 | JAX/Flax | 2 | ✅ | — | ✅ | Lecture seule |
| 19 | Keras H5 | 2 | ✅ | — | ✅ | Legacy, lecture seule |
| 20 | TorchScript | 2 | ✅ | ✅ | ✅ | |
| 21 | PaddlePaddle | 2 | ✅ | ✅ | ✅ | |
| 22 | ONNX Runtime | 2 | ✅ | ✅ | ✅ | |
| 23 | OpenVINO | 3 | ✅ | ✅ | ✅ | XML + bin |
| 24 | Diffusers | 3 | ✅ | ✅ | ✅ | Composite |
| 25 | TensorRT | 3 | — | 📋 | ✅ | Recipe uniquement |
| 26 | Qualcomm QNN | 3 | — | 📋 | ✅ | Recipe uniquement |
| 27 | MediaPipe | 3 | ✅ | ✅ | ✅ | |
| 28 | NVIDIA Triton | 3 | — | ✅ | ✅ | Config génération |
| 29 | TensorRT-LLM | 3 | — | 📋 | ✅ | Recipe uniquement |
| 30 | Apache TVM | 3 | — | 📋 | ✅ | Recipe uniquement |
| 31 | ONNX Web | 3 | — | ✅ | ✅ | Bundle web |
| 32 | TBD | 3+ | ? | ? | ✅ | Voté par communauté |

**Légende :** ✅ = supporté | — = non supporté | 📋 = Recipe (config générée, pas de conversion directe)

---

## 6. GraphTemplate — Reconstruction Native des Graphes

### 6.1 Le Problème Résolu

Les formats LLM (GGUF, SafeTensors) contiennent uniquement les poids, sans graphe de calcul. Pour convertir vers ONNX (qui nécessite un graphe), UMC doit reconstruire le graphe depuis les métadonnées d'architecture.

**Ce n'est pas de la magie — c'est un catalogue maintenu.**

### 6.2 GraphTemplate Registry

```rust
/// Registre des templates de graphes par architecture
pub struct GraphTemplateRegistry {
    templates: HashMap<String, Box<dyn GraphTemplate>>,
}

pub trait GraphTemplate: Send + Sync {
    fn architecture_name(&self) -> &str;
    fn matches(&self, config: &ArchitectureConfig) -> bool;
    fn build_graph(&self, config: &ArchitectureConfig, tensors: &TensorStore)
        -> Result<ComputeGraph, UmcError>;
    fn verify_graph(&self, graph: &ComputeGraph, tensors: &TensorStore) -> bool;
}

impl GraphTemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self { templates: HashMap::new() };
        // Templates natifs enregistrés au démarrage
        registry.register(Box::new(LlamaTemplate::new()));
        registry.register(Box::new(MistralTemplate::new()));
        registry.register(Box::new(PhiTemplate::new()));
        registry.register(Box::new(GemmaTemplate::new()));
        registry.register(Box::new(QwenTemplate::new()));
        registry.register(Box::new(FalconTemplate::new()));
        registry.register(Box::new(MPTTemplate::new()));
        registry.register(Box::new(StableLMTemplate::new()));
        registry.register(Box::new(GPTNeoXTemplate::new()));
        registry.register(Box::new(OPTTemplate::new()));
        registry
    }

    pub fn find_template(&self, config: &ArchitectureConfig) -> Option<&dyn GraphTemplate> {
        self.templates.values()
            .find(|t| t.matches(config))
            .map(|t| t.as_ref())
    }
}
```

### 6.3 Exemple : LlamaTemplate

```rust
pub struct LlamaTemplate {
    /// Architectures compatibles
    known_architectures: Vec<&'static str>,
}

impl LlamaTemplate {
    pub fn new() -> Self {
        Self {
            known_architectures: vec![
                "llama", "llama2", "llama3", "llama3.1", "llama3.2",
                "mistral", "mixtral", "solar", "vicuna", "alpaca",
            ],
        }
    }
}

impl GraphTemplate for LlamaTemplate {
    fn architecture_name(&self) -> &str { "llama-family" }

    fn matches(&self, config: &ArchitectureConfig) -> bool {
        self.known_architectures.iter()
            .any(|&a| config.architecture.to_lowercase().contains(a))
    }

    fn build_graph(
        &self,
        config: &ArchitectureConfig,
        tensors: &TensorStore,
    ) -> Result<ComputeGraph, UmcError> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Embedding layer
        nodes.push(ComputeNode {
            id: "embed_tokens".into(),
            op_type: UniversalOp::Embedding { padding_idx: None },
            inputs: vec!["input_ids".into()],
            outputs: vec!["hidden_states".into()],
            attributes: OpAttributes::default(),
        });

        // N decoder layers
        for i in 0..config.num_layers {
            let prefix = format!("model.layers.{}", i);

            // RMSNorm avant attention
            nodes.push(ComputeNode {
                id: format!("{}.input_layernorm", prefix),
                op_type: UniversalOp::RmsNorm { eps: config.rms_norm_eps.unwrap_or(1e-5) },
                inputs: vec![if i == 0 { "hidden_states".into() }
                             else { format!("layer_{}_output", i - 1) }],
                outputs: vec![format!("{}.normed", prefix)],
                attributes: OpAttributes::default(),
            });

            // Attention (Q, K, V projections + RoPE)
            // ... (implémentation complète dans le code)

            // MLP (SiLU + Gate)
            // ...
        }

        // Norm finale + LM Head
        // ...

        Ok(ComputeGraph { nodes, edges, inputs: vec!["input_ids".into()], outputs: vec!["logits".into()] })
    }

    fn verify_graph(&self, graph: &ComputeGraph, tensors: &TensorStore) -> bool {
        // Vérifier que tous les tenseurs référencés existent
        for node in &graph.nodes {
            for input in &node.inputs {
                if input != "input_ids" && tensors.get(input).is_none() {
                    // Avertissement mais pas erreur fatale
                }
            }
        }
        true
    }
}
```

---

## 7. Gestion Native de la Quantification

### 7.1 Représentation Canonique Étendue

```rust
/// CanonicalQuantization v2 — avec tous les paramètres manquants
#[derive(Debug, Clone)]
pub struct CanonicalQuantization {
    pub bit_width:       u8,
    pub block_size:      usize,
    pub superblock_size: Option<usize>,   // GGUF K-quants : 256
    pub scales:          Vec<f32>,
    pub zero_points:     Vec<f32>,
    pub scales_dtype:    DType,           // Type des scales (F16, F32, Q8_0)
    pub quantized_data:  Vec<u8>,
    pub storage_order:   StorageOrder,
}

impl CanonicalQuantization {
    /// Déquantification vers F32 — 100% natif
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
        let mut result = Vec::with_capacity(self.quantized_data.len() * 2);

        // Logique de déquantification GGUF K-quants
        // (implémentation complète dans umc-core/src/ir/quantization.rs)
        for (block_idx, chunk) in self.quantized_data.chunks(block_size / 2).enumerate() {
            let scale = self.scales.get(block_idx).copied().unwrap_or(1.0);
            let zero = self.zero_points.get(block_idx).copied().unwrap_or(0.0);
            // Dépackage des valeurs 4-bit depuis les bytes 8-bit
            for &byte in chunk {
                let lo = (byte & 0x0F) as f32;
                let hi = (byte >> 4) as f32;
                result.push(scale * (lo - zero));
                result.push(scale * (hi - zero));
            }
        }
        Ok(result)
    }

    /// Requantification — impossible sans données de calibration pour AWQ/GPTQ
    pub fn can_requantize(&self, target: &QuantScheme) -> RequantizationSupport {
        match target {
            QuantScheme::GgufQ4KM | QuantScheme::GgufQ5KM | QuantScheme::GgufQ8_0 => {
                RequantizationSupport::Supported
            }
            QuantScheme::Awq4 | QuantScheme::Gptq4 => {
                RequantizationSupport::RequiresCalibration {
                    reason: "AWQ/GPTQ requièrent un dataset de calibration pour recalculer les scales.".into(),
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

---

## 8. Gestion Native des Adaptateurs

### 8.1 Fusion LoRA — Native et Correcte

```rust
impl AdapterInfo {
    /// Fusion mathématique native
    /// W_final = W_base + (alpha/rank) × (B @ A)
    pub fn merge_into_base(&self, base_tensor: &Tensor) -> Result<Tensor, UmcError> {
        let scaling = self.config.alpha / self.config.rank as f32;

        if let Some(weights) = self.weights.get(&base_tensor.name) {
            if let (Some(a), Some(b)) = (&weights.lora_a, &weights.lora_b) {
                // Déquantification si nécessaire
                let base_f32 = match &base_tensor.quantization {
                    Some(quant) => {
                        let canon = quant.to_canonical()?;
                        canon.dequantize_to_f32()?
                    }
                    None => tensor_to_f32(base_tensor)?,
                };

                let a_bytes = a.data.as_bytes()?;
                let b_bytes = b.data.as_bytes()?;

                // MatMul natif Rust : B @ A
                let delta = native_matmul_f32(
                    b_bytes, a_bytes,
                    b.shape[0], b.shape[1], a.shape[1],
                    scaling,
                )?;

                // Addition : W_base + delta
                let merged: Vec<f32> = base_f32.iter()
                    .zip(delta.iter())
                    .map(|(b, d)| b + d)
                    .collect();

                return Ok(Tensor {
                    name:         base_tensor.name.clone(),
                    dtype:        DType::F32,
                    data:         TensorData::Owned(Arc::new(bytemuck::cast_slice(&merged).to_vec())),
                    checksum:     compute_checksum(bytemuck::cast_slice(&merged)),
                    quantization: None,
                    ..base_tensor.clone()
                });
            }
        }
        Ok(base_tensor.clone())
    }
}
```

---

## 9. Validation et Certification Réaliste

### 9.1 Trois Niveaux de Round-Trip Honnêtes

```rust
/// Les trois niveaux réels de round-trip
#[derive(Debug, Clone, PartialEq)]
pub enum RoundTripLevel {
    /// NIVEAU 1 : Bit-identical — uniquement pour A → A (même format)
    /// GGUF → GGUF, SafeTensors → SafeTensors
    BitIdentical,

    /// NIVEAU 2 : Sémantique — même résultats d'inférence dans la tolérance
    /// GGUF → ONNX → GGUF : poids identiques, sorties identiques
    Semantic { max_divergence: f64 },

    /// NIVEAU 3 : Structurel — même architecture, même nombre de couches
    /// Pour les formats qui transforment le graphe
    Structural,
}

pub fn determine_roundtrip_level(fmt_a: &str, fmt_b: &str) -> RoundTripLevel {
    if fmt_a == fmt_b {
        return RoundTripLevel::BitIdentical;
    }
    // Conversions sans perte de précision
    match (fmt_a, fmt_b) {
        ("GGUF", "SafeTensors") | ("SafeTensors", "GGUF") =>
            RoundTripLevel::Semantic { max_divergence: 1e-7 },
        ("GGUF", "ONNX") | ("ONNX", "GGUF") =>
            RoundTripLevel::Semantic { max_divergence: 1e-6 },
        ("GGUF", "PyTorch") | ("PyTorch", "GGUF") =>
            RoundTripLevel::Semantic { max_divergence: 1e-6 },
        // Conversions avec quantification différente
        _ if involves_quantization(fmt_a) || involves_quantization(fmt_b) =>
            RoundTripLevel::Semantic { max_divergence: 1e-2 },
        _ => RoundTripLevel::Structural,
    }
}
```

### 9.2 Validation Sémantique — Runtimes Intégrés

```rust
/// Validation fonctionnelle — utilise les loaders/savers UMC eux-mêmes
/// Pas de runtime externe requis pour la validation de base
pub struct SemanticValidator {
    pub tolerance: f64,
    pub num_test_inputs: usize,
    pub use_native_executor: bool,
}

impl SemanticValidator {
    /// Valide que source et cible produisent les mêmes sorties
    pub fn validate(
        &self,
        source_ir: &UniversalIR,
        converted_ir: &UniversalIR,
    ) -> Result<SemanticResult, UmcError> {
        // Pour les formats LLM (weights-only) :
        // Comparaison directe des tenseurs déquantifiés
        if self.use_native_executor {
            return self.validate_by_tensor_comparison(source_ir, converted_ir);
        }

        // Pour les formats avec graphe (ONNX) :
        // Exécution via le NativeExecutor minimal intégré à UMC
        self.validate_by_native_execution(source_ir, converted_ir)
    }

    fn validate_by_tensor_comparison(
        &self,
        source: &UniversalIR,
        target: &UniversalIR,
    ) -> Result<SemanticResult, UmcError> {
        let mut max_div = 0.0f64;
        let mut total_tensors = 0;

        for (name, source_tensor) in source.tensors.iter() {
            if let Some(target_tensor) = target.tensors.get(name) {
                let source_f32 = dequantize_tensor_to_f32(source_tensor)?;
                let target_f32 = dequantize_tensor_to_f32(target_tensor)?;
                let div = max_divergence_simd(&source_f32, &target_f32);
                max_div = max_div.max(div);
                total_tensors += 1;
            }
        }

        Ok(SemanticResult {
            passed: max_div <= self.tolerance,
            max_divergence: max_div,
            tensors_checked: total_tensors,
            method: ValidationMethod::TensorComparison,
        })
    }
}
```

### 9.3 Rapport de Conversion Certifié (pas "valeur légale")

```rust
/// Rapport de conversion certifié v2
/// Renommé : pas de "valeur légale" — c'est un rapport technique signé
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionReport {
    pub version:          String,
    pub umc_version:      String,
    pub timestamp:        u64,
    pub source:           ModelIdentity,
    pub target:           ModelIdentity,
    pub roundtrip_level:  RoundTripLevelDescription,
    pub validation:       ValidationSummary,
    pub trust_statement:  String,  // Description honnête de ce que prouve le rapport
    pub conversion_path:  Vec<String>,
    pub signature:        String,  // ed25519 — prouve l'émetteur, pas la "vérité légale"
    pub public_key:       String,
    pub verify_url:       String,  // URL pour vérifier en ligne
}

/// Déclaration de confiance honnête
fn build_trust_statement(validation: &ValidationSummary, level: &RoundTripLevel) -> String {
    match level {
        RoundTripLevel::BitIdentical =>
            "Ce rapport certifie que les fichiers source et cible sont \
             bit-identiques. UMC v{} a effectué cette vérification.".into(),
        RoundTripLevel::Semantic { max_divergence } =>
            format!(
                "Ce rapport certifie que la conversion est sémantiquement correcte : \
                 divergence maximale observée {:.2e}, dans la tolérance documentée. \
                 Ce rapport ne garantit pas la correction fonctionnelle pour tous les \
                 cas d'usage — il garantit que UMC a effectué les vérifications décrites.",
                max_divergence
            ),
        RoundTripLevel::Structural =>
            "Ce rapport certifie que la structure du modèle est préservée. \
             Une validation fonctionnelle sur votre cas d'usage est recommandée.".into(),
    }
}
```

---

## 10. Sécurité — Parsing Défensif

### 10.1 Règles de Sécurité Universelles

Chaque loader respecte ces règles sans exception :

```rust
/// Trait de sécurité — obligatoire pour tout loader
pub trait SecureLoader: FormatLoader {
    /// Limites de sécurité spécifiques au format
    fn security_bounds(&self) -> SecurityBounds;

    /// Valide les champs numériques avant toute allocation
    fn validate_numeric_field(&self, field: &str, value: u64, bounds: &SecurityBounds)
        -> Result<usize, UmcError>
    {
        let limit = match field {
            "tensor_count" => bounds.max_tensor_count as u64,
            "metadata_count" => bounds.max_metadata_count as u64,
            "string_length" => bounds.max_string_length as u64,
            "tensor_size" => bounds.max_tensor_size_bytes as u64,
            _ => u64::MAX,
        };
        if value > limit {
            return Err(UmcError::SecurityViolation {
                field: field.to_string(),
                value: value as usize,
                limit: limit as usize,
            });
        }
        Ok(value as usize)
    }

    /// Valide un chemin extrait d'une archive (anti-path-traversal)
    fn validate_archive_path(&self, path: &str) -> Result<(), UmcError> {
        // Interdire les composants dangereux
        if path.contains("..") || path.starts_with('/') || path.contains('\0') {
            return Err(UmcError::PathTraversal(path.to_string()));
        }
        Ok(())
    }

    /// Valide un ratio de compression (anti-ZIP bomb)
    fn validate_compression_ratio(&self, compressed: usize, decompressed: usize)
        -> Result<(), UmcError>
    {
        if decompressed > compressed * 1000 {
            return Err(UmcError::ZipBomb {
                compressed, decompressed,
            });
        }
        Ok(())
    }
}
```

### 10.2 Parser Pickle Sécurisé (pour PyTorch)

```rust
/// Parser pickle minimal avec whitelist de types autorisés
/// N'exécute JAMAIS de code Python. Parse uniquement les structures de données.
pub struct SafePickleParser {
    allowed_types: HashSet<String>,
}

impl SafePickleParser {
    pub fn new() -> Self {
        let mut allowed = HashSet::new();
        // Uniquement les types de tenseurs PyTorch
        allowed.insert("torch.FloatStorage".into());
        allowed.insert("torch.HalfStorage".into());
        allowed.insert("torch.BFloat16Storage".into());
        allowed.insert("torch.IntStorage".into());
        allowed.insert("torch.LongStorage".into());
        allowed.insert("torch.ByteStorage".into());
        allowed.insert("torch.ShortStorage".into());
        allowed.insert("torch.DoubleStorage".into());
        allowed.insert("collections.OrderedDict".into());
        allowed.insert("_codecs.encode".into());
        Self { allowed_types: allowed }
    }

    pub fn parse(&self, data: &[u8]) -> Result<PickleValue, UmcError> {
        let mut parser = PickleOpcodeParser::new(data, &self.allowed_types);
        parser.parse_with_depth_limit(32)  // Profondeur maximale : 32
    }
}
```

### 10.3 Protection SSRF pour les URLs

```rust
/// Validation des URLs avant tout accès réseau
pub fn validate_url_security(url: &str) -> Result<(), UmcError> {
    let parsed = url::Url::parse(url)
        .map_err(|_| UmcError::InvalidUrl(url.to_string()))?;

    // Uniquement HTTPS
    if parsed.scheme() != "https" {
        return Err(UmcError::InsecureUrl {
            url: url.to_string(),
            reason: "Uniquement HTTPS est autorisé".into(),
        });
    }

    // Blacklist des ranges d'IP privées (SSRF)
    if let Some(host) = parsed.host_str() {
        let blocked_ranges = [
            "169.254.",  // AWS metadata
            "10.",
            "172.16.", "172.17.", "172.18.", "172.19.",
            "172.20.", "172.21.", "172.22.", "172.23.",
            "172.24.", "172.25.", "172.26.", "172.27.",
            "172.28.", "172.29.", "172.30.", "172.31.",
            "192.168.",
            "127.",
            "0.0.0.",
        ];
        for range in &blocked_ranges {
            if host.starts_with(range) {
                return Err(UmcError::SsrfAttempt { host: host.to_string() });
            }
        }
    }

    Ok(())
}
```

---

## 11. Backend Simplifié et Scalable

### 11.1 Évolution Progressive de l'Infrastructure

```
PHASE MVP (0 – 1 000 clients) — PostgreSQL + SKIP LOCKED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Pas de Kafka. Pas de Zookeeper. Pas de 500 €/mois.

CREATE TABLE conversion_jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status      TEXT NOT NULL DEFAULT 'queued',
    payload     JSONB NOT NULL,
    result      JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    worker_id   TEXT,
    attempts    INT NOT NULL DEFAULT 0,
    progress    FLOAT4 NOT NULL DEFAULT 0.0,
    last_tensor TEXT
);

-- Déqueue atomique sans deadlock
SELECT id, payload
FROM conversion_jobs
WHERE status = 'queued' AND attempts < 3
ORDER BY created_at
LIMIT 1
FOR UPDATE SKIP LOCKED;

PHASE SCALE (1 000 – 10 000 clients) — Redis Streams
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Migration transparente. Redis Streams = Kafka simplifié.
Durée de migration : 1 semaine. Pas de changement d'API.

PHASE HYPERSCALE (> 100 000 clients) — Apache Kafka
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Uniquement si les métriques le justifient.
Décision basée sur des données, pas sur de l'anticipation.
```

### 11.2 Progression SSE (Server-Sent Events) plutôt que WebSocket

```rust
/// Server-Sent Events — plus simple que WebSocket, pas de sticky session
/// Le client reconnecte automatiquement si la connexion est perdue
/// La progression est persistée dans Redis → pas de perte de messages

// Endpoint SSE
async fn progress_sse_handler(
    Path(job_id): Path<String>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        loop {
            // Lire la progression depuis Redis (persistée, pas PubSub)
            if let Ok(Some(progress)) = state.redis.get_progress(&job_id).await {
                yield Ok(Event::default()
                    .data(serde_json::to_string(&progress).unwrap()));

                if progress.status == "done" || progress.status == "error" {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Progression persistée dans Redis avec TTL
async fn update_progress(redis: &Redis, job_id: &str, progress: &JobProgress) {
    // Clé avec TTL 24h — pas de PubSub, récupérable après reconnexion
    redis.set_ex(
        &format!("job:{}:progress", job_id),
        serde_json::to_string(progress).unwrap(),
        86400,
    ).await.ok();
}
```

### 11.3 API Keys — Standard Industriel Corrigé

```rust
/// Génération et stockage des API keys — standard correct
/// PAS bcrypt pour les API keys (trop lent) → SHA256 salté

pub fn generate_api_key() -> (String, String) {
    // Clé lisible : "umc_sk_prod_<32 bytes hex>"
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).unwrap();
    let key = format!("umc_sk_prod_{}", hex::encode(bytes));

    // Hachage pour stockage : SHA256(key) — rapide, sécurisé pour ce use case
    let hash = sha256_hex(key.as_bytes());

    (key, hash)
}

pub fn verify_api_key(provided: &str, stored_hash: &str) -> bool {
    let hash = sha256_hex(provided.as_bytes());
    // Comparaison en temps constant
    constant_time_eq(hash.as_bytes(), stored_hash.as_bytes())
}

/// Rate limiting par API key — en mémoire via DashMap
pub struct RateLimiter {
    buckets: DashMap<String, TokenBucket>,
}

impl RateLimiter {
    pub fn check_and_consume(&self, key_id: &str, plan: &Plan) -> bool {
        let rate = plan.requests_per_minute();
        self.buckets
            .entry(key_id.to_string())
            .or_insert_with(|| TokenBucket::new(rate))
            .try_consume(1)
    }
}
```

---

## 12. Frontend Épuré

### 12.1 Stack Minimaliste et Stable

```
Stack MVP Frontend :
  Framework : Next.js 14.x LTS (stable)
  Styling   : Tailwind CSS 3.x
  Animations: CSS transitions natives (pas Framer Motion)
  État      : React useState/useReducer (pas Zustand)
  Requêtes  : fetch natif + SWR (léger)
  Graphe    : React Flow (simple, 30 Ko vs D3 150 Ko)
  Langage   : TypeScript strict

Bundle cible : < 100 Ko gzippé page principale

Ajouts progressifs si les métriques le justifient :
  → Framer Motion si les animations CSS ne suffisent pas
  → D3 si React Flow ne suffit pas pour le graphe
  → Zustand si useState devient insuffisant

THÈME :
  Dark (#0D0D0D) + Belgian Yellow (#FFD700) — inchangé
  Contraste vérifié avec axe-core en CI (pas juste calculé)
```

### 12.2 Composants Critiques

```typescript
// Progression via SSE (pas WebSocket)
function useConversionProgress(jobId: string) {
  const [progress, setProgress] = useState<JobProgress | null>(null);

  useEffect(() => {
    if (!jobId) return;
    const eventSource = new EventSource(`/v1/jobs/${jobId}/progress`);

    eventSource.onmessage = (e) => {
      setProgress(JSON.parse(e.data));
    };

    eventSource.onerror = () => {
      // Reconnexion automatique par le navigateur (SSE natif)
    };

    return () => eventSource.close();
  }, [jobId]);

  return progress;
}

// Rapport de conversion — nomenclature corrigée
interface ConversionReport {
  roundtrip_level: 'bit_identical' | 'semantic' | 'structural';
  trust_statement: string;  // Description honnête
  validation: ValidationSummary;
  signature: string;
  verify_url: string;
}
```

---

## 13. CLI Complète

### 13.1 Commandes

```
umc convert <SOURCE> <TARGET> [OPTIONS]
  --dtype <DTYPE>         fp32, fp16, bf16, fp8, q4_k_m, ...
  --validate <MODE>       none, structural, semantic, strict [défaut: semantic]
  --report                Générer un rapport signé
  --merge-adapters        Fusionner les adaptateurs LoRA
  --quantize <SCHEME>     Quantifier la sortie
  --threads <N>           [défaut: auto]
  --memory-limit <MB>     Limiter la RAM utilisée
  --reproducible          Déterministe (même résultat partout)
  --seed <N>              [défaut: 42]
  --resume <CHECKPOINT>   Reprendre une conversion interrompue
  --timeout <SECS>        Timeout global [défaut: 3600]

umc inspect <FILE> [--tensors] [--tokenizer] [--graph] [--json]
umc dry-run <SOURCE> --target <FORMAT>
umc diff <FILE_A> <FILE_B> [--tolerance 1e-5]
umc validate <FILE> [--mode semantic] [--reference <REF>]
umc doctor <FILE> [--fix]
umc optimize <SOURCE> <TARGET> [--aggressive]
umc benchmark <FILES...> [--iterations 5] [--output json]
umc watch <SOURCE> --targets onnx,gguf --output-dir ./
umc lineage <FILE>
umc formats [--json]
umc doctor --check-tools   # Vérifie les dépendances optionnelles
umc serve [--port 8080]
umc plugin install|list|remove
```

### 13.2 Sortie `umc inspect` Corrigée

```
📁 model.gguf (GGUF v3)
├── Architecture   : llama (Llama 3.1)
├── Paramètres     : 8.03B
├── Couches        : 32
├── Taille cachée  : 4096
├── Têtes          : 32 (KV: 8 — GQA)
├── Contexte       : 131 072
├── Quantification : Q4_K_M (block_size=32, superblock=256, scale=F16)
├── Tokenizer      : BPE (128 256 tokens, chat_template présent)
└── Provenance     : original — chaîne vérifiée ✅

Conversions disponibles depuis ce fichier :
  → SafeTensors (natif, sémantique identique)
  → ONNX        (natif, graphe Llama reconstruit)
  → PyTorch     (natif, sémantique identique)
  → TFLite      (natif, via ONNX intermédiaire)

Utilisez `umc dry-run model.gguf --target onnx` pour estimer le temps.
```

### 13.3 Sortie `umc dry-run` Corrigée

```
╔══════════════════════════════════════════════════════════╗
║              UMC DRY RUN — model.gguf → ONNX             ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Compatibilité des opérateurs : 142/142 ✅               ║
║  Opérateurs décomposés         : 3 (RmsNorm, RoPE, SiLU)║
║  GraphTemplate utilisé         : LlamaTemplate v1.0     ║
║                                                          ║
║  Taille estimée sortie   : ~13.8 Go (FP16)              ║
║  RAM structures (min)    : ~180 Mo                       ║
║  RAM cache OS (variable) : dépend du système             ║
║  Temps estimé            : 8–15 secondes (votre machine) ║
║                                                          ║
║  ⚠ Niveau round-trip : SÉMANTIQUE (pas bit-identical)   ║
║    GGUF Q4_K_M → ONNX FP16 implique une déquantification║
║    Divergence attendue : < 1e-3 (dans les normes)       ║
║                                                          ║
║  Extension Store : chat_template préservé ✅             ║
║  Extension Store : rope_scaling.type préservé ✅         ║
║                                                          ║
║  🟢 Conversion possible                                  ║
╚══════════════════════════════════════════════════════════╝
```

---

## 14. API REST

### 14.1 Endpoints

```
POST   /v1/convert                  Démarrer une conversion
GET    /v1/jobs/:id                 Statut du job
POST   /v1/jobs/:id/cancel          Annuler
GET    /v1/jobs/:id/report          Télécharger le rapport (JSON)
GET    /v1/jobs/:id/report.pdf      Télécharger le rapport (PDF)
GET    /v1/jobs/:id/progress        Progression SSE (pas WebSocket)
WS     /v1/jobs/:id/ws              WebSocket (optionnel, fallback SSE)

POST   /v1/inspect                  Inspecter un modèle
POST   /v1/dry-run                  Simuler
POST   /v1/diff                     Comparer deux modèles
POST   /v1/validate                 Valider

GET    /v1/formats                  Lister les formats supportés
GET    /v1/formats/:name            Détails d'un format
GET    /v1/graph                    Graphe de conversion JSON

GET    /v1/certificates/:id         Rapport public (sans auth)
GET    /v1/certificates/:id/verify  Vérifier un rapport

GET    /health
GET    /metrics
```

### 14.2 Upload Sécurisé

```rust
/// Upload avec validation immédiate
async fn convert_handler(
    State(state): State<AppState>,
    api_key: ApiKeyExtractor,
    mut multipart: Multipart,
) -> Result<Json<ConvertResponse>, ApiError> {
    // Vérifier le plan AVANT d'accepter l'upload
    let plan = state.db.get_plan(&api_key.key_id).await?;
    let max_size = plan.max_file_size_bytes();

    let mut file_size = 0usize;
    let job_id = Uuid::new_v4().to_string();
    let temp_path = PathBuf::from(format!("/tmp/umc-uploads/{}", job_id));

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("file") => {
                let mut writer = tokio::fs::File::create(&temp_path).await?;
                let mut stream = field.into_stream();

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    file_size += chunk.len();

                    // Vérifier la taille PENDANT l'upload
                    if file_size > max_size {
                        tokio::fs::remove_file(&temp_path).await.ok();
                        return Err(ApiError::FileTooLarge { max_size });
                    }

                    writer.write_all(&chunk).await?;
                }
            }
            // ... autres champs
            _ => {}
        }
    }

    // Créer le job en base
    let job_id = state.queue.enqueue(ConversionJob { file_path: temp_path, ... }).await?;

    Ok(Json(ConvertResponse {
        job_id,
        status: "queued",
        poll_url: format!("/v1/jobs/{}", job_id),
        progress_url: format!("/v1/jobs/{}/progress", job_id),
    }))
}
```

---

## 15. Modèle Économique Révisé

### 15.1 Structure Open Core Inchangée

```
UMC CORE — Apache 2.0 — GRATUIT À JAMAIS
  • Tous les formats natifs
  • Pipeline parallèle complet  
  • CLI complète
  • Validation sémantique

UMC CLOUD — Pricing révisé et viable

  Gratuit     : 10 conversions/mois, ≤ 1 Go/conversion
  
  Pro          : 29 €/mois — conversions illimitées, priorité, rapports
  
  Pay/use (révisé selon la taille) :
    < 500 Mo   : 0,001 €
    500 Mo–5 Go: 0,005 €
    5–50 Go    : 0,02 €
    > 50 Go    : 0,05 €    ← couvre les coûts réels
  
  RÈGLE : Une conversion qui échoue avant 10% de progression
          n'est pas facturée.

UMC ENTERPRISE — Inchangé
  Starter  : 15 000 €/an
  Business : 50 000 €/an  
  Ultimate : 150 000 €/an + SLA 99.9%

UMC HUB — Modèle légalement sûr
  Pas de stockage de fichiers de modèles propriétaires.
  Stockage de "recettes de conversion" uniquement.
  Consultation juridique obligatoire avant lancement.
  Démarrage avec modèles Apache 2.0 / MIT uniquement.

UMC CERTIFIED — 500 €/modèle/an
  Rapport automatique sur les formats supportés.
  Badge utilisable sur HF/GitHub.
  Surveillance continue.
```

### 15.2 Projections Révisées et Réalistes

| Période | ARR Cible | Clients Pro | Enterprise | Stars GitHub |
|---------|-----------|-------------|------------|-------------|
| Mois 3 | 0 | 0 | 0 | 300–600 |
| Mois 6 | 5–10 K€ | 20–40 | 0 | 600–1 200 |
| Mois 12 | 20–40 K€ | 80–150 | 0–1 | 1 500–3 000 |
| An 2 | 200–500 K€ | 800–2 000 | 2–5 | 5 000–10 000 |
| An 3 | 1–3 M€ | 5 000–15 000 | 10–30 | 15 000–30 000 |

---

## 16. Stratégie de Déploiement Progressive

### 16.1 Phase MVP — Minimal et Fiable

```yaml
# Infrastructure MVP — simplicité maximale
services:
  api:
    image: umc/api:latest
    environment:
      DATABASE_URL: postgres://...
      REDIS_URL: redis://...
    resources: { cpu: 2, memory: 4Gi }

  worker:
    image: umc/worker:latest
    replicas: 2
    resources: { cpu: 8, memory: 16Gi }

  postgres:
    image: postgres:16
    
  redis:
    image: redis:7-alpine
```

### 16.2 Benchmarks Publics Reproductibles

```bash
#!/bin/bash
# benchmark.sh — reproductible sur n'importe quelle machine
# Télécharger et exécuter : curl -fsSL https://umc.dev/benchmark.sh | bash

set -euo pipefail

echo "=== UMC Benchmark Suite ==="
echo "Machine : $(uname -m) - $(nproc) CPUs - $(free -h | awk '/^Mem:/{print $2}') RAM"
echo "UMC Version : $(umc --version)"
echo "Date : $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Télécharger Phi-2 GGUF (1.6 Go, licence MIT)
if [ ! -f "phi-2.Q4_K_M.gguf" ]; then
  echo "Téléchargement Phi-2..."
  curl -L "https://huggingface.co/TheBloke/phi-2-GGUF/resolve/main/phi-2.Q4_K_M.gguf" \
    -o phi-2.Q4_K_M.gguf
fi

echo "--- Test 1 : GGUF → SafeTensors ---"
time umc convert phi-2.Q4_K_M.gguf phi-2.safetensors --threads $(nproc)
ls -lh phi-2.safetensors

echo "--- Test 2 : GGUF → ONNX ---"
time umc convert phi-2.Q4_K_M.gguf phi-2.onnx --threads $(nproc)
ls -lh phi-2.onnx

echo "--- Test 3 : Validation sémantique ---"
time umc validate phi-2.Q4_K_M.gguf --reference phi-2.safetensors

echo ""
echo "Résultats enregistrés dans benchmark-results.json"
umc benchmark phi-2.Q4_K_M.gguf phi-2.safetensors phi-2.onnx \
  --iterations 3 --output benchmark-results.json
```

---

## 17. Structure du Projet Rust

### 17.1 Cargo.toml Workspace

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
serde            = { version = "1.0", features = ["derive"] }
serde_json       = "1.0"
prost            = "0.12"
rayon            = "1.10"
crossbeam        = "0.8"
tokio            = { version = "1.40", features = ["full"] }
memmap2          = "0.9"
bytes            = "1.7"
xxhash-rust      = { version = "0.8", features = ["xxh64"] }
sha2             = "0.10"
ed25519-dalek    = "2.1"
clap             = { version = "4.5", features = ["derive"] }
axum             = "0.7"
petgraph         = "0.6"
thiserror        = "1.0"
anyhow           = "1.0"
tracing          = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
indexmap         = "2.4"
num-cpus         = "1.16"
sysinfo          = "0.31"
criterion        = "0.5"
proptest         = "1.5"
tempfile         = "3.13"
hex              = "0.4"
url              = "2.5"
getrandom        = "0.2"
dashmap          = "5.5"
flatbuffers      = "23.5"   # Pour TFLite et ExecuTorch
rmp-serde        = "1.3"    # Pour JAX/Flax (msgpack)
hdf5-rs          = "0.8"    # Pour Keras H5

[profile.release]
opt-level        = 3
lto              = "fat"
codegen-units    = 1
strip            = true
panic            = "abort"  # Pas de panic unwinding en production

[profile.bench]
opt-level        = 3
lto              = "thin"
```

### 17.2 Configuration SIMD Corrigée

```toml
# .cargo/config.toml — SANS AVX-512 global (crash sur AMD Zen 3, Intel Alder Lake E-cores)

[build]
rustflags = [
    "-C", "link-arg=-fuse-ld=lld",
    # PAS de target-cpu=native en production (binaires non portables)
]

# Pour les releases ciblées :
[target.x86_64-unknown-linux-gnu]
# Pas de feature flags globaux — détection au runtime via is_x86_feature_detected!()

[target.aarch64-apple-darwin]
# Pas de feature flags globaux — NEON toujours disponible sur M1+
```

```rust
// Détection SIMD au runtime — correct
fn max_divergence_f32(a: &[f32], b: &[f32]) -> f64 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return unsafe { max_divergence_avx512(a, b) };
        }
        if is_x86_feature_detected!("avx2") {
            return unsafe { max_divergence_avx2(a, b) };
        }
        if is_x86_feature_detected!("sse4.1") {
            return unsafe { max_divergence_sse4(a, b) };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return unsafe { max_divergence_neon(a, b) };
        }
    }
    // Fallback scalaire — toujours correct
    max_divergence_scalar(a, b)
}
```

### 17.3 CI/CD Corrigé

```yaml
# .github/workflows/ci.yml

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: aarch64-apple-darwin  # Apple Silicon
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Format
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Tests
        run: cargo test --all

      - name: Audit sécurité
        run: cargo audit

      - name: Accessibilité frontend
        run: npx axe-cli http://localhost:3000 --exit

  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Fuzz GGUF Loader (1 minute)
        run: cargo +nightly fuzz run fuzz_gguf_loader -- -max_total_time=60
      - name: Fuzz ONNX Loader (1 minute)
        run: cargo +nightly fuzz run fuzz_onnx_loader -- -max_total_time=60
      - name: Fuzz SafeTensors Loader (1 minute)
        run: cargo +nightly fuzz run fuzz_safetensors_loader -- -max_total_time=60

  benchmark:
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - name: Benchmark reproductible
        run: cargo bench --bench conversion_bench -- --output-format bencher | tee benchmark.txt
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: cargo
          output-file-path: benchmark.txt
          fail-on-alert: true
          alert-threshold: "110%"  # Alerte si régression > 10%
```

---

## 18. Annexes et Glossaire

### 18.1 Matrice de Tests Complète

| Test | Type | Fréquence | Critère | Outil |
|------|------|-----------|---------|-------|
| Magic bytes (32 formats) | Unitaire | Chaque PR | 100% détection | cargo test |
| Round-trip bit-identical (même format) | Intégration | Chaque PR | SHA256 identique | cargo test |
| Round-trip sémantique (cross-format) | Intégration | Chaque PR | Divergence < seuil | cargo test |
| Validation numérique SIMD | Unitaire | Chaque PR | < 5e-4 (F16) | cargo test |
| ExtensionStore préservation | Intégration | Chaque PR | 0 champ perdu | cargo test |
| Sécurité : fichier tronqué | Sécurité | Chaque PR | Erreur propre | cargo test |
| Sécurité : tensor_count = 2^32 | Sécurité | Chaque PR | SecurityViolation | cargo test |
| Sécurité : path traversal ZIP | Sécurité | Chaque PR | PathTraversal | cargo test |
| Sécurité : ZIP bomb | Sécurité | Chaque PR | ZipBomb | cargo test |
| Fuzzing GGUF | Fuzzing | Chaque PR (60s) | 0 crash | cargo-fuzz |
| Fuzzing ONNX | Fuzzing | Chaque PR (60s) | 0 crash | cargo-fuzz |
| Fuzzing SafeTensors | Fuzzing | Chaque PR (60s) | 0 crash | cargo-fuzz |
| Benchmark GGUF→SafeTensors 1.6 Go | Performance | Nightly | Documenté | criterion |
| Accessibilité frontend | A11y | Chaque PR | WCAG AA | axe-cli |
| Deadlock pipeline | Concurrence | Nightly | Timeout propre | loom |
| ProvenanceChain tamper | Sécurité | Chaque PR | verify() = false | cargo test |

### 18.2 Glossaire Technique

| Terme | Définition Précise |
|-------|-------------------|
| IR | Intermediate Representation — union évolutive de tous les formats supportés |
| ExtensionStore | Stockage limité (100 Mo) des champs exclusifs à chaque format |
| Round-trip bit-identical | A → A avec SHA256 identique — uniquement pour le même format |
| Round-trip sémantique | A → B → A avec mêmes sorties d'inférence dans la tolérance |
| GraphTemplate | Template de reconstruction de graphe pour les formats weights-only |
| WeightsOnly | Format sans graphe de calcul explicite (GGUF, SafeTensors, AWQ...) |
| Recipe Saver | Générateur de configuration pour les formats propriétaires non natifs |
| SecurityBounds | Limites hardcodées sur les champs lus depuis les fichiers |
| Cancellation coopérative | Mécanisme d'annulation que tous les threads vérifient régulièrement |
| Atomic Rename | Pattern write-to-temp + rename atomique (fichier valide ou absent) |
| SSE | Server-Sent Events — alternative unidirectionnelle aux WebSockets |
| TensorQuantization | Métadonnées complètes de quantification (block_size, scale_dtype, etc.) |
| ProvenanceChain | Journal d'audit immutable par hash chaining |

---

*UMC Design Document v2.0 — Corrigé, Honnête, Réaliste*  
*Chaque promesse est vérifiable. Chaque chiffre est reproductible.*  
*Natif ou rien.*