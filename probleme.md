# UMC — Diagnostic Complet et Plan d'Amélioration
## Analyse critique exhaustive · Tous les plans · Toutes les dimensions

> **Type de document :** Audit indépendant et plan d'action  
> **Périmètre :** Architecture technique, backend, frontend, modèle économique, stratégie, sécurité, opérations  
> **Méthode :** Analyse contradictoire — chaque promesse est confrontée à sa faisabilité réelle  
> **Verdict global :** Projet solide et ambitieux avec des failles précises, corrigeables, et priorisables

---

## TABLE DES MATIÈRES

1. [Résumé Exécutif](#1-résumé-exécutif)
2. [Failles Architecturales Critiques](#2-failles-architecturales-critiques)
3. [Failles de l'IR Universelle](#3-failles-de-lir-universelle)
4. [Failles du Pipeline de Conversion](#4-failles-du-pipeline-de-conversion)
5. [Failles de Performance](#5-failles-de-performance)
6. [Failles de Validation et Certification](#6-failles-de-validation-et-certification)
7. [Failles du Backend Distribué](#7-failles-du-backend-distribué)
8. [Failles du Frontend](#8-failles-du-frontend)
9. [Failles du Modèle Économique](#9-failles-du-modèle-économique)
10. [Failles de la Stratégie Go-to-Market](#10-failles-de-la-stratégie-go-to-market)
11. [Failles de Sécurité](#11-failles-de-sécurité)
12. [Failles des Formats Spécifiques](#12-failles-des-formats-spécifiques)
13. [Limites Fondamentales Non Adressées](#13-limites-fondamentales-non-adressées)
14. [Opportunités Manquées](#14-opportunités-manquées)
15. [Plan d'Amélioration Priorisé](#15-plan-damélioration-priorisé)
16. [Matrice de Risques](#16-matrice-de-risques)
17. [Indicateurs de Succès Révisés](#17-indicateurs-de-succès-révisés)

---

## 1. Résumé Exécutif

### Verdict Global

UMC est un projet techniquement cohérent avec une vision stratégique forte. La comparaison avec ffmpeg est juste et inspirante. Mais la documentation actuelle présente **des promesses qui outrepassent la réalité actuelle**, des **angles morts techniques sérieux**, et une **stratégie commerciale qui sous-estime les frictions d'adoption**.

### Score par Dimension

| Dimension | Score | Commentaire |
|-----------|-------|-------------|
| Vision et positionnement | 9/10 | Excellente, différenciée, mémorable |
| Architecture IR | 7/10 | Solide en théorie, des trous en pratique |
| Pipeline de conversion | 7/10 | Bon design, hypothèses de perf. non prouvées |
| Validation et certification | 6/10 | Ambitieuse mais sous-spécifiée |
| Backend distribué | 6/10 | Surcomplex pour le MVP, risques Kafka |
| Frontend | 5/10 | Design réfléchi mais stack non justifiée |
| Modèle économique | 6/10 | Pricing raisonnable, acquisition sous-estimée |
| Sécurité | 4/10 | Lacunes critiques non adressées |
| Stratégie communauté | 8/10 | Bien pensée, exécutable |
| Faisabilité solo/petit équipe | 4/10 | Scope irréaliste pour une équipe de 2 |

### Les 5 Problèmes les Plus Critiques

```
CRITIQUE 1 : La promesse "round-trip bit-identical" est mathématiquement fausse
             pour la majorité des conversions réelles.

CRITIQUE 2 : L'architecture backend est sur-dimensionnée pour le MVP et crée
             une dette technique qui ralentira l'itération.

CRITIQUE 3 : Les benchmarks annoncés (×17, ×18) ne sont pas reproductibles
             dans la documentation actuelle.

CRITIQUE 4 : Le modèle de sécurité pour le parsing de fichiers uploadés
             est dangereux — vecteur d'attaque non adressé.

CRITIQUE 5 : Le scope de 31 formats avec une équipe de 2 personnes
             est irréaliste sur la timeline annoncée.
```

---

## 2. Failles Architecturales Critiques

### 2.1 La Promesse "N+M au lieu de N×M" est une Simplification Trompeuse

**Ce qui est dit dans la doc :**
> "Au lieu de coder N×M = 961 convertisseurs, UMC code N+M = 62 loaders/savers."

**La réalité :**

Ce n'est vrai que si l'IR est *vraiment* un sur-ensemble parfait. Or :

- Chaque format a des **sémantiques d'opérateurs différentes** pour le même nom. L'op `BatchNorm` de PyTorch n'est pas identique à l'op `BatchNorm` d'ONNX. Le loader ne peut pas "juste mapper" — il doit **interpréter**.
- Les **différences de layout mémoire** (NCHW vs NHWC, row-major vs col-major, endianness) nécessitent des conversions spécifiques à chaque paire source→cible, pas seulement source→IR.
- Les **versions de formats** créent une combinatoire cachée : ONNX opset 12 vs 21, GGUF v1/v2/v3, TFLite schema v3/v4. Chaque version est effectivement un format différent.
- Les **formats avec état implicite** (PyTorch `state_dict` avec `_metadata`) ont des conventions non documentées que le mapping IR ne peut pas capturer universellement.

**Conséquence :** En pratique, vous aurez besoin de logique de conversion spécifique à des paires de formats, ce qui rapproche du N×M original pour les cas edge.

**Plan de correction :**
```
□ Documenter explicitement les cas où la conversion IR ne suffit pas
□ Créer une matrice de "conversion loss" par paire de formats
□ Introduire le concept de "conversion hints" (métadonnées supplémentaires
  transmises avec l'IR pour guider le saver cible)
□ Être honnête dans la communication : "N+M pour 80% des cas, 
  avec des adaptateurs spécifiques pour les 20% restants"
```

---

### 2.2 L'ExtensionStore ne Garantit Pas le Round-Trip dans le Cas Général

**Ce qui est dit dans la doc :**
> "∀ A→B→A : résultat bit-identical à l'original"

**La réalité :**

Cette garantie est fausse pour de nombreux cas concrets :

**Cas 1 — Quantification :**
```
GGUF Q4_K_M → ONNX FP16 → GGUF

L'ONNX ne stocke pas les paramètres de quantification Q4_K_M de manière
identique à GGUF. Quand on re-quantifie en Q4_K_M pour revenir à GGUF,
les scales et zero-points seront recalculés différemment.
Résultat : PAS bit-identical. Divergence possible de 1–2%.
```

**Cas 2 — Formats avec compression interne :**
```
PyTorch → ONNX → PyTorch

PyTorch compresse son format avec pickle et peut utiliser différentes
versions de compression. Le round-trip ne reproduit pas le pickle exact.
SHA256 différent. Round-trip ÉCHOUE.
```

**Cas 3 — Formats avec padding/alignement dynamique :**
```
GGUF (alignement 32 octets) → SafeTensors → GGUF

SafeTensors utilise un alignement de 256 octets.
En re-convertissant vers GGUF, l'alignement et le padding interne seront
différents de l'original. SHA256 différent.
```

**Cas 4 — Tenseurs partagés (tied weights) :**
```
Llama a embed_tokens.weight == lm_head.weight (tied).
Certains formats stockent les deux, d'autres une seule fois.
Le round-trip peut dupliquer ou dédupliquer ces tenseurs.
```

**Plan de correction :**
```
□ Changer la garantie : "Round-trip SÉMANTIQUEMENT identique"
  (pas bit-identical sauf pour les formats purement structurels)
□ Définir 3 niveaux de round-trip :
  - NIVEAU 1 : Bit-identical (GGUF→GGUF, SafeTensors→SafeTensors)
  - NIVEAU 2 : Fonctionnellement identique (même sorties d'inférence)
  - NIVEAU 3 : Structurellement identique (même graphe, même architecture)
□ Documenter quel niveau est garanti pour chaque paire de formats
□ Mettre à jour le certificat pour refléter le bon niveau
□ NE PAS promettre bit-identical par défaut dans la communication
```

---

### 2.3 Le ComputeGraph est Inadapté aux Formats "Weights-Only"

**Le problème :**

La majorité des formats LLM (GGUF, SafeTensors, AWQ, GPTQ, bitsandbytes) sont des formats **poids uniquement** — ils ne contiennent pas de graphe de calcul. Or l'IR contient un `ComputeGraph` obligatoire.

Cela crée plusieurs problèmes :
- Pour les conversions GGUF → ONNX, UMC doit **reconstruire le graphe de calcul depuis rien** en se basant sur les métadonnées d'architecture. C'est de la **rétro-ingénierie du modèle**, pas une conversion de format.
- La reconstruction du graphe est spécifique à l'architecture (Llama ≠ Mistral ≠ Phi ≠ Gemma). Chaque architecture nécessite un template de graphe différent.
- Des erreurs dans la reconstruction du graphe produiront des modèles ONNX invalides malgré des checksums de tenseurs corrects.

**Ce que la doc ne dit pas :** Comment UMC reconstruit le graphe ONNX depuis un GGUF qui ne contient que des poids ?

**Plan de correction :**
```
□ Introduire le concept de "GraphTemplate" par architecture connue
□ Maintenir un catalogue de templates : Llama, Mistral, Phi, Gemma, Qwen, etc.
□ Pour les architectures inconnues : mode "weights-only ONNX" avec avertissement
□ Documenter clairement que GGUF→ONNX nécessite la connaissance de l'architecture
□ Intégrer des tests spécifiques : GGUF Mistral → ONNX → vérification du graphe
□ Séparer clairement dans l'IR :
  - ComputeGraph PRÉSENT (ONNX, PyTorch, TFSavedModel)
  - ComputeGraph ABSENT (GGUF, SafeTensors → reconstruction nécessaire)
```

---

### 2.4 Dépendance aux Outils Externes Non Gérée

**Le problème :**

Pour les formats Tier 1 critiques (TensorRT, OpenVINO, TFLite, CoreML), UMC délègue à des outils externes (`trtexec`, `mo`, `tflite_convert`, `coremltools`). La doc mentionne ces outils mais ne traite pas :

- Les **versions incompatibles** : `trtexec` de TensorRT 8 vs TensorRT 10 ont des CLI différentes. `coremltools` v7 vs v8 ont des comportements différents.
- Les **licences restrictives** : TensorRT requiert l'acceptation d'une licence NVIDIA. UMC ne peut pas l'intégrer ni le distribuer automatiquement.
- Les **environnements CI/CD** : dans un container Docker sans GPU, `trtexec` échoue silencieusement ou produit des engines inutilisables.
- L'**indisponibilité sur certaines plateformes** : `coremltools` ne fonctionne que sur macOS pour la compilation (iOS target).
- Le **mode dégradé** : si l'outil externe n'est pas disponible, que se passe-t-il exactement ? La doc dit "UmcError::ExternalToolMissing" mais ne définit pas le comportement de l'API/CLI pour l'utilisateur.

**Plan de correction :**
```
□ Créer une matrice de disponibilité par plateforme pour chaque outil externe
□ Implémenter un système de "capability detection" au démarrage
  umc doctor --check-tools → vérifie et documente ce qui est disponible
□ Mode "offline" explicite : liste les conversions disponibles sans outils externes
□ Versionner les intégrations d'outils externes (ex: "TensorRT 10.x requis")
□ Pour l'API cloud : documenter exactement quels outils sont installés dans les workers
□ Avertissement clair quand l'outil externe n'est pas la dernière version
□ Tests d'intégration CI qui simulent les outils externes manquants
```

---

## 3. Failles de l'IR Universelle

### 3.1 L'ExtensionStore peut Croître de Manière Incontrôlée

**Le problème :**

L'ExtensionStore est conçu pour stocker "tout ce qui ne rentre pas dans l'IR". Sans limites ni gouvernance, cela mène à :

- Un **ExtensionStore de plusieurs gigaoctets** pour les modèles complexes (ONNX avec custom ops embarqués, modèles multimodaux avec métadonnées volumineuses).
- Une **explosion de la mémoire** : l'IR entière doit tenir en mémoire pour que l'ExtensionStore soit consultable pendant la conversion.
- Des **collisions de clés** : si deux formats utilisent la même clé dans l'ExtensionStore (ex: deux formats qui ont chacun un champ `version`), les données se corrompent silencieusement.
- Une **impossibilité de sérialisation** : l'IR avec ExtensionStore ne peut pas être facilement sérialisée/désérialisée pour le checkpointing.

**Plan de correction :**
```
□ Définir une taille maximale pour l'ExtensionStore (ex: 100 Mo par défaut)
□ Utiliser des clés namespaced non-ambiguës :
  format: "GGUF@v3/tokenizer.chat_template" (format + version + chemin)
□ Implémenter l'overflow sur disque pour les grandes extensions
□ Ajouter des métriques : taille totale de l'ExtensionStore dans les logs
□ Mode "--strip-extensions" pour les conversions où le round-trip n'est pas requis
□ Validation des clés à l'insertion (rejeter les clés malformées)
```

---

### 3.2 Les Types de Données Quantifiés Manquent de Métadonnées Critiques

**Le problème :**

L'énumération `DType` liste `Q4KM`, `Q4_0`, etc., mais la quantification bloc-par-bloc nécessite des métadonnées qui vont au-delà du simple type :

- **La taille de bloc** (superblock size) : Q4_K_M utilise des superblocs de 256 éléments contenant 8 blocs de 32. Ce n'est pas encodé dans le DType.
- **Le type de scale** : certains schémas GGUF utilisent des scales en FP16, d'autres en FP32, d'autres encore en Q8_0.
- **L'ordre de stockage** : les poids interleaved vs sequential (AWQ vs GPTQ).
- **Les paramètres de calibration** (pour AWQ/GPTQ) : les zero-points et scales sont calculés sur un dataset de calibration spécifique et ne peuvent pas être "recalculés" lors d'une reconversion.

Sans ces métadonnées dans l'IR, la déquantification sera incorrecte pour de nombreux modèles réels.

**Plan de correction :**
```
□ Enrichir TensorQuantization avec :
  pub struct TensorQuantization {
      pub scheme: QuantScheme,
      pub block_size: usize,          // OBLIGATOIRE
      pub superblock_size: usize,     // Pour GGUF K-quants
      pub scale_dtype: DType,         // Type des scales
      pub zp_dtype: DType,            // Type des zero-points
      pub storage_order: StorageOrder, // interleaved vs sequential
      pub calibration_dataset: Option<String>, // référence (AWQ/GPTQ)
      pub calibration_method: Option<String>,  // "minmax", "percentile", etc.
  }
□ Tests de déquantification sur des modèles réels (vérification numérique)
□ Documenter les cas où la re-quantification est impossible sans les données
  de calibration originales
```

---

### 3.3 ProvenanceChain — Modèle de Sécurité Insuffisant

**Le problème :**

La `ProvenanceChain` est un journal d'audit, mais elle est entièrement modifiable par le code UMC lui-même. N'importe qui peut :

- Créer une ProvenanceChain falsifiée.
- Supprimer des entrées d'une chaîne existante.
- Modifier les hashes SHA256.

Pour une fonctionnalité commercialisée comme "valeur légale", cela est insuffisant.

**Plan de correction :**
```
□ Implémenter une ProvenanceChain immutable via hash chaining :
  entry[n].hash = SHA256(entry[n-1].hash || entry[n].content)
□ Chaque entrée signe cryptographiquement l'entrée précédente
□ Rendre la chaîne vérifiable publiquement via un nœud (hash racine)
□ Définir clairement ce que "valeur légale" signifie et ce qu'il ne signifie pas
  (UMC ne peut pas remplacer un expert judiciaire)
□ Tamper-evident log : toute modification de la chaîne est détectable
```

---

## 4. Failles du Pipeline de Conversion

### 4.1 Le Pipeline 3-Thread peut Créer des Deadlocks

**Le problème :**

Le pipeline Reader → Transformer → Writer utilise des canaux Crossbeam bornés (`bounded(8)`). La capacité de 8 tenseurs dans le canal crée un risque de deadlock dans les situations suivantes :

- **Tenseurs gigantesques bloquants** : si un seul tenseur est trop grand pour être traité dans le `tile_size_bytes`, le Transformer peut bloquer en attendant que le Writer libère de l'espace, pendant que le Writer attend le tenseur suivant du Transformer. **Deadlock circulaire.**
- **Erreur propagée en cascade** : si le Writer rencontre une erreur disque, il ferme son canal de réception. Le Transformer reçoit `ChannelClosed`, envoie `PipelineMessage::Error` — mais si le canal vers le Writer est plein, cet envoi bloque indéfiniment.
- **Cancellation partielle** : si l'utilisateur annule une conversion en cours, aucun mécanisme de cancellation coopérative n'est décrit dans l'architecture actuelle.

**Plan de correction :**
```
□ Utiliser select! de crossbeam pour gérer simultanément :
  - Réception d'un nouveau tenseur
  - Signal de cancellation (CancellationToken de tokio ou channel dédié)
□ Canaux non-bornés avec backpressure explicite (remplacer bounded(8))
□ Timeout sur chaque send/recv (ex: 30 secondes max par opération)
□ Test de deadlock : créer un test unitaire qui simule le scénario de blocage
□ Implémenter un watchdog thread qui tue le pipeline si aucun progrès en N secondes
□ Documentation explicite du comportement en cas d'annulation
```

---

### 4.2 La Configuration Auto-Détectée peut Être Contre-Productive

**Le problème :**

`PipelineConfig::auto()` détermine le nombre de workers en fonction du nombre de CPUs (`num_cpus::get()`). Sur les machines cloud avec 64 vCPUs partagés, cela peut :

- Créer 64 workers qui lisent simultanément 64 shards, saturant le disque réseau S3.
- Utiliser plus de RAM que disponible (64 workers × 64 Mo tile_size = 4 Go minimum).
- Sur les petites machines (Raspberry Pi, VPS 2 Go), créer des OOM immédias.
- Interférer avec d'autres processus sur la machine (aucun `nice` ou limitation de ressources).

**Plan de correction :**
```
□ Implémenter une détection de ressources disponibles (pas juste totales) :
  - RAM disponible (pas RAM totale)
  - Bande passante disque détectée (non supposée)
  - Quota d'I/O dans les environnements containerisés
□ Mode --conservative : réduit le parallélisme pour coexister avec d'autres processus
□ Limiter shard_workers = min(num_cpus, num_shards, ram_gb / 2)
□ Test sur des machines "petit budget" (2 Go RAM, 2 CPUs)
□ Permettre la configuration manuelle avec des valeurs sensées par défaut
□ Ajouter un banner au démarrage : "UMC: utilizing 8/64 vCPUs (auto)"
```

---

### 4.3 Le Streaming depuis URL/S3 n'est pas Concrètement Spécifié

**Le problème :**

La doc mentionne à plusieurs reprises le streaming depuis S3, HTTP, et Hugging Face, mais ne spécifie pas :

- **L'authentification** : credentials AWS, tokens HuggingFace, Basic Auth ?
- **La gestion des rate limits** : Hugging Face rate-limite à ~1 requête/seconde pour les non-authentifiés.
- **Les fichiers shardés distants** : comment UMC gère `model.safetensors.index.json` + 10 shards S3 simultanément ?
- **Le checkpointing réseau** : si la connexion est interrompue à 80%, peut-on reprendre sans re-télécharger depuis le début ?
- **La vérification d'intégrité en streaming** : comment calculer le SHA256 d'un fichier sans le charger entièrement ?

**Plan de correction :**
```
□ Définir une abstraction DataSource avec support authentication :
  pub enum DataSource {
      LocalFile(PathBuf),
      Http { url: String, headers: HashMap<String, String> },
      S3 { bucket, key, credentials: AwsCredentials },
      HuggingFace { repo_id, filename, token: Option<String> },
  }
□ Implémenter le resumable download (Range: bytes=N- header)
□ SHA256 en streaming : utiliser sha2::Sha256 avec update() par chunk
□ Intégration HuggingFace Hub : utiliser l'API officielle (pas juste HTTP)
□ Tests avec des connexions lentes simulées (tc qdisc)
□ Backoff exponentiel pour les rate limits
```

---

## 5. Failles de Performance

### 5.1 Les Benchmarks Annoncés ne sont pas Reproductibles

**Ce qui est dit dans la doc :**
> "×17 plus rapide que llama.cpp, ×12 moins de RAM"

**Le problème :**

Ces chiffres sont présentés comme des faits, mais :

- **Pas de méthodologie publique** : quelle version de llama.cpp ? quels flags de compilation ? quelle machine exactement ?
- **Comparaison biaisée** : llama.cpp convert n'utilise pas de parallélisme agressif par design (pour compatibilité maximale). La comparaison est contre un outil qui ne cherche pas la performance.
- **Le chiffre ×17 vient d'une seule conversion** (GGUF → ONNX, Phi-2 1.6 Go). Ce n'est pas le ratio moyen sur tous les formats.
- **Les benchmarks ONNX → TensorRT incluent le temps de `trtexec`**, un outil C++ NVIDIA qui compile le modèle. Le "temps UMC" dans ce cas est essentiellement le temps de `trtexec`, pas celui d'UMC.
- **Le benchmark "Llama 3.1 405B en 192s"** suppose un NVMe à 7 Go/s et 64 cœurs. Cette configuration coûte ~5000€ et n'est pas représentative de la majorité des utilisateurs.

**Plan de correction :**
```
□ Créer un benchmark public reproducible :
  → Script bash complet avec versions exactes
  → Dockerfile pour environnement reproductible
  → Résultats sur 3 machines différentes (serveur haut de gamme, workstation,
    laptop 16 Go RAM)
□ Comparer contre des adversaires équivalents en effort (pas llama.cpp convert
  qui n'est pas optimisé pour la performance)
□ Présenter les benchmarks par catégorie :
  - Petits modèles (<2 Go) : métriques différentes
  - Modèles moyens (2–20 Go)
  - Grands modèles (>50 Go)
□ Être honnête sur les configurations requises pour atteindre les chiffres annoncés
□ Créer un outil de benchmark intégré : umc benchmark --compare-all
```

---

### 5.2 Le Claim "200 Mo de RAM pour 200 Go de Modèle" est Trompeur

**Ce qui est dit :**
> "Un modèle de 200 Go ne consomme que ~200 Mo de RAM pour ses métadonnées."

**La réalité :**

- **mmap ne signifie pas 200 Mo de RSS** : mmap mappe les pages en adresses virtuelles, mais l'OS chargera les pages en RAM au fur et à mesure des accès. Sur un système avec 32 Go de RAM et un modèle de 200 Go, le kernel utilisera toute la RAM disponible pour le cache des pages mmap.
- **Le "200 Mo" est le métadonnées seulement** : pendant la conversion active (lecture + écriture simultanées), plusieurs dizaines de Go de pages mmap seront en RAM physique.
- **Sur macOS et Windows**, le comportement mmap est différent de Linux : les politiques de cache-invalidation varient, ce qui peut créer des comportements inattendus.
- **La conversion génère un fichier de sortie** qui peut être aussi grand que la source, doublant l'utilisation du cache disque.

**Plan de correction :**
```
□ Clarifier le claim : "UMC utilise ~200 Mo de RAM pour les structures de données
  (hors cache OS)"
□ Documenter le comportement mmap par OS
□ Ajouter une option --memory-limit pour contraindre l'utilisation mmap
□ Mesurer et documenter la RSS réelle (Resident Set Size) pendant une conversion
  réelle d'un modèle de 200 Go
□ Recommander vm.vfs_cache_pressure et d'autres tunings kernel pour les grandes
  conversions
```

---

### 5.3 SIMD non Universel — Risques sur Apple Silicon

**Le problème :**

La configuration `.cargo/config.toml` active `+avx2,+avx512f` pour x86 et `+neon` pour ARM. Mais :

- **AVX-512** n'est pas disponible sur tous les x86_64 (Intel Alder Lake désactive AVX-512 sur les E-cores, AMD Zen 3 ne le supporte pas). Le flag `-C target-feature=+avx512f` globalement causera des **illegal instruction crashes** sur ces CPUs.
- **Apple Silicon (M1/M2/M3)** supporte NEON mais pas certaines variantes SVE utilisées en inférence. Les extensions spécifiques à Apple (AMX) ne sont pas accessibles depuis Rust stable.
- **Les compilations WASM** ne supportent pas SIMD par défaut (requires `wasm32-unknown-unknown` + feature `simd128`).

**Plan de correction :**
```
□ Retirer le flag global AVX-512 de .cargo/config.toml
□ Utiliser la détection runtime via is_x86_feature_detected!() et
  std::arch::is_aarch64_feature_detected!()
□ Implémenter 3 chemins de code :
  - scalar : fonctionnel partout
  - simd_sse4 : x86_64 minimum
  - simd_avx2 : x86_64 avec AVX2
  - simd_avx512 : uniquement si détecté au runtime
□ Tests sur ARM64 (Graviton AWS, Apple M1 via GitHub Actions)
□ Benchmark de la différence réelle : SIMD vs scalaire sur les dtypes cibles
```

---

## 6. Failles de Validation et Certification

### 6.1 La Validation "Fonctionnelle" est Impossible sans Runtime IA

**Ce qui est dit :**
> "Exécution sur 10 entrées aléatoires. Comparaison des sorties couche par couche."

**Le problème :**

Pour exécuter un modèle IA depuis l'IR UMC, il faut un **moteur d'inférence**. La doc mentionne un `IrExecutor` mais ne le définit pas concrètement. Dans la réalité :

- Implémenter un moteur d'inférence Rust capable d'exécuter tous les 142 opérateurs de l'IR (RoPE, FlashAttention, RmsNorm, etc.) est un projet séparé de **plusieurs années**.
- Les runtimes existants (ONNX Runtime, candle, burn) ont chacun leurs limitations.
- La validation "couche par couche" nécessite d'instrumenter le forward pass, ce qui n'est pas trivial.
- Pour un modèle quantifié Q4_K_M, l'exécution nécessite une déquantification spécifique non standard.

**Plan de correction :**
```
□ Court terme : utiliser des runtimes EXTERNES pour la validation fonctionnelle
  - Pour ONNX : ONNX Runtime via subprocess
  - Pour GGUF : llama.cpp via subprocess
  - Pour TFLite : TFLite runtime via subprocess
□ Documenter clairement : "La validation fonctionnelle nécessite l'installation
  du runtime cible"
□ Niveau de validation par défaut : STRUCTURAL (pas FUNCTIONAL)
□ Rendre FUNCTIONAL optionnel et documenté comme "nécessite X"
□ À long terme : intégrer candle (Rust) pour un sous-ensemble d'architectures
□ Définir un IrExecutor minimal qui fonctionne pour Llama-like (les 80% du marché)
```

---

### 6.2 Le Certificat n'a Pas de Valeur Légale Réelle

**Ce qui est dit :**
> "Certificat signé à valeur légale"

**Le problème :**

- Une signature ed25519 prouve que le certificat a été émis par UMC. Elle **ne prouve pas** que la conversion est correcte — elle prouve seulement que UMC a affirmé qu'elle l'était.
- La **clé de signature privée d'UMC** n'est pas dans un HSM (Hardware Security Module). Elle peut être compromise, révoquée, ou oubliée.
- Il n'y a **pas d'infrastructure PKI** (Certificate Authority, revocation lists, OCSP) décrite.
- La **valeur légale** d'une signature ed25519 varie selon les juridictions. Dans la plupart des pays, elle n'est pas reconnue comme signature électronique qualifiée (au sens eIDAS en Europe).
- La FDA n'accepte pas des JSON signés arbitrairement — elle requiert des processus de validation spécifiques (21 CFR Part 11).

**Plan de correction :**
```
□ Renommer "certificat à valeur légale" en "rapport de conversion certifié"
□ Décrire le modèle de confiance clairement :
  "Ce certificat prouve qu'UMC a effectué les validations documentées.
   Il ne garantit pas la correction fonctionnelle du modèle pour votre use case."
□ Pour l'Enterprise : implémenter un vrai PKI avec CA root UMC
□ Pour la conformité FDA : associer UMC à un partenaire validation qualifié
□ Implémenter la révocation de certificats (si un bug est trouvé après coup)
□ Rendre les certificats vérifiables publiquement (endpoint /v1/certificates/:id/verify)
```

---

### 6.3 Les Seuils de Divergence sont Arbitraires

**Ce qui est dit :**
> "F32 → F16 : atol=1e-3, rtol=5e-4 (ULP de FP16 ≈ 9.77e-4)"

**Le problème :**

- Ces seuils sont mathématiquement défendables pour UN tenseur, mais **une accumulation sur 32 couches** peut amplifier l'erreur par un facteur 32× ou plus.
- La **divergence dépend des valeurs** : un modèle avec des poids proches de zéro aura une divergence différente d'un modèle avec des poids dans [-100, +100].
- Le seuil `1e-2` pour la quantification Q4 est **trop permissif** pour les modèles de précision (médical, finance). Il peut être **trop strict** pour les modèles de génération de texte.
- **Aucune mention de la divergence en sortie** (end-to-end divergence) — seule la divergence par tenseur est mesurée.

**Plan de correction :**
```
□ Définir des profils de validation adaptés au use case :
  - PROFILE_STRICT : médical, finance (seuils ×10 plus stricts)
  - PROFILE_STANDARD : usage général
  - PROFILE_PERMISSIVE : prototypage, édge avec contraintes matérielles
□ Ajouter une validation end-to-end optionnelle :
  mesure la divergence sur la sortie finale (logits) pas seulement les poids
□ Documenter la propagation d'erreur : "pour un LLM de 32 couches,
  une divergence de 1e-3 par tenseur peut devenir X en sortie"
□ Permettre aux utilisateurs de définir leurs propres seuils
  --atol 1e-5 --rtol 1e-5
```

---

## 7. Failles du Backend Distribué

### 7.1 Kafka est Sur-Dimensionné pour le MVP

**Le problème :**

L'architecture décrit Kafka comme "colonne vertébrale" dès le départ. Kafka est un excellent choix à grande échelle, mais pour un MVP :

- **Complexité opérationnelle** : Kafka nécessite Zookeeper (ou KRaft), au moins 3 brokers pour la haute disponibilité, une configuration réseau précise. C'est 40–80 heures de setup et maintenance.
- **Coût infrastructure** : un cluster Kafka minimal (3 brokers, 3 Zookeeper) coûte ~500€/mois sur AWS.
- **Latence supplémentaire** : publier un message Kafka + le consommer ajoute 10–50ms de latence par conversion, imperceptible à grande échelle mais visible pour les petits modèles.
- **Aucun besoin pour les 1000 premiers clients** : PostgreSQL avec SKIP LOCKED suffit pour des files d'attente jusqu'à des dizaines de milliers de jobs.

**Plan de correction :**
```
□ PHASE MVP (0–1000 clients) : utiliser PostgreSQL + SKIP LOCKED
  CREATE TABLE jobs (id UUID, status TEXT, payload JSONB, created_at TIMESTAMPTZ);
  SELECT * FROM jobs WHERE status='queued' LIMIT 1 FOR UPDATE SKIP LOCKED;
□ PHASE SCALE (1000–10000 clients) : migrer vers Redis Streams ou BullMQ
□ PHASE HYPERSCALE (>100000 clients) : migrer vers Kafka
□ Documenter le chemin de migration dans la roadmap technique
□ Économiser ~500€/mois en infrastructure pour réinvestir dans le développement
```

---

### 7.2 Le Modèle WebSocket Crée des Problèmes de Scalabilité

**Le problème :**

Le design décrit :
> "Le worker publie sa progression sur un canal Redis PubSub. Le serveur WebSocket s'y abonne."

Ce modèle fonctionne, mais :

- **Sticky sessions nécessaires** : le WebSocket d'un client est connecté à un serveur spécifique. Si ce serveur redémarre, la connexion WebSocket est perdue.
- **Redis PubSub non durable** : si le client est déconnecté pendant 2 secondes et se reconnecte, les messages de progression envoyés pendant cette fenêtre sont perdus.
- **Scalabilité des WebSockets** : chaque connexion WebSocket maintient un file descriptor ouvert. À 10000 connexions simultanées, cela requiert une configuration OS spécifique (ulimit -n).
- **Le protocole de reconnexion** n'est pas défini : que fait le client quand la connexion WebSocket est interrompue ?

**Plan de correction :**
```
□ Implémenter Server-Sent Events (SSE) comme alternative plus simple aux WebSockets
  pour la progression (unidirectionnel, pas de sticky session)
□ Stocker la progression dans Redis avec TTL (pas seulement PubSub)
  SET job:{id}:progress 0.64 EX 3600
□ Le client peut polluer /v1/jobs/:id pour rattraper la progression manquée
□ Implémenter une reconnexion automatique côté client avec backoff exponentiel
□ Définir clairement le format du message de progression et le protocole de reprise
```

---

### 7.3 Le Checkpointing "Toutes les 30 Secondes" est Insuffisant

**Le problème :**

Pour un modèle de 810 Go qui met 3 minutes à convertir, un checkpoint toutes les 30 secondes implique :

- **6 checkpoints** au total
- En cas de crash, on reprend depuis le dernier checkpoint (~30s perdues max)
- **Mais** : réécrire l'offset dans Redis toutes les 30 secondes crée 6 writes/conversion. Pour 1M de conversions simultanées = 6M writes/min dans Redis. C'est faisable mais doit être dimensionné.

Le problème plus fondamental : **la reprise nécessite que le fichier de sortie soit en append-only et seekable**. ONNX protobuf ne permet pas facilement l'écriture incrémentale. Si le Writer a écrit 80% d'un fichier ONNX et crashe, le fichier est invalide (le footer protobuf manque).

**Plan de correction :**
```
□ Utiliser le pattern "write-to-temp + atomic rename" :
  1. Écrire dans /tmp/umc-{job_id}-{timestamp}.tmp
  2. Checkpoint = enregistrer l'offset dans le .tmp
  3. En cas de reprise, continuer d'écrire dans le .tmp
  4. À la fin : atomic rename .tmp → cible finale
□ Pour ONNX : écrire les tenseurs d'abord, le header en dernier
  (ONNX supporte ce pattern)
□ Format de checkpoint minimal :
  {job_id, last_tensor_name, output_offset, tensors_done, bytes_done}
□ Tester la reprise : tuer le process en cours de conversion et vérifier
  la reprise correcte
```

---

## 8. Failles du Frontend

### 8.1 Le Stack Frontend est Sur-Spécifié sans Justification

**Le problème :**

La doc spécifie exactement :
- Next.js 15 (App Router)
- Framer Motion
- Zustand
- TanStack Query
- Recharts
- D3.js pour le graphe interactif

Ce stack total représente **~3 Mo de JS** (gzippé), 6+ librairies à maintenir, et des versions qui changent fréquemment. Des problèmes spécifiques :

- **Next.js 15 + App Router** : encore en maturation, breaking changes fréquents, documentation partielle.
- **D3.js + graphe 31 nœuds** : complexité élevée pour un graphe qui peut être rendu avec une librairie plus simple (Cytoscape.js, React Flow).
- **Framer Motion** pour les animations : lourde (200 Ko gzippé) pour des animations qui peuvent être faites en CSS pur.
- **Zustand + TanStack Query** : double gestion d'état (local + serveur). À surveiller pour les conflits.
- Le thème **"Belgian Yellow"** (#FFD700) sur fond **#0D0D0D** : contraste 11:1 ✅ mais uniquement pour le texte. Les boutons primaires en jaune avec texte sombre (#08090B) — vérifier le contraste du texte sur ce fond précis.

**Plan de correction :**
```
□ MVP Frontend : Next.js 14 (stable) + Tailwind CSS + Vanilla JS pour les
  animations simples
□ Graphe de conversion : React Flow (plus accessible pour un développeur unique)
  ou même une visualisation SVG statique pour le MVP
□ Remplacer Framer Motion par CSS transitions pour 80% des animations
□ Retarder D3.js jusqu'à ce que le besoin soit prouvé par des utilisateurs
□ Bundle size target : < 100 Ko gzippé pour la page principale
□ Tester le contraste Belgian Yellow avec les outils d'accessibilité réels
  (pas seulement le ratio calculé)
```

---

### 8.2 L'UX du Certificat Sous-Estime les Besoins Enterprise

**Le problème :**

Le certificat est présenté comme un fichier JSON téléchargeable. Pour les clients Enterprise (healthcare, finance), ce n'est pas suffisant :

- Ils ont besoin d'intégrer le certificat dans leurs **systèmes de gestion documentaire** (SharePoint, Confluence, JIRA).
- Ils ont besoin d'une **URL permanente et stable** pour référencer le certificat dans leurs audits.
- Ils ont besoin d'un **format PDF** signé électroniquement (pas juste JSON) pour les auditeurs non-techniques.
- Ils ont besoin de **webhooks** pour être notifiés quand un certificat est émis.
- La révocation d'un certificat (si un bug est trouvé) doit être **proactive** (notification email) pas passive.

**Plan de correction :**
```
□ Endpoint public permanent : GET /certificates/{id} → JSON (accessible sans auth)
□ Génération PDF : endpoint GET /certificates/{id}/pdf avec signature PDF
□ Webhook : POST sur URL configurée quand conversion + certificat terminés
□ Révocation : endpoint POST /certificates/{id}/revoke (Enterprise seulement)
□ Intégration SIEM : export des certificats en SYSLOG format (Enterprise)
□ Retention configurable : 30 jours (Pro), 7 ans (Enterprise, requis FDA/finance)
```

---

## 9. Failles du Modèle Économique

### 9.1 Le Pricing "Pay-as-You-Go" Crée un Problème d'Unité

**Le problème :**

```
0,002 €/conversion (≤ 5 Go)
0,010 €/conversion (> 5 Go)
```

Ces prix sont **trop bas** pour être durables et **trop hauts** pour être compétitifs :

- **Trop bas** : une conversion de Llama 3.1 405B (810 Go) coûte 0,010 € mais consomme ~3 minutes de 64 CPUs + bande passante S3. Coût infrastructure : ~0,50 €. **Marge négative de ×50.**
- **Trop hauts** : un développeur qui fait 50 conversions/jour paie 3 €/jour = 90 €/mois. Il prendra le plan Pro (19 €/mois) dès le deuxième jour.
- **L'unité "conversion" est mal définie** : convertir un modèle de 100 Mo et un de 100 Go coûtent pareil ?
- **Pas de pricing pour les conversions qui échouent** : si une conversion échoue à 90%, est-elle facturée ?

**Plan de correction :**
```
□ Repriser le pay-as-you-go sur la taille du modèle :
  - < 500 Mo : 0,001 € (petits modèles, tests)
  - 500 Mo – 5 Go : 0,005 €
  - 5 Go – 50 Go : 0,02 €
  - > 50 Go : 0,05 € (couvre les coûts infrastructure)
□ Définir "conversion" : un job qui aboutit à un fichier de sortie valide
□ Les conversions échouées avant 10% ne sont pas facturées
□ Calculer le coût infrastructure réel avant de fixer le prix
□ Modèle alternatif : facturer à la "Go convertie" (ex: 0,001 €/Go)
  plus prévisible pour l'utilisateur et aligné sur les coûts réels
```

---

### 9.2 UMC Hub — Problème de Droits sur les Modèles

**Le problème :**

Le Hub UMC propose des "modèles populaires pré-convertis dans tous les formats". Cela implique :

- **Stocker des copies de modèles propriétaires** (Llama 3, Gemma, etc.) qui ont des licences spécifiques (Meta LLAMA 3 Community License, Google Gemma Terms).
- La plupart de ces licences **interdisent la redistribution commerciale** sans accord explicite.
- Stocker 30 formats × 100 modèles populaires = des **pétaoctets de stockage** S3.
- Si un modèle est mis à jour (sécurité, fine-tuning), UMC doit re-convertir et re-stocker toutes les versions.

**Plan de correction :**
```
□ Consultation légale OBLIGATOIRE avant le lancement du Hub
□ Modèle alternatif : UMC Hub ne stocke pas les fichiers — il stocke des
  "recettes de conversion" (configuration UMC pour chaque modèle HuggingFace)
  L'utilisateur déclenche la conversion, UMC récupère depuis HF et convertit
□ Partenariat officiel avec HuggingFace pour les modèles sous licence permissive
□ Commencer uniquement avec des modèles en licence Apache 2.0 / MIT
  (TinyLlama, Phi-2, etc.)
□ Cache temporaire côté utilisateur, pas stockage permanent
```

---

### 9.3 La Stratégie d'Acquisition Client est Sous-Estimée

**Le problème :**

La doc suppose que "publier sur r/MachineLearning + HN = 2000 stars + 500 signups". L'expérience réelle :

- r/MachineLearning reçoit **50+ posts de qualité par semaine**. Un post sans démo fonctionnelle et sans benchmarks vérifiables sera ignoré.
- HN (Show HN) a un **taux de succès < 5%** pour les posts de projets open source.
- La conversion GitHub stars → utilisateurs actifs est de **~1%** (2000 stars = 20 utilisateurs actifs).
- Le CAC (Customer Acquisition Cost) pour un outil MLOps B2B est de **200–2000 €** selon les études du marché. Le budget marketing de 2000 €/mois couvre 1–10 clients Pro.

**Plan de correction :**
```
□ Stratégie de contenu plus réaliste :
  - 1 article technique exhaustif publié sur HN = 500-2000 vues
  - Nécessite une démo interactive fonctionnelle dès le premier post
  - Intégrer la démo dans la landing page (pas "coming soon")
□ Canal alternatif sous-exploité : intégrations directes dans les outils existants
  - PR dans Ollama, LM Studio, Jan.ai avec UMC comme backend de conversion
  - Ces PR = exposition à 500K+ utilisateurs sans coût marketing
□ Réviser les projections :
  - Mois 6 : 500 GitHub stars (pas 2000), 50 signups (pas 500)
  - Mois 12 : 3000 stars, 200 signups
□ Priorité absolue au Product-Led Growth : rendre l'outil tellement bon
  que les ingénieurs le partagent spontanément
```

---

## 10. Failles de la Stratégie Go-to-Market

### 10.1 La Roadmap 31 Formats est Irréaliste pour une Petite Équipe

**Le problème :**

La roadmap actuelle prévoit 31 formats en 12 sprints. En pratique :

- **Un loader sérieux** prend 2–4 semaines de développement (lecture de spec + implémentation + tests + débug sur modèles réels).
- **Les tests round-trip** nécessitent des fichiers de test réels pour chaque format, qui doivent être collectés, validés, et stockés.
- **Chaque format évolue** : GGUF v4 sera publié, ONNX opset 22 sortira, TensorRT 11 changera ses APIs. Maintenance continue.
- Une équipe de 2 personnes peut maintenir sérieusement **8–12 formats maximum**.

**Ce que cela signifie :** En essayant de couvrir 31 formats avec 2 personnes, chaque format sera implémenté superficiellement. C'est exactement ce qu'UMC dit vouloir éviter.

**Plan de correction :**
```
□ Réduire le MVP à 5 formats maximaux :
  GGUF ↔ ONNX ↔ SafeTensors + PyTorch + TFSavedModel
  (couvre 90% des cas d'usage réels)
□ Ouvrir les autres formats à la communauté via :
  - Programme Bounty clairement financé (2000 € par format Tier 1)
  - Plugin system solide qui permet aux contributeurs d'ajouter des formats
  - Template de loader/saver avec 200 lignes de boilerplate
□ Être honnête dans la communication sur la timeline
□ Marquer clairement les formats "community-maintained" vs "core-maintained"
□ Concentrer les 2 core developers sur l'excellence de l'IR et du pipeline,
  pas sur la quantité de formats
```

---

### 10.2 La Compétition avec GGUF et llama.cpp est Asymétrique

**Le problème :**

llama.cpp est maintenu par Georgi Gerganov avec ~2000 contributeurs et bénéficie de :
- Un momentum communautaire massif (60K+ GitHub stars)
- Une intégration profonde dans l'écosystème Ollama/LM Studio
- Un format GGUF qu'il contrôle — s'il ajoute une fonctionnalité de conversion dans llama.cpp, UMC perd son avantage sur ce chemin spécifique.

Hugging Face dispose des ressources pour développer son propre outil de conversion et a tous les modèles, toute la communauté, et une infrastructure cloud.

ONNX Runtime (Microsoft) a déjà un pipeline de conversion et est maintenu par une équipe de 50+ ingénieurs.

**Plan de correction :**
```
□ Ne pas positionner UMC contre llama.cpp — positionner UMC comme
  COMPLÉMENTAIRE (UMC est la couche de conversion, llama.cpp est le runtime)
□ Intégrer UMC dans llama.cpp comme outil de conversion recommandé
  (PR officielle, pas compétition)
□ Identifier le "wedge market" : secteur industriel où les concurrents sont absents
  Healthcare → UMC + certification FDA-compatible
  Automotive → UMC + certifications ISO 26262
□ Construire des partenariats AVANT d'être en compétition
□ Si HuggingFace veut intégrer UMC : le permettre activement (ne pas avoir peur
  de se faire "racheter" par un partenariat)
```

---

## 11. Failles de Sécurité

### 11.1 Parsing de Fichiers Non Fiables — Surface d'Attaque Critique

**Le problème — SÉVÉRITÉ HAUTE :**

UMC accepte des fichiers de modèle uploadés par des utilisateurs anonymes et les parse en Rust. Les formats de modèles sont des formats binaires complexes, sources historiques de vulnérabilités :

- **ONNX/Protobuf** : les parsers protobuf ont eu des CVE récents (parsing de messages malformés, stack overflow sur récursion infinie).
- **PyTorch pickle** : les fichiers `.pt/.pth` sont des pickles Python. Même en Rust, parser du pickle peut exposer à des gadget chains si des types imprévus sont rencontrés.
- **GGUF** : le champ `tensor_count` est lu depuis l'utilisateur. Si `tensor_count = 2^32`, UMC essaie d'allouer un Vec avec 4 milliards d'éléments → OOM/DoS.
- **Zip (PyTorch, TorchScript)** : ZIP bombs, path traversal dans les noms de fichiers (`../../../etc/passwd`).
- **mmap d'un fichier malveillant** : si le fichier est modifié pendant le mmap (TOCTOU attack), les données lues peuvent changer en cours d'utilisation.

La doc mentionne "valider les tailles déclarées" mais pas :
- Fuzzing des parsers
- Sandboxing du parsing
- Limites sur la récursion/imbrication

**Plan de correction — URGENT :**
```
□ IMMÉDIAT : Fuzzing de tous les loaders avec cargo-fuzz ou AFL++
  Priorité : GGUF, ONNX, PyTorch
□ IMMÉDIAT : Limites hardcodées sur TOUS les champs numériques lus depuis le fichier :
  - tensor_count : max 1_000_000
  - metadata_kv_count : max 10_000
  - string_length : max 1_024 * 1_024 (1 Mo)
  - nested_depth (protobuf) : max 32
□ Sandboxing du parsing avec seccomp-bpf (Linux) ou pledge (OpenBSD)
□ Parsing PyTorch pickle : whitelist des types autorisés UNIQUEMENT
  (jamais de classe Python arbitraire)
□ Protection ZIP bomb : limiter le ratio de compression et la taille décompressée
□ Path traversal : normaliser tous les chemins extraits des archives
□ TOCTOU : ne pas re-lire le fichier après le mmap initial, utiliser le mmap
□ Audit de sécurité tiers avant le lancement de l'API publique
```

---

### 11.2 Les API Keys ne sont pas Assez Sécurisées

**Ce qui est dit :**
> "Les API keys sont hachées (bcrypt ou argon2) en base de données"

**Le problème :**

- Les API keys ont besoin d'être cherchées en base pour chaque requête. bcrypt/argon2 sont conçus pour être **lents** (protection contre le brute-force). Hasher avec bcrypt chaque requête = 50–100ms ajoutés à chaque call API.
- Le standard industriel pour les API keys est **HMAC-SHA256** ou **SHA256 salté**, pas bcrypt.
- Il n'est pas mentionné de **rate limiting par API key** (distinct du rate limiting par IP).
- Les API keys devraient avoir une **date d'expiration** et la possibilité de **rotation**.
- L'**audit log** des appels API par key n'est pas décrit.

**Plan de correction :**
```
□ Utiliser le schéma standard :
  1. Générer la clé : umc_sk_prod_<32 bytes random hex>
  2. Stocker en DB : SHA256(clé) (rapide à chercher)
  3. Afficher à l'utilisateur une seule fois au moment de la création
□ Rate limiting par API key en mémoire (pas base de données) avec Redis
□ API keys avec expiration optionnelle et rotation automatique
□ Audit log : chaque appel API loggué avec (key_id, endpoint, timestamp, ip)
□ Scopes par API key : read-only, convert, admin
□ Alerte automatique si une clé est utilisée depuis plus de 3 pays différents
```

---

### 11.3 Injection de Commandes via les Noms de Fichiers

**Ce qui est dit dans la doc :**
> "Les commandes sont construites avec des listes d'arguments (jamais d'interpolation de shell)"

**Le problème :**

La règle est bonne, mais il y a des angles morts :

- **Les noms de fichiers passés aux outils externes** : si le fichier s'appelle `model; rm -rf /; .gguf`, et qu'UMC utilise `Command::new("trtexec").arg(format!("--onnx={}", filename))` au lieu de `.arg("--onnx").arg(filename)`, il y a injection.
- **Les chemins de sortie** contrôlés par l'utilisateur via l'API peuvent pointer vers des destinations non autorisées.
- **Les URLs streaming** peuvent pointer vers des ressources internes (SSRF - Server-Side Request Forgery) : `http://169.254.169.254/` (AWS metadata service).

**Plan de correction :**
```
□ Valider TOUS les noms de fichiers avec une whitelist de caractères autorisés
  Regex : ^[a-zA-Z0-9_\-\.\/]+$
□ Toujours passer les noms de fichiers comme arguments séparés, jamais interpolés
□ Pour les URLs streaming : whitelist des domaines autorisés
  Blacklister 169.254.0.0/16, 10.0.0.0/8, 172.16.0.0/12, 127.0.0.0/8 (SSRF)
□ Chemins de sortie : forcer vers un répertoire contrôlé (/tmp/umc-outputs/)
□ Test de pénétration spécifique sur les inputs noms de fichiers
```

---

## 12. Failles des Formats Spécifiques

### 12.1 Diffusers n'est pas un Format Simple

**Le problème :**

Diffusers est décrit comme un format (Tier 3) mais c'est en réalité une **convention de répertoire HuggingFace** qui peut contenir :

- 2 à 10 sous-modèles différents (UNet, VAE, Text Encoder 1, Text Encoder 2, Image Encoder, etc.)
- Chaque sous-modèle dans un format différent (SafeTensors, PyTorch, ONNX selon la version)
- Des configurations JSON complexes par sous-modèle
- Des tokenizers multiples (CLIP, T5, etc.)
- Des schedulers avec états internes

La doc dit "charger chaque composant avec son loader approprié" mais ne définit pas :
- Comment déterminer quelle version de Diffusers est utilisée (SD 1.4 vs SD 2.1 vs SDXL vs SD3 vs Flux)
- Comment convertir le pipeline entier vers ONNX (nécessite de connecter les graphes des sous-modèles)
- Comment gérer les formats mixtes dans un seul Diffusers repo

**Plan de correction :**
```
□ Traiter Diffusers comme un "format de niveau 2" : conversion d'abord vers
  SafeTensors (par sous-modèle), puis vers le format cible
□ Implémenter des détecteurs spécifiques par version :
  - SD1.x, SD2.x, SDXL, SD3.x, Flux, Wan, etc.
□ Conversion Diffusers → ONNX : nécessite de créer un graphe ONNX composite
  (hors scope du MVP)
□ Documenter clairement les limitations :
  "Diffusers → GGUF non supporté (nécessite un runtime de diffusion)"
□ Commencer par : Diffusers → SafeTensors (conversion par sous-modèle uniquement)
```

---

### 12.2 ONNX Web et WASM — Limitations Non Documentées

**Le problème :**

"ONNX Web / WebGPU" est listé comme Tier 3, mais :

- ONNX Web est un runtime web qui utilise WebAssembly et/ou WebGPU. Le "format" de sortie n'est pas un seul fichier mais un bundle : `.onnx` + `.wasm` + `.js` + configuration.
- WebGPU n'est pas disponible dans tous les navigateurs (Safari partiel, Firefox expérimental en 2024).
- Les modèles ONNX pour le web doivent être **quantifiés et optimisés** (les grands modèles sont trop lents dans le navigateur).
- La taille maximale pratique est ~500 Mo pour une inférence web raisonnable.

**Plan de correction :**
```
□ Renommer "ONNX Web" en "ONNX Runtime Web Bundle" dans la documentation
□ Documenter les contraintes dures :
  - Taille max recommandée : 200 Mo (expérience utilisateur acceptable)
  - Quantification obligatoire : INT8 ou FP16
  - Navigateurs supportés : Chrome 113+, Edge 113+, Firefox (limité), Safari (limité)
□ Le "saver" produit un répertoire, pas un fichier unique
□ Tester sur les navigateurs cibles avant d'annoncer le support
□ Avertissement automatique si le modèle > 200 Mo : "Ce modèle sera lent dans
  les navigateurs. Considérez la quantification."
```

---

## 13. Limites Fondamentales Non Adressées

### 13.1 Les Modèles Multimodaux

Les modèles multimodaux (Llava, Flamingo, CLIP, Whisper, etc.) combinent des encodeurs vision, audio, texte dans un seul modèle. L'IR actuelle suppose implicitement un seul graphe linéaire. Ces modèles nécessitent :

- Plusieurs sous-graphes avec des inputs de types différents (tenseurs image + tenseurs texte)
- Des encodeurs séparés avec des dtypes différents
- Des mécanismes de fusion spécifiques à l'architecture

**Impact :** UMC ne peut pas convertir LLava, Phi-3-Vision, Gemini (multimodal) correctement sans modifications majeures de l'IR.

---

### 13.2 Les Modèles avec Custom CUDA Kernels

Certains modèles utilisent des CUDA kernels personnalisés (Flash Attention 2, Triton kernels, etc.) qui ne sont pas représentables par les opérateurs standard d'ONNX. Ces kernels doivent soit :

- Être décomposés en opérateurs standard (peut perdre 50–80% de performance)
- Être représentés comme `Custom` ops (format cible ne peut pas les exécuter)

La doc reconnaît les `Custom` ops mais ne décrit pas la politique pour les modèles massivement dépendants de ces ops (ex: toute la famille Mistral avec Flash Attention 2).

---

### 13.3 Les Modèles de Génération d'Images (Diffusion)

Les modèles de diffusion (Stable Diffusion, Flux, Wan) ont des architectures radicalement différentes des LLMs :

- Boucles d'inférence itératives (pas un seul forward pass)
- Schedulers stateful (DDPM, DDIM, etc.)
- Modèles nécessitant plusieurs appels séquentiels (text encoder → denoiser → VAE decoder)

L'IR actuelle est conçue pour des forward passes uniques. La conversion de ces modèles est fondamentalement différente.

---

### 13.4 Les Modèles sur GPU Fragmentés (Tensor Parallelism)

Les très grands modèles (405B+) sont souvent shardés non seulement en fichiers mais en **tensor parallelism** (chaque GPU contient une fraction de chaque tenseur). Ces modèles nécessitent des opérations `AllReduce` et des annotations de parallélisme que l'IR actuelle ne représente pas.

---

## 14. Opportunités Manquées

### 14.1 UMC comme Outil de Validation de Modèles (pas juste de Conversion)

L'infrastructure de validation numérique d'UMC est intrinsèquement précieuse au-delà de la conversion :

```
Opportunité : umc validate model.onnx --reference model.pt
              → Valider qu'une mise à jour d'un modèle n'a pas
                introduit de régression numérique

Opportunité : umc audit model.gguf
              → Détecter des patterns suspects (backdoors simples,
                poids anormalement élevés, tenseurs corrompus)

Opportunité : umc diff model-v1.onnx model-v2.onnx
              → Comparer deux versions du même modèle
```

**Plan :**
```
□ Développer umc validate comme produit standalone
□ Intégrer dans les pipelines CI/CD des équipes ML :
  "Aucune régression numérique entre les versions du modèle"
□ C'est un produit Enterprise à part entière (audit, conformité)
```

---

### 14.2 Format UMC Natif comme Standard Intermédiaire

Plutôt que de simplement convertir entre formats existants, UMC pourrait définir son propre format IR sérialisable :

```
UMC Format (.umcm) :
- L'IR sérialisée directement
- Contient TOUT (graphe + poids + tokenizer + config)
- Peut être converti vers n'importe quel format cible
- Optimisé pour le streaming et le checkpointing
```

Ce format serait :
- Le format de stockage intermédiaire pour les grandes conversions
- Un format d'échange universel entre équipes
- La base du UMC Hub (stocker en .umcm, convertir à la demande)

---

### 14.3 UMC CLI comme Plugin pour les IDEs

VS Code, JetBrains, et Cursor ont des millions d'utilisateurs ML. Un plugin UMC qui permet :

- Clic droit sur un modèle → "Convert with UMC..."
- Inspection inline des métadonnées du modèle
- Diffing visuel de deux versions de modèles

Ce canal d'acquisition est gratuit et ciblé.

---

### 14.4 Intégration avec les Registres de Modèles

Clearml, MLflow, Weights & Biases, Comet ML — ces outils gèrent les expériences ML mais pas la conversion de format. Une intégration native UMC permettrait :

- "Track model conversion" dans MLflow
- Conversion automatique à la fin d'un run d'entraînement
- Versioning du modèle converti dans le registre

Marché d'intégration sous-exploité avec un accès direct aux ingénieurs ML.

---

## 15. Plan d'Amélioration Priorisé

### Niveau P0 — Corrections Critiques (Semaines 1–4)

Ces corrections doivent être faites AVANT tout lancement public.

```
P0.1 — SÉCURITÉ : Fuzzing des loaders GGUF, ONNX, PyTorch
  Effort : 1 semaine
  Impact : évite des vulnérabilités critiques sur l'API publique
  Responsable : dev backend

P0.2 — SÉCURITÉ : Limites hardcodées sur tous les champs numériques des parsers
  Effort : 2 jours
  Impact : prévient les DoS via modèles malformés

P0.3 — ARCHITECTURE : Clarifier et corriger la promesse round-trip
  Effort : 1 jour (doc + code)
  Impact : évite des promesses fausses qui détruisent la confiance

P0.4 — ARCHITECTURE : Documenter l'exigence de GraphTemplate pour GGUF→ONNX
  Effort : 3 jours
  Impact : évite des bugs silencieux sur les modèles réels

P0.5 — PIPELINE : Corriger le risque de deadlock dans le pipeline 3-thread
  Effort : 3 jours
  Impact : stabilité de l'outil de base
```

---

### Niveau P1 — Améliorations Majeures (Mois 1–3)

```
P1.1 — BACKEND : Remplacer Kafka par PostgreSQL+SKIP LOCKED pour le MVP
  Effort : 1 semaine
  Impact : -70% complexité infrastructure, -500€/mois

P1.2 — VALIDATION : Implémenter la validation fonctionnelle via runtime externe
  Effort : 2 semaines
  Impact : rend le niveau FUNCTIONAL réellement fonctionnel

P1.3 — BENCHMARKS : Créer un suite de benchmarks reproductibles publics
  Effort : 1 semaine
  Impact : crédibilité des chiffres annoncés

P1.4 — QUANTIFICATION : Enrichir TensorQuantization avec les métadonnées manquantes
  Effort : 1 semaine
  Impact : déquantification correcte sur les modèles réels

P1.5 — SCOPE : Réduire le MVP à 5 formats et documenter le plan de contributions
  Effort : 1 jour (décision + doc)
  Impact : livrer un produit excellent plutôt que 31 formats médiocres
```

---

### Niveau P2 — Optimisations et Fonctionnalités Manquantes (Mois 3–6)

```
P2.1 — FRONTEND : Réduire le stack, utiliser Next.js 14 stable
  Effort : 1 semaine

P2.2 — CERTIFICAT : Implémenter le modèle de confiance correct (pas "valeur légale")
  Effort : 3 jours

P2.3 — STREAMING : Spécifier et implémenter le streaming S3/HTTP complet
  Effort : 2 semaines

P2.4 — SIMD : Corriger la détection runtime AVX-512 (retirer le flag global)
  Effort : 2 jours

P2.5 — PRICING : Réviser le modèle Pay-as-You-Go pour la viabilité
  Effort : 1 jour (décision)

P2.6 — HUB LÉGAL : Consultation juridique sur les droits de redistribution
  Effort : 2 semaines (processus externe)

P2.7 — PIPELINE : Implémenter la cancellation coopérative
  Effort : 3 jours
```

---

### Niveau P3 — Améliorations Long Terme (Mois 6–12)

```
P3.1 — FORMAT .umcm : Définir et implémenter le format IR natif sérialisable
P3.2 — MULTIMODAL : Étendre l'IR pour les modèles multimodaux
P3.3 — VALIDATION : Développer umc validate comme produit standalone
P3.4 — IDE PLUGINS : Créer un plugin VS Code/JetBrains
P3.5 — PKI : Implémenter une vraie infrastructure de certificats
P3.6 — MLFLOW/W&B : Intégrations avec les registres de modèles
```

---

## 16. Matrice de Risques

| Risque | Probabilité | Impact | Score | Mitigation |
|--------|-------------|--------|-------|------------|
| Vulnérabilité sécurité dans un parser | HAUTE | CRITIQUE | 🔴 P0 | Fuzzing immédiat |
| Round-trip promis mais non livré | HAUTE | HAUTE | 🔴 P0 | Corriger la promesse |
| GGUF→ONNX sans graphe valide | HAUTE | HAUTE | 🔴 P0 | GraphTemplate obligatoire |
| Deadlock pipeline | MOYENNE | HAUTE | 🟠 P1 | Correction architecture |
| Scope trop large pour l'équipe | HAUTE | HAUTE | 🟠 P1 | Réduire MVP |
| Droits sur les modèles du Hub | HAUTE | HAUTE | 🟠 P1 | Consultation légale |
| Kafka trop complexe pour MVP | HAUTE | MOYENNE | 🟠 P1 | Simplifier infra |
| Benchmarks non reproductibles | HAUTE | MOYENNE | 🟠 P1 | Suite publique |
| Compétition HuggingFace | MOYENNE | HAUTE | 🟡 P2 | Partenariat > compétition |
| SIMD crash sur CPUs sans AVX-512 | HAUTE | MOYENNE | 🟡 P2 | Détection runtime |
| Certificats sans valeur légale réelle | HAUTE | BASSE | 🟡 P2 | Renommer + clarifier |
| Pricing non viable (grands modèles) | HAUTE | MOYENNE | 🟡 P2 | Réviser unité de facturation |
| OOM sur petites machines | MOYENNE | MOYENNE | 🟡 P2 | Tests sur configs limitées |

---

## 17. Indicateurs de Succès Révisés

### Métriques Techniques (plus réalistes)

| Métrique | Objectif Original | Objectif Révisé | Raison |
|----------|-------------------|-----------------|--------|
| Formats supportés (Mois 6) | 31 | 5 (GGUF, ONNX, SafeTensors, PyTorch, TF) | Qualité > quantité |
| GitHub Stars (Mois 6) | 2 000 | 500–1 000 | Distribution réaliste |
| Conversion GGUF→ONNX Phi-2 | < 5s | < 10s (acceptable) | Benchmark plus conservateur |
| RAM modèle 8B | 1.2 Go | < 4 Go (acceptable) | Cache OS non contrôlable |
| Round-trip GGUF→ONNX→GGUF | bit-identical | fonctionnellement identique | Réalité mathématique |
| Tests automatiques | > 90% couverture | > 80% (MVP) | Pragmatisme |

### Métriques Business (plus réalistes)

| Métrique | Objectif Original | Objectif Révisé | Raison |
|----------|-------------------|-----------------|--------|
| Clients Pro (Mois 12) | 400 | 50–100 | CAC réaliste |
| ARR (Mois 12) | 100 K€ | 20–30 K€ | Traction réaliste |
| Conversions/mois (Mois 6) | 50 000 | 5 000–10 000 | Base utilisateurs réelle |
| Clients Enterprise (An 1) | 2 | 0–1 | Cycle de vente Enterprise long |
| GitHub Stars (An 2) | 10 000 | 3 000–5 000 | Croissance organique réelle |

---

## Conclusion

UMC est un projet avec une **vision juste**, une **architecture correcte dans les grandes lignes**, et une **documentation remarquablement complète**. Les problèmes identifiés sont réels mais **tous corrigeables**.

La priorité absolue est de :

1. **Corriger les failles de sécurité** (fuzzing des parsers) — avant tout lancement public
2. **Honnêter la promesse round-trip** — elle est fausse telle qu'énoncée
3. **Réduire le scope du MVP** — 5 formats excellents > 31 formats médiocres
4. **Simplifier l'infrastructure** — PostgreSQL > Kafka pour les 18 premiers mois
5. **Rendre les benchmarks reproductibles** — sans ça, la crédibilité technique s'effondre

Avec ces corrections, UMC peut réellement devenir l'outil de référence qu'il ambitionne d'être.

---

*UMC Diagnostic Complet v1.0 — Document d'amélioration interne*  
*Priorités : P0 (critique) → P1 (important) → P2 (optimisation) → P3 (long terme)*