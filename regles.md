# UMC — Règles d'Excellence en Conversion
## Le Standard Absolu · Document de Référence Définitif v1.0

> **Statut :** Document normatif — obligatoire pour tout développement d'UMC  
> **Philosophie :** Natif ou Rien · Honnête ou Rien · Parfait ou Documenté  
> **Principe directeur :** UMC ne ment jamais. Chaque perte est mesurée, bornée, documentée, certifiée.  
> **Ambition :** Être à l'IA ce que ffmpeg est à la vidéo — invisible, universel, indispensable.

---

# TABLE DES MATIÈRES

**PARTIE I — FONDATIONS**
1. [Les Cinq Vérités Fondamentales](#1-les-cinq-vérités-fondamentales)
2. [Classification des Formats par Convertibilité](#2-classification-des-formats-par-convertibilité)
3. [Les Trois Niveaux de Round-Trip](#3-les-trois-niveaux-de-round-trip)
4. [L'Architecture IR et l'ExtensionStore](#4-larchitecture-ir-et-lextensionstore)

**PARTIE II — CATALOGUE DES DIFFICULTÉS ET SOLUTIONS**
5. [Niveaux de Difficulté de Conversion](#5-niveaux-de-difficulté-de-conversion)
6. [Catalogue Exhaustif des Cas Difficiles et Solutions](#6-catalogue-exhaustif-des-cas-difficiles-et-solutions)
7. [Gestion des Formats Compilés](#7-gestion-des-formats-compilés)
8. [Gestion de la Quantification Croisée](#8-gestion-de-la-quantification-croisée)
9. [Gestion des Opérateurs Incompatibles](#9-gestion-des-opérateurs-incompatibles)
10. [Gestion des Structures Radicalement Différentes](#10-gestion-des-structures-radicalement-différentes)

**PARTIE III — LES 10 MÉCANISMES DE VÉRIFICATION**
11. [Mécanisme 1 — Dry-Run Pré-Conversion](#11-mécanisme-1--dry-run-pré-conversion)
12. [Mécanisme 2 — Checksums Hiérarchiques](#12-mécanisme-2--checksums-hiérarchiques)
13. [Mécanisme 3 — ExtensionStore Witness](#13-mécanisme-3--extensionstore-witness)
14. [Mécanisme 4 — Validation Numérique Exhaustive](#14-mécanisme-4--validation-numérique-exhaustive)
15. [Mécanisme 5 — Round-Trip Automatique](#15-mécanisme-5--round-trip-automatique)
16. [Mécanisme 6 — Conformity Check](#16-mécanisme-6--conformity-check)
17. [Mécanisme 7 — Pipeline Watchdog](#17-mécanisme-7--pipeline-watchdog)
18. [Mécanisme 8 — Checkpointing et Reprise](#18-mécanisme-8--checkpointing-et-reprise)
19. [Mécanisme 9 — Certificat de Conversion](#19-mécanisme-9--certificat-de-conversion)
20. [Mécanisme 10 — Audit Trail Immutable](#20-mécanisme-10--audit-trail-immutable)

**PARTIE IV — STANDARDS PAR FORMAT**
21. [Standard de Complétude par Format](#21-standard-de-complétude-par-format)
22. [Matrice de Couverture des Conversions](#22-matrice-de-couverture-des-conversions)
23. [Bornes de Divergence Officielles](#23-bornes-de-divergence-officielles)

**PARTIE V — RÈGLES OPÉRATIONNELLES**
24. [Règles du Pipeline de Conversion](#24-règles-du-pipeline-de-conversion)
25. [Règles de Sécurité du Parsing](#25-règles-de-sécurité-du-parsing)
26. [Règles de Performance](#26-règles-de-performance)
27. [Règles de Gestion des Erreurs](#27-règles-de-gestion-des-erreurs)
28. [Règles du Certificat et de la Certification](#28-règles-du-certificat-et-de-la-certification)

**PARTIE VI — DIMENSIONS DE COMPLÉTUDE**
29. [Les 12 Dimensions de Complétude UMC](#29-les-12-dimensions-de-complétude-umc)
30. [Fonctionnalités Avancées Obligatoires](#30-fonctionnalités-avancées-obligatoires)
31. [Check-list Avant Toute Décision de Conversion](#31-check-list-avant-toute-décision-de-conversion)

---

# PARTIE I — FONDATIONS

---

## 1. LES CINQ VÉRITÉS FONDAMENTALES

Ces vérités priment sur tout le reste. En cas de doute, revenez à elles.

### Vérité I — UMC ne ment jamais

> **Si une conversion est parfaite, UMC le certifie. Si elle est approximative, UMC documente chaque approximation. Si elle est impossible, UMC l'explique clairement. UMC ne fait jamais semblant.**

Ce que cela signifie en pratique :
- Jamais de "conversion réussie" sans validation effective.
- Jamais de chiffre de performance sans méthodologie publique reproductible.
- Jamais d'"information préservée" sans ExtensionStore Witness qui le vérifie.
- Jamais de certificat `full` si la moindre divergence non documentée existe.
- Jamais de "format supporté" si un seul cas de la spécification n'est pas implémenté.

### Vérité II — La perte zéro est le standard, la perte documentée est acceptable, la perte silencieuse est un bug critique

```
HIÉRARCHIE DE LA QUALITÉ DE CONVERSION :

1. Bit-identical (SHA256 identique)           → IDÉAL — viser pour A→A
2. Sémantiquement identique (δ < seuil)       → ACCEPTABLE — documenter δ
3. Structurellement identique (même graphe)   → ACCEPTABLE MINIMUM
4. Perte documentée explicitement             → TOLÉRÉ avec avertissement
5. Perte silencieuse                          → BUG CRITIQUE — inacceptable
6. Corruption de données                      → BUG FATAL — blocage immédiat
```

### Vérité III — L'ExtensionStore est le gardien absolu de l'information

> **Tout champ présent dans un fichier source DOIT survivre à une conversion, soit en étant mappé dans l'IR, soit en étant stocké dans l'ExtensionStore. Aucune information ne disparaît en silence.**

Corollaire : si une information ne peut pas être préservée (format compilé détruisant les poids), ce fait est documenté explicitement avant la conversion, pas découvert après.

### Vérité IV — La spécification officielle est loi

> **Avant d'écrire un seul loader ou saver, lire la spécification officielle du format en entier. Un fichier "qui s'ouvre dans l'outil officiel" n'est pas la même chose qu'un fichier "conforme à la spécification".**

Corollaire : en cas de contradiction entre la spécification et le comportement observé dans la nature, les deux sont documentés. La spécification est implémentée. La variante observée est stockée dans ExtensionStore avec une note explicative.

### Vérité V — L'utilisateur ne devrait jamais avoir à réfléchir

> **UMC fait le bon choix automatiquement. L'utilisateur indique source et cible, c'est tout. Le format est détecté, le chemin est calculé, les options sont optimisées, les problèmes sont anticipés.**

Corollaire : les messages d'erreur d'UMC ne disent jamais "quelque chose s'est mal passé". Ils disent exactement ce qui s'est mal passé, quel fichier, quel tenseur, et comment corriger le problème.

---

## 2. CLASSIFICATION DES FORMATS PAR CONVERTIBILITÉ

### Les Quatre Catégories de Formats

Chaque format est classé dans une catégorie qui détermine son comportement dans le graphe Dijkstra. Cette classification est déterminée par les propriétés du format, pas par des choix arbitraires.

| Catégorie | Symbole | Arêtes dans le graphe | Signification |
|-----------|---------|----------------------|---------------|
| **Bidirectionnel** | ↔ | Deux arêtes (A→B et B→A) | Peut être source ET cible. Round-trip garanti. |
| **Source uniquement** | → | Une arête sortante uniquement | Format legacy ou format dont l'écriture n'est pas utile. Lecture seule. |
| **Cible uniquement** | ← | Une arête entrante uniquement | Format compilé propriétaire. Écriture possible, relecture fidèle impossible. |
| **Best-effort** | ⇢ | Deux arêtes avec poids majoré + flag ⚠️ | Conversion possible avec perte documentée. Round-trip non garanti en SHA256. |

### Critères de Classification

Un format est **Cible uniquement** si au moins une de ces conditions est vraie :

**Condition 1 — Format compilé propriétaire**
Le fichier produit est un binaire optimisé pour un matériel spécifique. Extraire les poids d'un tel fichier est structurellement impossible ou produit des données incomplètes.
- TensorRT : kernels CUDA compilés, poids fusionnés avec les opérations, constantes pliées.
- CoreML : bundle compilé pour le Neural Engine Apple, poids réorganisés de manière destructive.
- Qualcomm QNN : binaire optimisé pour le DSP Hexagon dans un format propriétaire non documenté.
- ExecuTorch : binaire Meta compilé pour appareils mobiles.
- OpenVINO (partiellement) : certains opérateurs sont fusionnés de manière irréversible.

**Condition 2 — Perte d'information structurelle massive lors de la reconversion**
La conversion vers ce format détruit des informations essentielles non reconstituables.
- PyTorch dynamique → TFLite statique : la logique conditionnelle doit être "dépliée" en graphe statique. Le modèle résultant n'est plus structurellement équivalent.
- GGUF → CoreML (avec RoPE YaRN) : si le modèle utilise des extensions spécifiques que CoreML décompose en primitives, le round-trip ne retrouve pas la structure originale.

**Condition 3 — Format legacy non maintenu**
Le format est obsolète, sa spécification est incomplète, et l'écriture serait une maintenance ingérable.
- GGML : prédécesseur de GGUF, format non extensible, spécification informelle.
- Keras H5 : remplacé par le format `.keras` et TensorFlow SavedModel.

### Table de Classification des 32 Formats UMC

| Format | Catégorie | Round-trip | Note |
|--------|-----------|------------|------|
| GGUF | ↔ Bidirectionnel | ✅ SHA256 identique | Weights-only, graphe reconstruit via GraphTemplate |
| SafeTensors | ↔ Bidirectionnel | ✅ SHA256 identique | Format pivot, le plus simple |
| ONNX | ↔ Bidirectionnel | ✅ SHA256 identique | Pivot central du graphe, supporte les sous-graphes |
| PyTorch | ↔ Bidirectionnel | ✅ SHA256 identique | State dict pickle, ops dynamiques détectés |
| TFSavedModel | ↔ Bidirectionnel | ✅ SHA256 identique | Protobuf complet |
| TFLite | ↔ Bidirectionnel | ✅ SHA256 identique | FlatBuffer, ensemble d'ops restreint |
| AWQ | ↔ Bidirectionnel | ✅ Via ExtensionStore | Quantification par canal |
| GPTQ | ↔ Bidirectionnel | ✅ Via ExtensionStore | Quantification avec ordering |
| LoRA / QLoRA / PEFT | ↔ Bidirectionnel | ✅ SHA256 identique | Adaptateurs, pas un format complet |
| SentencePiece | ↔ Bidirectionnel | ✅ SHA256 identique | Tokenizer |
| TikToken | ↔ Bidirectionnel | ✅ SHA256 identique | Tokenizer |
| PaddlePaddle | ↔ Bidirectionnel | ✅ SHA256 identique | Protobuf + pdparams |
| TorchScript | ↔ Bidirectionnel | ✅ SHA256 identique | ZIP + JIT serialization |
| ONNXRuntime | ↔ Bidirectionnel | ✅ Sémantique | ONNX avec extensions ORT |
| OpenVINO | ↔ Bidirectionnel | ✅ Sémantique | XML + bin natif |
| Diffusers | ↔ Bidirectionnel | ✅ Sémantique | Format composite multi-sous-modèles |
| MediaPipe | ↔ Bidirectionnel | ✅ Sémantique | TFLite + metadata JSON |
| bitsandbytes | → Source uniquement | ❌ — | NF4/FP4, lecture seule |
| GGML | → Source uniquement | ❌ — | Legacy, migration → GGUF |
| Keras H5 | → Source uniquement | ❌ — | Legacy, migration → TFSavedModel |
| JAX/Flax | → Source uniquement | ❌ — | Lecture seule, → SafeTensors |
| CoreML | ← Cible uniquement | ❌ Compilé | .mlpackage non compilé = natif ; compilé = irréversible |
| TensorRT | ← Cible uniquement | ❌ Compilé | Recipe Saver uniquement |
| QualcommQNN | ← Cible uniquement | ❌ Compilé | Recipe Saver uniquement |
| TensorRTLLM | ← Cible uniquement | ❌ Compilé | Recipe Saver uniquement |
| ApacheTVM | ← Cible uniquement | ❌ Compilé | Recipe Saver uniquement |
| NVIDIATriton | ← Cible uniquement | ❌ Config | Config génération uniquement |
| ExecuTorch | ← Cible uniquement | ❌ Compilé | FlatBuffers compilé mobile |
| ONNXWeb | ← Cible uniquement | ❌ Bundle | Bundle web, pas de relecture |

> **Règle absolue** : Le graphe Dijkstra n'a AUCUNE arête sortante depuis un format Cible uniquement. Si l'utilisateur demande `TensorRT → ONNX`, UMC répond avec un message clair expliquant l'impossibilité structurelle, pas une erreur générique.

---

## 3. LES TROIS NIVEAUX DE ROUND-TRIP

Ces niveaux ne sont pas arbitraires. Ils correspondent à des garanties mathématiques vérifiables automatiquement.

### Niveau 1 — Bit-Identical

**Définition** : SHA256(fichier_source) == SHA256(fichier_reconstruit)  
**Applicable** : Uniquement pour les conversions A → A (même format)  
**Vérification** : Automatique, instantanée

```
GGUF → GGUF                   : SHA256 identique ✅
SafeTensors → SafeTensors     : SHA256 identique ✅
ONNX → ONNX                   : SHA256 identique ✅
PyTorch → PyTorch             : SHA256 identique ✅
```

**Exception documentée** : Si le format source utilise un alignement ou un padding dynamique (ex : GGUF aligne sur 32 octets, SafeTensors sur 256 octets), la conversion A → B → A peut produire un fichier fonctionnellement identique mais avec un SHA256 différent en raison du padding. Ce cas est documenté explicitement par paire de formats.

### Niveau 2 — Sémantique

**Définition** : Les sorties d'inférence du modèle source et du modèle converti sont identiques dans la tolérance documentée. `|output_source - output_converted| ≤ δ` pour tout input.  
**Applicable** : Conversions cross-format entre formats bidirectionnels  
**Vérification** : Comparaison tenseur par tenseur après déquantification

```
GGUF → SafeTensors         : δ < 1e-7 (F32→F32 direct)
GGUF Q4_K_M → SafeTensors F16 : δ < 9.2e-3 (déquant + F16)
AWQ 4-bit → ONNX F16       : δ < 1e-2
GPTQ 4-bit → SafeTensors F16 : δ < 1e-2
```

**Mécanisme de préservation du round-trip** : les paramètres de quantification originaux (scales, zero-points, block_size, etc.) sont TOUJOURS stockés dans ExtensionStore, même après déquantification. Ainsi, au round-trip SafeTensors → GGUF, les paramètres Q4_K_M sont restaurés depuis ExtensionStore — pas de requantification, SHA256 identique au GGUF original.

### Niveau 3 — Structurel

**Définition** : La structure du modèle est préservée — même architecture, même nombre de couches, même topologie de graphe.  
**Applicable** : Conversions vers des formats compilés, ou conversions avec perte de graphe documentée  
**Vérification** : Hash topologique du graphe (avant et après conversion)

```
ONNX → TensorRT    : Structurel (engine compilé, poids non extractibles)
ONNX → CoreML      : Structurel (certains ops décomposés)
PyTorch → ExecuTorch : Structurel (optimisations mobiles)
```

### Règle d'Application des Niveaux

UMC détermine automatiquement le niveau approprié pour chaque paire de formats et le documente dans le certificat. Cette détermination est :
- Calculée au moment du dry-run
- Affichée à l'utilisateur avant conversion
- Inscrite dans le certificat de conversion
- Jamais rehaussée (on ne prétend pas Bit-Identical si c'est Sémantique)

---

## 4. L'ARCHITECTURE IR ET L'EXTENSIONSTORE

### L'IR comme Pivot Universel

L'IR (Intermediate Representation) d'UMC est le pivot central de toutes les conversions. Elle est :
- Un **sur-ensemble mathématique** de tous les formats supportés
- **Évolutive** : enrichie à chaque nouveau format ajouté
- **Jamais parfaite d'emblée** : certains cas edge nécessitent une logique de paire source→cible

```
Format A → [Loader A] → IR_UMC → [Saver B] → Format B

80% des conversions : N + M composants (IR suffit)
20% des conversions : logique spécifique à la paire source→cible
                       (BatchNorm PyTorch ≠ BatchNorm ONNX,
                        NCHW ↔ NHWC, endianness, alignement)
```

Ces 20% sont gérés via les **ConversionHints** : métadonnées supplémentaires transmises avec l'IR pour guider le saver cible. Ils sont documentés dans la ConversionHintsMap et ne constituent pas une exception silencieuse.

### L'ExtensionStore : Garantie Absolue de Zéro Perte

L'ExtensionStore est le mécanisme qui rend la garantie "zéro perte d'information" possible, même entre des formats incompatibles.

**Règle absolue** : Tout champ présent dans un fichier source qui ne peut pas être mappé nativement dans l'IR DOIT être placé dans ExtensionStore avec une clé namespaced.

**Format des clés** : `"FORMAT@VERSION/chemin/vers/champ"`
```
"GGUF@v3/tokenizer.chat_template"
"GGUF@v3/rope_scaling.type"
"GGUF@v3/tokenizer.ggml.tokens"
"ONNX@opset21/custom_metadata/producer_name"
"PyTorch@1.x/metadata/_metadata"
```

**Limites obligatoires** :
- Taille maximale : 100 Mo par défaut (configurable)
- Clés : alphanumériques + `@/._-`, longueur ≤ 512 caractères
- Validation obligatoire à l'insertion (clé malformée = erreur explicite, pas silencieuse)

**Cycle de vie obligatoire** :
1. **Au chargement** : tous les champs non-mappables → ExtensionStore (avec log)
2. **Pendant la conversion** : ExtensionStore traverse l'IR intact
3. **À l'écriture** : le saver consulte ExtensionStore pour les champs du format cible
4. **Vérification post-écriture** : ExtensionStore Witness vérifie qu'aucun champ n'a été perdu
5. **Au round-trip** : les champs originaux sont restaurés depuis ExtensionStore

**Ce qui n'est PAS préservable** (documenté honnêtement) :
| Information | Pourquoi impossible |
|------------|---------------------|
| Poids après compilation TensorRT | Engine binaire opaque, poids fusionnés avec les opérations |
| Graphe dynamique PyTorch → TFLite statique | Boucles conditionnelles structurellement absentes de TFLite |
| Distribution NF4 → INT8 | NF4 est non-linéaire, INT8 est linéaire. Approximation inévitable. |
| Kernels CUDA custom | Binaires compilés non portables |

---

# PARTIE II — CATALOGUE DES DIFFICULTÉS ET SOLUTIONS

---

## 5. NIVEAUX DE DIFFICULTÉ DE CONVERSION

Comprendre la difficulté d'une conversion permet d'anticiper les problèmes, d'allouer les ressources correctement, et de définir les attentes utilisateur.

### Niveau 1 🟢 — Direct (Faible difficulté)

**Définition** : Copie de données avec changement d'enveloppe. Les données restent identiques, seul le conteneur change.

**Caractéristiques** :
- Même dtype source et cible
- Pas de transformation des données
- Mappage direct des métadonnées
- Round-trip bit-identical garanti

**Exemples** :
- GGUF F32 ↔ SafeTensors F32 : copie pure, changement de header
- SafeTensors F16 ↔ ONNX F16 : les deux supportent F16 nativement
- PyTorch state_dict F32 ↔ SafeTensors F32 : changement de sérialisation (pickle → JSON + flat buffer)

### Niveau 2 🟡 — Changement de dtype (Difficulté moyenne)

**Définition** : Les données doivent être transformées, mais de manière déterministe et avec une perte bornée.

**Caractéristiques** :
- Conversion IEEE 754 standardisée
- Bornes d'erreur connues et documentées
- Round-trip sémantique garanti (via ExtensionStore)

**Exemples et bornes** :
- F32 → F16 : perte de mantisse (23 bits → 10 bits), δ < 4.88e-4
- F16 → F32 : lossless (élargissant), δ = 0
- F32 → BF16 : perte plus importante (23 bits → 7 bits), δ < 7.8e-3
- BF16 → F32 : lossless (élargissant), δ = 0

### Niveau 3 🟠 — Quantification (Difficulté élevée)

**Définition** : Chaque schéma de quantification a sa propre logique de regroupement, ses métadonnées propres, et ses hypothèses sur les données.

**Caractéristiques** :
- Déquantification requise pour la plupart des conversions cross-scheme
- Paramètres de quantification (scales, zero-points, block_size) doivent être préservés
- Round-trip via ExtensionStore si même schéma source/cible
- Double perte si changement de schéma

**Exemples et bornes** :
- GGUF Q4_K_M → SafeTensors F32 : déquantification Q4_K_M → F32, δ < 8.7e-3
- GGUF Q4_K_M → AWQ 4-bit : déquantification + requantification, δ < 1.9e-2
- GPTQ → GGUF Q4_K_M : désordering + déquantification + requantification, δ < 1.9e-2
- NF4 (bitsandbytes) → INT8 : table de correspondance NF4→F32, puis INT8, δ variable

### Niveau 4 🔴 — Opérateurs incompatibles (Très difficile)

**Définition** : Deux formats n'ont pas les mêmes opérateurs, ou ont des opérateurs similaires avec des sémantiques différentes.

**Caractéristiques** :
- Décomposition mathématique requise
- Risque de micro-divergence due à l'arithmétique flottante non-associative
- Certains opérateurs peuvent être impossibles à décomposer fidèlement
- Tests de précision obligatoires

**Exemples** :
- RmsNorm → ONNX (décomposition en 7 ops)
- RoPE avec scaling YaRN → ONNX
- GroupedQueryAttention (GQA) → formats ne le supportant pas
- FlashAttention → ONNX standard (décomposition en SDPA)
- BatchNorm PyTorch ≠ BatchNorm ONNX (sémantiques différentes)

### Niveau 5 🔴🔴 — Structures radicalement différentes (Extrême)

**Définition** : Certains formats n'ont pas la même philosophie. La conversion est une traduction entre deux paradigmes.

**Caractéristiques** :
- Reconstruction ou transformation massive de la structure
- Requiert une connaissance externe au fichier (architecture du modèle)
- Certaines conversions sont structurellement impossibles en sens inverse

**Exemples** :
- GGUF (weights-only) → ONNX (graph-full) : nécessite GraphTemplate + catalogue d'architectures
- Diffusers (dossier multi-modèle) → format unique : fusion ou multi-fichiers
- TensorRT → autre format : structurellement impossible (engine compilé)
- PyTorch avec ops dynamiques → formats statiques : dépliage de graphe

---

## 6. CATALOGUE EXHAUSTIF DES CAS DIFFICILES ET SOLUTIONS

### 6.1 GGUF → ONNX (weights-only → graph complet)

**Problème** : GGUF ne contient que des poids. ONNX exige un graphe de calcul complet. Sans connaissance de l'architecture, impossible de reconstruire le graphe.

**Solution UMC** :
1. Détecter l'architecture via `general.architecture` dans les métadonnées GGUF.
2. Chercher le template correspondant dans le GraphTemplateRegistry.
3. Si template trouvé : instancier le graphe en injectant les hyperparamètres (hidden_size, num_layers, num_heads, num_kv_heads, rope_theta, etc.).
4. Si template non trouvé : produire un ONNX weights-only avec avertissement clair. Ne pas produire un graphe approximatif.
5. Pour les architectures inconnues : mode dégradé avec message explicite et demande de contribution à la communauté.

**Templates obligatoires à maintenir** :
- LlamaTemplate : Llama 1/2/3/3.1/3.2, Mistral, Mixtral, Vicuna, Alpaca, Hermes, Zephyr
- PhiTemplate : Phi-1/2/3/3.5
- GemmaTemplate : Gemma 1/2
- QwenTemplate : Qwen 1/1.5/2/2.5
- FalconTemplate : Falcon
- GPTNeoXTemplate : GPT-NeoX, Pythia, Dolly
- OPTTemplate : OPT
- BloomTemplate : BLOOM, BLOOMZ
- GPT2Template : GPT-2 et variantes

**Vérification obligatoire après reconstruction** :
- Tous les tenseurs référencés dans le graphe existent dans TensorStore.
- Les shapes des tenseurs sont compatibles avec les opérateurs du graphe.
- Le graphe est un DAG valide (pas de cycles).

### 6.2 ONNX → GGUF (graph complet → weights-only)

**Problème** : ONNX contient un graphe complet. GGUF n'a pas de graphe. "Oublier" le graphe sans perdre d'information.

**Solution UMC** :
1. Extraire les poids du graphe ONNX et les écrire dans GGUF.
2. Stocker le graphe ONNX INTÉGRALEMENT dans ExtensionStore avec la clé `"ONNX@opset{N}/compute_graph"`.
3. Stocker les métadonnées ONNX exclusives (producer_name, model_version, doc_string) dans ExtensionStore.
4. Au round-trip GGUF → ONNX : restaurer le graphe depuis ExtensionStore.
5. Résultat : SHA256(ONNX_original) == SHA256(ONNX_reconstruit).

**Règle** : le graphe ONNX complet peut représenter plusieurs Mo. L'ExtensionStore peut gérer cela jusqu'à sa limite de 100 Mo. Si le graphe dépasse cette limite, avertir l'utilisateur et documenter la perte partielle.

### 6.3 Diffusers (dossier) → format unique

**Problème** : Un pipeline Diffusers contient UNet, VAE, Text Encoder, Tokenizer, Scheduler dans plusieurs sous-répertoires.

**Solution UMC — Trois options présentées à l'utilisateur** :

**Option 1 (Défaut) — Conversion multi-fichiers** :
- Chaque sous-modèle est converti séparément.
- Sortie : dossier `output/` contenant `unet.onnx`, `vae.onnx`, `text_encoder.onnx`.
- Round-trip : chaque sous-modèle a son propre round-trip.

**Option 2 — Conversion sélective** :
- L'utilisateur choisit quel sous-modèle convertir.
- Exemple : `umc convert ./sd-model/ unet.onnx --component unet`

**Option 3 (Expérimental) — Fusion en un seul fichier** :
- UMC tente de fusionner les sous-modèles en un seul graphe ONNX avec entrées/sorties multiples.
- Marqué `partial` car la fusion peut introduire des incompatibilités avec certains runtimes.
- Avertissement obligatoire affiché.

**Détection automatique obligatoire** :
- SD 1.x vs SD 2.x vs SDXL vs SD 3.x vs Flux vs Wan
- Chaque version a une structure de dossier différente
- UMC lit `model_index.json` pour détecter la version et les composants

### 6.4 PyTorch avec ops dynamiques → formats statiques

**Problème** : PyTorch supporte les branchements conditionnels et les boucles dynamiques. TFLite, CoreML, ONNX (partiellement) exigent des graphes statiques.

**Solution UMC** :
1. Le dry-run tente de tracer le modèle avec `torch.jit.trace`.
2. Si le traçage réussit : le graphe statique est produit, la conversion continue.
3. Si le traçage échoue (ops dynamiques non traçables) : la conversion vers le format statique est **bloquée**.
4. Message précis affiché : "Le modèle contient des opérations dynamiques ({liste des ops}) non traçables. Conversion vers {format} impossible. Alternatives : PyTorch → SafeTensors (weights-only), ou fournissez un exemple d'input pour le traçage statique."
5. Si l'utilisateur force : les ops dynamiques sont stockées en `Custom`. Certificat `partial`.

---

## 7. GESTION DES FORMATS COMPILÉS

### Règle Fondamentale

> **Un format compilé est une destination finale, jamais un intermédiaire. UMC le génère mais ne le lit pas pour reconvertir.**

### Comportement Obligatoire pour les Formats Compilés

**Avant la conversion** :
- Le dry-run affiche obligatoirement : "⚠️ {format} est un format compilé. Il ne peut pas être reconverti. Conservez toujours le fichier source ({source_format})."
- Le dry-run suggère : "Recommandation : stockez {source_file} (SHA256: {hash}) dans un emplacement sûr avant de procéder."

**Lors de la conversion** :
- UMC génère le format compilé via les outils appropriés (Recipe Saver ou outil natif).
- Le fichier source (ONNX ou SafeTensors) est enregistré dans l'Audit Trail avec son SHA256.

**Dans le certificat** :
- Le certificat est obligatoirement `partial` pour tout format compilé.
- La mention suivante est obligatoire : "Ce fichier est un format compilé cible unique. La conversion inverse est structurellement impossible. Le fichier source {nom} (SHA256: {hash}) doit être conservé pour toute conversion future."

**Si l'utilisateur demande format_compilé → autre_format** :
- Dijkstra ne trouve aucun chemin (pas d'arête sortante).
- Message précis : "TensorRT (.engine) est un format compilé propriétaire. Les poids sont fusionnés avec les kernels CUDA de manière irréversible, même par NVIDIA. Pour convertir ce modèle, vous avez besoin du fichier source ONNX ou PyTorch qui a servi à générer l'engine."
- UMC propose : "Si vous avez le fichier source, UMC peut le convertir vers n'importe quel format. Commande : `umc convert model.onnx model_new.safetensors`"

### Recipe Savers — Standard Obligatoire

Pour TensorRT, QNN, TVM, Triton, TensorRT-LLM : UMC ne génère pas le format directement. Il génère une **recette de build reproductible** que l'utilisateur exécute.

**Contenu obligatoire d'une Recipe** :
- Le fichier ONNX optimisé (préparé par UMC, optimisé pour le format cible)
- La commande exacte à exécuter (avec toutes les options)
- La version de l'outil requise
- L'URL d'installation de l'outil
- Les paramètres recommandés pour ce modèle spécifique
- Un avertissement sur l'irréversibilité

**Exemple pour TensorRT** :
```bash
# Recette générée par UMC v3.0.0
# Fichier source : model.gguf
# Format cible : TensorRT FP16
# ⚠️ Le fichier .engine produit ne peut pas être reconverti

# Prérequis : TensorRT 10.x (https://developer.nvidia.com/tensorrt)
# Vérifiez la version : trtexec --version

trtexec \
  --onnx=model_optimized.onnx \
  --saveEngine=model.engine \
  --fp16 \
  --workspace=4096 \
  --minShapes=input_ids:1x1 \
  --optShapes=input_ids:1x512 \
  --maxShapes=input_ids:8x2048

# SHA256 du fichier ONNX source à conserver : {hash}
# Ce fichier ONNX source est nécessaire pour toute conversion future
```

---

## 8. GESTION DE LA QUANTIFICATION CROISÉE

### Principe Général

La quantification est le domaine le plus complexe de la conversion. Deux schémas 4-bit ne sont pas compatibles. La conversion cross-scheme est toujours une approximation.

### Représentation Canonique de Quantification

Tout schéma de quantification peut être converti vers la **représentation canonique** (CanonicalQuantization) qui sert de pivot universel :

```
CanonicalQuantization {
    bit_width: u8,                    // 4, 8, etc.
    block_size: usize,                // OBLIGATOIRE
    superblock_size: Option<usize>,   // Pour GGUF K-quants : 256
    scales: Vec<f32>,
    zero_points: Vec<f32>,
    scales_dtype: DType,              // F16, F32, Q8_0...
    quantized_data: Vec<u8>,
    storage_order: StorageOrder,      // Sequential, Interleaved, BlockPacked
}
```

**Règle** : La représentation canonique est le seul moyen de convertir entre deux schémas de quantification différents. Aucune conversion directe schéma → schéma sans passer par la représentation canonique.

### Tableau des Conversions de Quantification

| Source | Cible | Méthode | Perte max | Avertissement |
|--------|-------|---------|-----------|---------------|
| GGUF Q4_K_M | SafeTensors F32 | Déquantification | δ < 8.7e-3 | Orange : paramètres préservés ExtensionStore |
| GGUF Q4_K_M | SafeTensors F16 | Déquant + F16 | δ < 9.2e-3 | Orange : paramètres préservés ExtensionStore |
| GGUF Q4_K_M | AWQ 4-bit | Déquant + Requant | δ < 1.9e-2 | Rouge : double quantification, recommander F16 |
| GGUF Q4_K_M | GPTQ 4-bit | Déquant + Requant | δ < 1.9e-2 | Rouge : double quantification |
| AWQ 4-bit | SafeTensors F16 | Déquant | δ < 1e-2 | Orange : paramètres AWQ préservés |
| AWQ 4-bit | GGUF Q4_K_M | Déquant + Requant | δ < 1.9e-2 | Rouge : double quantification |
| GPTQ 4-bit | SafeTensors F16 | Désordering + Déquant | δ < 1e-2 | Orange |
| NF4 (bnb) | F32 | Table NF4→F32 | δ dépend des valeurs | Orange : distribution non-linéaire |
| NF4 (bnb) | INT8 | Table NF4→F32 + Requant | δ élevée | Rouge : recommander F16 |
| INT8 symétrique | F32 | Déquant (lossless) | δ = 0 | Vert |
| F32 | GGUF Q4_K_M | Requant | δ < 8.7e-3 | Orange |
| F16 | GGUF Q4_K_M | F16→F32 + Requant | δ < 8.8e-3 | Orange |

### Règles Obligatoires pour la Quantification Croisée

**Règle 1** : Toute requantification AWQ ou GPTQ est signalée avec un avertissement rouge car ces schémas utilisent une optimisation de second ordre (Hessian pour GPTQ, calibration dataset pour AWQ) qui ne peut pas être reproduite sans les données d'entraînement.

**Règle 2** : La conversion NF4 → tout schéma linéaire est traitée avec la table de correspondance officielle bitsandbytes. Jamais d'interpolation linéaire.

**Règle 3** : Pour GPTQ, l'algorithme de désordering GPTQ doit être appliqué avant déquantification. L'ordering est dans les métadonnées du modèle GPTQ.

**Règle 4** : Les paramètres de quantification originaux (scales, zero-points, block_size, superblock_size, calibration_method, calibration_dataset) sont TOUJOURS préservés dans ExtensionStore, même si le modèle est déquantifié vers F32 ou F16.

**Règle 5** : Si la perte cumulée dépasse 2e-2, UMC propose activement une alternative : "Pour une fidélité maximale, convertissez d'abord vers F16 : `umc convert model.gguf model.safetensors --dtype fp16`, puis vers votre format cible."

### Déquantification GGUF K-quants — Algorithme Exact

GGUF utilise des super-blocs de 256 éléments contenant 8 blocs de 32 éléments. L'algorithme exact doit être respecté :

```
Pour Q4_K_M (exemple) :
1. Lire le super-bloc (256 éléments = 128 octets de données 4-bit)
2. Lire les 2 scales du super-bloc (F16, 4 octets total)
3. Pour chaque bloc de 32 éléments :
   a. Lire les 16 octets de données 4-bit
   b. Appliquer le scale du bloc (dérivé des 2 scales du super-bloc)
   c. Dépackager les nibbles : lo = byte & 0x0F, hi = byte >> 4
   d. Déquantifier : float = scale * (nibble - zero_point)
4. Résultat : 256 valeurs F32 par super-bloc
```

Toute implémentation qui dévie de cet algorithme exact produit des résultats incorrects. Les tests round-trip sur des modèles réels (Llama, Mistral, Phi) valident l'implémentation.

---

## 9. GESTION DES OPÉRATEURS INCOMPATIBLES

### Principe Général

Un opérateur non supporté par le format cible n'est jamais une erreur fatale. Il est :
1. Décomposé en primitives si possible (décomposition mathématiquement exacte)
2. Stocké en `Custom` dans le graphe si non décomposable
3. Accompagné d'un avertissement explicite

### Catalogue de Décompositions Obligatoires

Ces décompositions sont mathématiquement prouvées. Elles doivent produire exactement le même résultat que l'opérateur original sur tout input.

**RmsNorm → ONNX opset < 20** :
```
RmsNorm(x, weight, eps) :
= Pow(x, 2.0)                              [op 1]
  ReduceMean(..., axes=[-1], keepdims=true) [op 2]
  Add(..., eps_constant)                    [op 3]
  Sqrt(...)                                 [op 4]
  Reciprocal(...)                           [op 5]
  Mul(x, ...)                               [op 6]
  Mul(..., weight)                          [op 7]

Source de micro-divergence : ordre des additions dans ReduceMean
Borne documentée : δ < 1e-7 en pratique
```

**HardSwish → décomposition universelle** :
```
HardSwish(x) = x * ReLU6(x + 3) / 6
= Add(x, 3.0_constant)   [op 1]
  Relu6(...)              [op 2]
  Mul(x, ...)             [op 3]
  Mul(..., 1/6_constant)  [op 4]
```

**SiLU (Swish) → décomposition universelle** :
```
SiLU(x) = x * Sigmoid(x)
= Sigmoid(x)  [op 1]
  Mul(x, ...) [op 2]
```

**GeluApprox → décomposition ONNX** :
```
GeluApprox(x) = 0.5 * x * (1 + Tanh(sqrt(2/π) * (x + 0.044715 * x³)))
= Pow(x, 3.0_constant)                   [op 1]
  Mul(..., 0.044715_constant)             [op 2]
  Add(x, ...)                             [op 3]
  Mul(..., sqrt(2/π)_constant)            [op 4]
  Tanh(...)                               [op 5]
  Add(..., 1.0_constant)                  [op 6]
  Mul(x, ...)                             [op 7]
  Mul(..., 0.5_constant)                  [op 8]
```

**RotaryPositionEmbedding (RoPE) → décomposition ONNX** :
```
RoPE(x, position_ids, theta) :
1. Calculer les fréquences : freq = theta^(-2i/d) pour i in 0..d/2
2. Calculer les angles : angles = position_ids * freq
3. cos_angles = Cos(angles)
4. sin_angles = Sin(angles)
5. Séparer x en paires : x_even = x[::2], x_odd = x[1::2]
6. Appliquer la rotation : x_rot = concat(
     x_even * cos - x_odd * sin,
     x_even * sin + x_odd * cos
   )
7. Réintégrer dans le tenseur original
```

### Comportement pour les Opérateurs Non Décomposables

Si un opérateur n'a pas de décomposition disponible ET ne peut pas être représenté dans le format cible :

1. **Log** : tracing warning avec le nom de l'opérateur, son domaine, ses attributs.
2. **Stockage** : l'opérateur est stocké en `Custom` dans le graphe, ses attributs dans ExtensionStore.
3. **Certificat** : `partial` avec la liste des opérateurs non supportés.
4. **Message utilisateur** : "L'opérateur '{domain}/{op_type}' n'a pas d'équivalent dans {format_cible}. Il est préservé comme blob opaque. Le modèle converti ne peut pas exécuter cet opérateur. Si c'est un opérateur d'entraînement (backward), cela n'affecte pas l'inférence."
5. **Suggestion** : UMC vérifie si c'est un opérateur d'entraînement et le mentionne dans le message.

### Différences Sémantiques d'Opérateurs Similaires

Certains opérateurs portent le même nom dans plusieurs frameworks mais ont des sémantiques différentes. Ces cas sont des sources de bugs silencieux que UMC doit détecter.

**BatchNorm PyTorch vs ONNX** :
- PyTorch en mode `training=True` : met à jour les running_mean/running_var
- PyTorch en mode `training=False` : utilise les running_mean/running_var figés
- ONNX BatchNorm : a un paramètre `training_mode` explicite
- **Solution UMC** : détecter le mode PyTorch et injecter le bon `training_mode` dans l'opérateur ONNX.

**Gather PyTorch vs ONNX** :
- PyTorch `torch.gather` : indices et source ont la même shape, résultat de même shape
- ONNX `Gather` : sémantique différente selon l'axe
- **Solution UMC** : mapper explicitement et tester avec des shapes typiques.

---

## 10. GESTION DES STRUCTURES RADICALEMENT DIFFÉRENTES

### Layout Mémoire (NCHW ↔ NHWC)

PyTorch utilise NCHW (batch, channels, height, width) par défaut.  
TFLite utilise NHWC (batch, height, width, channels) par défaut.

**Règle** : UMC détecte le layout automatiquement via `Tensor.layout` et applique la transposition nécessaire. La transposition est stockée dans ConversionHints pour être documentée dans le certificat.

**Méthode** :
```
NCHW → NHWC : Transpose(perm=[0, 2, 3, 1])
NHWC → NCHW : Transpose(perm=[0, 3, 1, 2])
```

La transposition est appliquée UNE SEULE FOIS par tenseur, pas à chaque étape de la chaîne de conversion.

### Tied Weights (Poids Liés)

Llama et d'autres architectures ont `embed_tokens.weight == lm_head.weight` (poids partagés). Certains formats stockent les deux tenseurs, d'autres une seule fois avec une référence.

**Politique UMC par défaut** :
- SafeTensors → GGUF : déduplique les tied weights (un seul tenseur stocké)
- GGUF → ONNX : selon le paramètre `tie_word_embeddings` dans l'architecture
- Pour les formats qui ne supportent pas les tied weights : dupliquer avec avertissement

**Stockage dans ConversionHints** :
```rust
TiedWeightsPolicy::PreserveShared   // Garder liés (économise mémoire)
TiedWeightsPolicy::Duplicate        // Dupliquer pour les formats sans sharing
TiedWeightsPolicy::Deduplicate      // Dédupliquer si le source avait des copies
```

### Sous-graphes ONNX (If, Loop, Scan)

ONNX supporte des sous-graphes pour les branchements et boucles. Ces sous-graphes doivent être traités récursivement.

**Règle** : La récursion est limitée à 32 niveaux (SecurityBounds.max_metadata_nesting). Au-delà, la conversion est bloquée avec un message précis.

**Traitement** : Chaque sous-graphe est traité comme un ComputeGraph indépendant, stocké dans `OpAttributes.graphs`.

---

# PARTIE III — LES 10 MÉCANISMES DE VÉRIFICATION

---

## 11. MÉCANISME 1 — DRY-RUN PRÉ-CONVERSION

**Statut** : Obligatoire. Automatique. Ne peut pas être désactivé.

**Déclenchement** : Avant CHAQUE conversion, sans exception.

### Ce que le Dry-Run Vérifie

**Vérification 1 — Compatibilité des formats**
- Le format source est-il lisible par UMC ?
- Le format cible est-il écrivable par UMC ?
- Existe-t-il un chemin de conversion (Dijkstra) ?
- Si non : message précis, pas d'erreur générique.

**Vérification 2 — Compatibilité des opérateurs**
- Pour chaque opérateur du graphe source : existe-t-il un équivalent dans la cible ?
- Si non : peut-il être décomposé ? → liste des décompositions applicables
- Si non décomposable : peut-il être stocké en `Custom` ? → avertissement
- Score de compatibilité : `N_supportés / N_total * 100`

**Vérification 3 — Compatibilité des dtypes**
- Pour chaque tenseur : le dtype est-il supporté par le format cible ?
- Si non : quelle conversion appliquer ? Quelle borne de perte ?
- Estimation de la divergence cumulée.

**Vérification 4 — Métadonnées et ExtensionStore**
- Combien de champs seront mappés dans l'IR ?
- Combien de champs seront dans ExtensionStore ?
- Y a-t-il des champs qui ne pourront pas être préservés ?

**Vérification 5 — Ressources**
- Espace disque estimé pour le fichier de sortie (formules par paire de formats)
- Espace disque disponible (vérification OS)
- RAM estimée nécessaire (fonction de la taille du modèle)
- Temps estimé (basé sur les benchmarks internes)
- Si ressources insuffisantes : blocage AVANT conversion avec message précis

**Vérification 6 — Niveau de round-trip**
- Quel niveau de round-trip (1/2/3) est garanti pour cette paire ?
- La divergence estimée est-elle dans les seuils acceptables ?

### Seuils de Blocage

| Score de compatibilité | Comportement | Message |
|------------------------|--------------|---------|
| 100% | ✅ Conversion autorisée | Aucun avertissement |
| 95-99% | ⚠️ Warning orange | "X opérateurs décomposés" |
| 90-94% | ⚠️ Warning rouge | "Confirmation requise" |
| < 90% | ❌ Blocage | "Conversion non recommandée. Force avec --force" |

### Format de Sortie du Dry-Run

```
╔══════════════════════════════════════════════════════════════╗
║         UMC DRY RUN — {source} → {target}                    ║
╠══════════════════════════════════════════════════════════════╣
║ Format détecté : {format} {version}                          ║
║ Architecture   : {architecture}                              ║
║ Paramètres     : {count}                                     ║
║                                                              ║
║ Compatibilité : {N}/{N_total} opérateurs ✅                   ║
║ Décompositions : {N_décomp} opérateurs (mathématiquement ex) ║
║ Ops non supportés : {N_custom} (stockés en Custom)           ║
║                                                              ║
║ Niveau round-trip : {NIVEAU} ({description})                 ║
║ Divergence estimée : < {delta_max}                           ║
║                                                              ║
║ Ressources :                                                 ║
║   Taille sortie estimée : {size}                             ║
║   RAM nécessaire : {ram}                                     ║
║   Temps estimé : {time}                                      ║
║                                                              ║
║ ⚠️ Avertissements : {warnings}                               ║
║                                                              ║
║ {VERDICT} — {message}                                        ║
╚══════════════════════════════════════════════════════════════╝
```

---

## 12. MÉCANISME 2 — CHECKSUMS HIÉRARCHIQUES

**Statut** : Obligatoire. Automatique. À chaque tenseur.

### Quatre Niveaux de Checksum

| Niveau | Quoi | Algorithme | Quand | Blocage si échec |
|--------|------|------------|-------|------------------|
| L1 | Chaque tenseur | xxHash64 | Au chargement + après conversion | Immédiat (tenseur identifié nommément) |
| L2 | Chaque shard (si shardé) | SHA256 | Après écriture du shard | 3 tentatives puis échec |
| L3 | Fichier complet | SHA256 | Après écriture finale | Suppression + notification |
| L4 | Topologie du graphe | Hash personnalisé | Avant et après conversion | Certificat `partial` |

### Règles d'Application

**Règle 1** : Le checksum xxHash64 est calculé une fois au chargement et stocké dans `Tensor.checksum`. Toute modification ultérieure du tenseur invalide ce checksum et déclenche un recalcul.

**Règle 2** : Lors de la conversion d'un tenseur (déquantification, transposition, dtype cast), le nouveau checksum est calculé sur les données converties. L'ancien checksum est préservé dans ExtensionStore pour le round-trip.

**Règle 3** : Si le checksum L1 d'un tenseur chargé ne correspond pas au checksum calculé, la conversion est immédiatement interrompue. Message : "Tenseur '{name}' corrompu. Checksum attendu : {expected}, calculé : {actual}. Le fichier source est peut-être incomplet ou altéré. Vérifiez avec : `umc doctor {source_file}`."

**Règle 4** : Le hash topologique L4 encode la structure du graphe (nœuds, arêtes, types d'opérateurs, ordre topologique) sans les valeurs des tenseurs. Un graphe différent produit un hash différent.

---

## 13. MÉCANISME 3 — EXTENSIONSTORE WITNESS

**Statut** : Obligatoire. Automatique. Après chaque conversion.

### Ce que le Witness Vérifie

Pour chaque champ placé dans ExtensionStore au chargement :
1. Le champ est-il toujours présent après la conversion ?
2. Le SHA256 du champ est-il identique (pas de modification silencieuse) ?
3. Le champ est-il correctement restitué si le format cible le supporte ?

### Format du Rapport Witness

```
ExtensionStore Integrity Report
────────────────────────────────
GGUF@v3/tokenizer.chat_template    : ✅ SHA256 identique
GGUF@v3/rope_scaling.type          : ✅ SHA256 identique
GGUF@v3/rope_scaling.factor        : ✅ SHA256 identique
GGUF@v3/tokenizer.ggml.tokens      : ✅ SHA256 identique (47,234 tokens)
GGUF@v3/general.file_type          : ✅ Restauré dans GGUF cible
────────────────────────────────
Total : 5/5 champs préservés ✅
```

### Comportement si Perte Détectée

- Champ manquant → certificat `partial`, champ listé nommément
- Champ modifié → alerte critique, investigation obligatoire (bug UMC probable)
- Champ restauré incorrectement → certificat `partial`, comparaison avant/après fournie

---

## 14. MÉCANISME 4 — VALIDATION NUMÉRIQUE EXHAUSTIVE

**Statut** : Obligatoire pour tout mode de validation ≠ `none`.

### Méthodes de Comparaison

**Pour les petits tenseurs (< 100M éléments)** :
- Comparaison exhaustive élément par élément
- Calcul de la divergence maximale, moyenne, et percentile 99

**Pour les grands tenseurs (> 100M éléments)** :
- Échantillonnage statistique avec borne de Hoeffding (confiance 99.9%)
- Taille d'échantillon : max(1000, n^0.5) éléments
- La borne statistique est documentée dans le certificat

### Implémentation SIMD Obligatoire

La validation numérique doit utiliser SIMD pour être performante sur les grands modèles.

```
Détection runtime obligatoire (pas de flags globaux) :
x86_64 :
  is_x86_feature_detected!("avx512f") → max_divergence_avx512()
  is_x86_feature_detected!("avx2")    → max_divergence_avx2()    [8 F32/cycle]
  is_x86_feature_detected!("sse4.1")  → max_divergence_sse4()
  fallback                             → max_divergence_scalar()

aarch64 :
  is_aarch64_feature_detected!("neon") → max_divergence_neon()
  fallback                              → max_divergence_scalar()
```

### Seuils de Tolérance par Conversion

Ces seuils ne sont pas arbitraires. Ils sont calculés à partir des précisions théoriques des types numériques.

| Conversion | atol | rtol | Justification |
|------------|------|------|---------------|
| F32 → F32 | 0.0 | 0.0 | Copie lossless |
| F16 → F32 | 0.0 | 0.0 | Élargissant lossless |
| F32 → F16 | 1e-3 | 5e-4 | ULP de FP16 ≈ 9.77e-4 |
| F32 → BF16 | 8e-3 | 4e-3 | ULP de BF16 ≈ 7.8e-3 |
| F32 → Q8_0 | 5e-3 | 2e-3 | Quantification 8-bit |
| F32 → Q4_K_M | 1e-2 | 5e-3 | Quantification 4-bit par blocs |
| Double quantification | 2e-2 | 1e-2 | Cumulatif (deux passes) |

**Profils de validation** :
- `PROFILE_STRICT` (médical, finance) : seuils divisés par 10
- `PROFILE_STANDARD` (usage général) : seuils ci-dessus
- `PROFILE_PERMISSIVE` (prototypage, edge) : seuils multipliés par 5

**Règle** : si un tenseur dépasse sa tolérance, la conversion est **bloquée**. UMC identifie nommément le tenseur fautif et sa divergence mesurée. L'utilisateur peut forcer, mais le certificat sera `partial`.

---

## 15. MÉCANISME 5 — ROUND-TRIP AUTOMATIQUE

**Statut** : Optionnel. Activé avec `--validate strict` ou `--certify`.

### Processus

1. Conversion A → B (exécutée normalement)
2. Conversion B → A (reconversion vers le format source)
3. Comparaison : SHA256(A_original) vs SHA256(A_reconstruit)

### Interprétation des Résultats

| Résultat | Signification | Certificat |
|----------|---------------|------------|
| SHA256 identique | Round-trip parfait | `full` |
| SHA256 différent, divergence < seuil | Round-trip sémantique | `full` avec note |
| SHA256 différent, divergence dans la borne du type | Round-trip documenté | `partial` avec divergence |
| SHA256 différent, divergence hors borne | Bug potentiel | Alerte, investigation |
| Round-trip structurellement impossible (format compilé) | Attendu et documenté | `partial` avec explication |

### Cas Normaux de SHA256 Différent

Ces cas produisent un SHA256 différent mais sont NORMAUX et documentés :
- GGUF Q4_K_M → SafeTensors F16 → GGUF Q4_K_M : les paramètres Q4_K_M sont restaurés depuis ExtensionStore, mais le padding interne peut différer. La divergence sur les poids est 0 (restauration exacte), mais le SHA256 peut différer à cause de l'alignement.
- Tout format utilisant de la compression (ZIP, HDF5) : la recompression peut produire un binaire différent.

Ces cas sont pre-documentés dans UMC et ne déclenchent pas d'alerte.

---

## 16. MÉCANISME 6 — CONFORMITY CHECK

**Statut** : Obligatoire pour chaque format cible qui a un validateur officiel.

### Validateurs par Format

| Format | Validateur | Ce qui est vérifié |
|--------|------------|-------------------|
| ONNX | `onnx.checker.check_model()` | Opsets, shapes, types, cohérence |
| TFLite | `tflite_verify` | Ops supportés, schema FlatBuffer |
| CoreML | `coremltools.models.utils.load_spec()` | Parsing, types, opérateurs |
| GGUF | Loader UMC lui-même (auto-test) | Tous les tenseurs lisibles, métadonnées cohérentes |
| SafeTensors | Header JSON parseable + offsets valides | Cohérence header/données |

### Comportement si Validateur Échoue

1. Le fichier de sortie n'est PAS remis à l'utilisateur.
2. L'erreur complète du validateur est transmise.
3. UMC tente une correction automatique (si connue) et réessaie une fois.
4. Si la correction échoue : message précis avec la cause.
5. Le fichier temporaire est supprimé.

### Comportement si Validateur Non Disponible

- Pour les outils externes (tflite_verify, etc.) : si non installé, le check est skippé avec un warning.
- Message : "Validateur {outil} non trouvé. Conformité non vérifiée. Installez {outil} pour une validation complète."
- Le certificat inclut la mention "Conformité non vérifiée (outil manquant)".

---

## 17. MÉCANISME 7 — PIPELINE WATCHDOG

**Statut** : Obligatoire. Actif pendant toute la durée de la conversion.

### Ce que le Watchdog Surveille

**Heartbeats des threads** :
- Chaque thread (Reader, Transformer, Writer) envoie un heartbeat toutes les 5 secondes.
- Si un heartbeat manque pendant 30 secondes : le thread est considéré bloqué.
- Action : tue le thread, le relance. Jusqu'à 3 redémarrages.
- Après 3 échecs : la conversion est abandonnée avec un rapport détaillé.

**Utilisation mémoire** :
- RAM utilisée (RSS) surveillée toutes les 10 secondes.
- Si RSS > 90% de la RAM disponible : réduction automatique du parallélisme (moins de workers Rayon).
- Si RSS > 95% : mise en pause du pipeline, attente que le GC libère de la mémoire.

**Débit d'écriture** :
- Débit mesuré toutes les 30 secondes.
- Si débit < 10 Mo/s pendant > 60 secondes : alerte "Débit d'écriture anormal. Vérifiez l'espace disque et les permissions."

**Annulations** :
- UMC vérifie le CancellationToken toutes les 2 secondes dans chaque thread.
- Si annulation demandée : tous les threads s'arrêtent proprement, le fichier temporaire est supprimé.

### Implémentation CancellationToken

```rust
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}
impl CancellationToken {
    pub fn cancel(&self) { self.cancelled.store(true, Ordering::SeqCst); }
    pub fn is_cancelled(&self) -> bool { self.cancelled.load(Ordering::SeqCst) }
}
```

Le CancellationToken doit être vérifié dans TOUS les threads, à intervalles réguliers. Un thread qui ne vérifie pas le CancellationToken est un bug.

---

## 18. MÉCANISME 8 — CHECKPOINTING ET REPRISE

**Statut** : Obligatoire pour les conversions > 1 Go.

### Format du Checkpoint

```json
{
  "job_id": "uuid",
  "source_file": "/path/to/source",
  "source_hash": "sha256:...",
  "last_tensor_name": "model.layers.15.mlp.down_proj.weight",
  "tensors_done": 52428,
  "bytes_done": 3221225472,
  "output_file": "/path/to/output.umc_tmp",
  "output_offset": 3221225472,
  "output_partial_hash": "partial_sha256_state",
  "saved_at": 1720000000,
  "umc_version": "3.0.0"
}
```

### Règles de Checkpointing

**Fréquence** : Toutes les 30 secondes pour les modèles < 10 Go. Toutes les 60 secondes pour les modèles > 10 Go.

**Pattern write-to-temp + atomic rename** :
- Écriture dans `{output_path}.{pid}.umc_tmp`
- Checkpoint dans `{output_path}.{pid}.umc_checkpoint`
- À la fin : atomic rename `{output_path}.{pid}.umc_tmp` → `{output_path}`
- Le fichier final est TOUJOURS valide ou absent. Jamais corrompu.

**Reprise** :
1. Détecter le checkpoint au démarrage.
2. Vérifier que le fichier source n'a pas changé (SHA256).
3. Vérifier l'intégrité du fichier de sortie partiel.
4. Reprendre depuis le `last_tensor_name`.
5. Si le checkpoint est corrompu : recommencer depuis le début.
6. Après 3 échecs consécutifs au même tenseur : identifier le tenseur problématique, bloquer, notifier l'utilisateur.

---

## 19. MÉCANISME 9 — CERTIFICAT DE CONVERSION

**Statut** : Optionnel (`--certify`). Jamais émis si un mécanisme de vérification a échoué.

### Contenu Obligatoire du Certificat

```json
{
  "schema_version": "3.0",
  "umc_version": "3.0.0",
  "timestamp": 1720000000,

  "source": {
    "format": "GGUF",
    "version": "v3",
    "sha256": "a1b2c3...",
    "size_bytes": 4920000000,
    "architecture": "llama",
    "num_parameters": 8030000000,
    "num_tensors": 81920
  },

  "target": {
    "format": "ONNX",
    "version": "opset21",
    "sha256": "f6e5d4...",
    "size_bytes": 13800000000,
    "num_tensors": 81920
  },

  "conversion": {
    "path": ["GGUF", "ONNX"],
    "duration_seconds": 14.2,
    "cpu_time_ms": 14200,
    "peak_ram_bytes": 420000000
  },

  "validation": {
    "dry_run": "passed",
    "checksums_l1": "passed",
    "checksums_l3": "passed",
    "extension_store_witness": "passed",
    "numeric_validation": "passed",
    "max_divergence": 2.3e-7,
    "round_trip": "passed",
    "conformity_check": "passed",
    "tools_used": ["onnx.checker"]
  },

  "roundtrip_level": "semantic",
  "roundtrip_max_divergence": 2.3e-7,
  "trust_statement": "...",

  "extensions_preserved": [
    "GGUF@v3/tokenizer.chat_template",
    "GGUF@v3/rope_scaling.type"
  ],

  "losses_documented": [],

  "warnings": [],

  "certificate_type": "full",

  "signature": "hex(ed25519_signature)",
  "public_key": "hex(verifying_key)",
  "verify_url": "https://umc.dev/verify/{cert_id}"
}
```

### Règles de Certification

**Règle 1** : Le certificat `full` exige que TOUS les mécanismes 1-6 aient passé sans anomalie.

**Règle 2** : Le certificat `partial` est émis quand au moins un mécanisme a produit un avertissement (pas un blocage). La liste des pertes est documentée exhaustivement.

**Règle 3** : Le certificat n'est JAMAIS émis si un mécanisme a déclenché un blocage. Une conversion bloquée = pas de certificat.

**Règle 4** : La signature ed25519 est calculée sur le hash SHA256 du contenu JSON du certificat (sans le champ `signature`). Elle prouve que le certificat a été émis par UMC, pas que la conversion est "légalement parfaite" (distinction honnête obligatoire).

**Règle 5** : Le certificat est accessible publiquement sans authentification via `/v1/certificates/{id}`. La vérification cryptographique est accessible via `/v1/certificates/{id}/verify`.

**Trust Statement selon le niveau de round-trip** :
- Bit-identical : "Ce rapport certifie que source et cible sont bit-identiques (SHA256 identiques). UMC v{N} a effectué cette vérification."
- Sémantique : "Ce rapport certifie que la conversion est sémantiquement correcte. Divergence maximale mesurée : {δ}. Ce rapport ne garantit pas la correction fonctionnelle pour tous les cas d'usage — il garantit que UMC a effectué les vérifications documentées."
- Structurel : "Ce rapport certifie que la structure du modèle est préservée. Une validation fonctionnelle sur votre cas d'usage est recommandée."

---

## 20. MÉCANISME 10 — AUDIT TRAIL IMMUTABLE

**Statut** : Obligatoire. Automatique. Toujours écrit, même en cas d'échec.

### Ce qui est Enregistré

Chaque conversion déclenche l'écriture d'une entrée dans l'Audit Trail :
- Qui : user_id, api_key_id
- Quand : timestamp Unix
- Source : chemin, SHA256, taille, format
- Cible : chemin, SHA256, taille, format
- Chemin de conversion emprunté
- Résultats de tous les mécanismes de vérification
- Version d'UMC
- Durée, RAM peak, CPU time
- Warnings et erreurs
- ID du certificat (si émis)

### Immutabilité par Hash Chaining

```
entry[n].chain_hash = SHA256(entry[n-1].chain_hash || entry[n].content_hash)
```

Chaque entrée référence la précédente. Toute modification d'une entrée est détectable en recalculant la chaîne. C'est un log tamper-evident.

**Vérification** : `umc lineage model.gguf --verify-chain` recalcule toute la chaîne et vérifie son intégrité.

---

# PARTIE IV — STANDARDS PAR FORMAT

---

## 21. STANDARD DE COMPLÉTUDE PAR FORMAT

Un format est "supporté" par UMC uniquement quand TOUS ces critères sont satisfaits :

```
□ La spécification officielle complète a été lue et documentée
□ Toutes les versions du format sont supportées (ou documentées comme non supportées)
□ Tous les dtypes documentés dans la spec sont supportés (ou mappés)
□ Tous les champs de métadonnées sont extraits (ou stockés dans ExtensionStore)
□ Tous les types de tokenizers associés sont supportés (si applicable)
□ Un loader natif Rust est implémenté et fuzzé (cargo-fuzz, 60s minimum en CI)
□ Un saver natif Rust est implémenté et validé par le validateur officiel
□ Des tests round-trip sur des modèles réels passent (SHA256 identique)
□ Les benchmarks sont mesurés et documentés (3 machines différentes)
□ Le format est maintenu : une personne est responsable des mises à jour de spec
□ La fiche de complétude est publiée sur umc.dev/formats/{format}
```

Un format partant à < 100% sur ces critères est documenté comme "support partiel" avec la liste des lacunes.

### Fiche de Complétude (Obligatoire pour chaque format)

```
Format : GGUF
Score : 100/100

Versions supportées :
  v1 ✅  v2 ✅  v3 ✅

Dtypes (24/24) :
  F32 ✅  F16 ✅  BF16 ✅  Q4_0 ✅  Q4_1 ✅
  Q5_0 ✅  Q5_1 ✅  Q8_0 ✅  Q2K ✅  Q3KS ✅
  Q3KM ✅  Q3KL ✅  Q4KS ✅  Q4KM ✅  Q5KS ✅
  Q5KM ✅  Q6K ✅  Q8K ✅  I8 ✅  I16 ✅
  I32 ✅  I64 ✅  F64 ✅  Bool ✅

Métadonnées (47/47 champs) : ✅
Tokenizers (BPE ✅, SentencePiece ✅, WordPiece ✅)
Round-trip : SHA256 identique ✅
Modèles testés : Phi-2, Mistral 7B, Llama 3.1 8B, Llama 3.1 70B
Validateur officiel : llama.cpp ✅
Fuzzing : 60s en CI, 0 crash ✅
Responsable : {nom}
Dernière mise à jour spec : GGUF v3, 2024-01
```

---

## 22. MATRICE DE COUVERTURE DES CONVERSIONS

Pour chaque chemin de conversion, UMC maintient une entrée dans la matrice qui documente :
- Le chemin complet (étapes intermédiaires si nécessaire)
- Le niveau de round-trip garanti
- La divergence maximale connue
- La commande exacte
- Un test automatisé en CI

### Chemins Critiques (Phase 0 — Obligatoires)

| Source | Cible | Niveau | Divergence | Test CI |
|--------|-------|--------|------------|---------|
| GGUF | ONNX | Sémantique | < 1e-6 | ✅ |
| GGUF | SafeTensors | Sémantique | < 1e-7 | ✅ |
| ONNX | GGUF | Sémantique | < 1e-6 | ✅ |
| ONNX | SafeTensors | Sémantique | < 1e-7 | ✅ |
| SafeTensors | GGUF | Sémantique | < 1e-7 | ✅ |
| SafeTensors | ONNX | Sémantique | < 1e-6 | ✅ |
| PyTorch | SafeTensors | Sémantique | < 1e-7 | ✅ |
| PyTorch | ONNX | Sémantique | < 1e-6 | ✅ |
| AWQ | SafeTensors | Sémantique | < 1e-2 | ✅ |
| GPTQ | SafeTensors | Sémantique | < 1e-2 | ✅ |

### Règle de la Matrice

Chaque chemin listé dans la matrice doit avoir :
1. Un test automatisé en CI qui vérifie la borne de divergence
2. Un test round-trip si les deux formats sont bidirectionnels
3. Une documentation publique sur umc.dev/conversions/{source}/{target}

---

## 23. BORNES DE DIVERGENCE OFFICIELLES

Ces bornes sont les engagements mathématiques d'UMC. Elles ne peuvent pas être modifiées sans une release majeure et une communication publique.

### Bornes par Type de Conversion

**Conversions sans perte (δ = 0)** :
- F16 → F32 (élargissant)
- BF16 → F32 (élargissant)
- F32 → F64 (élargissant)
- Tout entier signé → entier signé plus large
- Tout entier non signé → entier non signé plus large

**Conversions dtype flottant avec perte bornée** :
- F32 → F16 : δ_max = 4.88e-4 (ULP de FP16)
- F32 → BF16 : δ_max = 7.81e-3 (ULP de BF16)
- F64 → F32 : δ_max = 5.96e-8 (ULP de FP32)
- F32 → FP8 E4M3 : δ_max = 1.56e-2
- F32 → FP8 E5M2 : δ_max = 3.13e-2

**Conversions de quantification** :
- F32 → Q8_0 : δ_max = 5e-3 (quantification 8-bit par blocs de 32)
- F32 → Q4_K_M : δ_max = 8.7e-3 (quantification 4-bit K-quants)
- F32 → Q4_0 : δ_max = 1e-2
- AWQ 4-bit → F32 : δ_max = 1e-2 (dépend du calibration dataset)
- GPTQ 4-bit → F32 : δ_max = 1e-2 (dépend de l'ordering)
- NF4 → F32 (via table) : δ_max = 1e-4 (table de correspondance exacte)

**Décompositions d'opérateurs** :
- RmsNorm → 7 ops ONNX : δ_max = 1e-7 (arithmétique flottante non-associative)
- RoPE standard → ops ONNX : δ_max = 1e-7
- SiLU → 2 ops : δ = 0 (décomposition exacte)
- GeluApprox → 8 ops : δ = 0 (décomposition exacte)

**Conversions cumulées (doubles passages)** :
- Q4_K_M → F32 → AWQ 4-bit : δ_max = 1.9e-2
- Q4_K_M → F32 → GPTQ 4-bit : δ_max = 1.9e-2
- NF4 → F32 → INT8 : δ_max variable selon les valeurs

---

# PARTIE V — RÈGLES OPÉRATIONNELLES

---

## 24. RÈGLES DU PIPELINE DE CONVERSION

### Architecture 3-Thread Obligatoire

Le pipeline est toujours composé de trois threads simultanés :
1. **Reader** : lecture des tenseurs depuis le fichier source (mmap ou streaming)
2. **Transformer** : conversion des tenseurs (rayon, parallélisme de données, SIMD)
3. **Writer** : écriture dans le fichier de sortie (TempOutputFile + atomic rename)

**Communication** : canaux Crossbeam bornés avec timeout.
- Capacité par défaut : 4 messages (ajustée selon la RAM disponible)
- Timeout par opération : 120 secondes (configurable)
- select! de Crossbeam pour gérer simultanément les messages ET le CancellationToken (anti-deadlock)

### Règles Mmap — Zéro Copie

```
RÈGLE ABSOLUE : Les données des tenseurs > mmap_threshold ne sont jamais
                copiées en RAM lors du chargement.

Par défaut : mmap_threshold = 64 Mo

Violations interdites :
  ❌ let data = std::fs::read(path)?;         (charge tout le fichier en RAM)
  ❌ let bytes = mmap[offset..len].to_vec();  (copie le tenseur en RAM)

Correct :
  ✅ TensorData::MmapView { mmap: Arc::clone(&mmap), offset, length }

Exception légitime :
  Le Writer peut matérialiser un tenseur pour l'écrire dans le fichier de sortie.
  Cette copie est locale et éphémère.
```

**Sur les grands modèles (> RAM disponible)** :
- UMC utilise mmap. Le fichier n'est jamais chargé en RAM.
- L'OS gère le cache de pages. UMC n'essaie pas de le contrôler.
- La revendication "UMC utilise ~200 Mo de RAM" est précisée comme "200 Mo de structures de données (hors cache OS)".

### PipelineConfig Auto-Détection

```
PipelineConfig::auto() :
  shard_workers = min(num_cpus, num_shards, ram_available_gb / 2)
  tensor_threads = num_cpus
  tile_size_bytes = if ram_available_gb < 8 { 16 Mo } else if < 32 { 32 Mo } else { 64 Mo }
  channel_capacity = 4
  chunk_size_bytes = if ram_available_gb < 8 { 16 Mo } else { 64 Mo }
  prefetch_count = if ram_available_gb < 8 { 1 } else { 2 }
  op_timeout_secs = 120
  watchdog_secs = 30

IMPORTANT : Utiliser ram_available (pas ram_total) pour éviter les OOM
sur les machines cloud avec 64 vCPUs partagés.
```

### TempOutputFile — Atomic Rename Obligatoire

```
Séquence obligatoire pour toute écriture de fichier de sortie :

1. Créer /path/to/output.{pid}.umc_tmp
2. Écrire dans ce fichier temporaire
3. Checkpointer l'offset toutes les 30s
4. À la fin : std::fs::rename(tmp, final)  ← atomic sur POSIX
5. Si erreur : Drop de TempOutputFile supprime le .tmp automatiquement

JAMAIS écrire directement dans le fichier final.
Le fichier final est toujours VALIDE ou ABSENT. Jamais CORROMPU.
```

---

## 25. RÈGLES DE SÉCURITÉ DU PARSING

> **Tout fichier entrant est hostile jusqu'à preuve du contraire.**

### Limites Hardcodées (SecurityBounds)

Ces limites sont appliquées à CHAQUE insertion dans TensorStore. Elles ne sont jamais désactivables.

```
max_tensor_count       = 1_000_000
max_metadata_count     = 10_000
max_string_length      = 1_048_576 (1 Mo)
max_shape_rank         = 8
max_tensor_size_bytes  = 100 * 1024^3 (100 Go par tenseur)
max_extension_bytes    = 100 * 1024^2 (100 Mo pour ExtensionStore total)
max_metadata_nesting   = 32 (protobuf depth limit, anti-stack-overflow)
max_compression_ratio  = 1000 (anti-ZIP bomb)
```

**Si une limite est dépassée** : `UmcError::SecurityViolation { field, value, limit }` immédiatement. Jamais d'allocation de la mémoire demandée.

### Protection ZIP Bomb

```rust
fn validate_compression_ratio(compressed: usize, decompressed: usize) -> Result<(), UmcError> {
    if decompressed > compressed * 1000 {
        return Err(UmcError::ZipBomb { compressed, decompressed });
    }
    Ok(())
}
```

### Protection Path Traversal

```rust
fn validate_archive_path(path: &str) -> Result<(), UmcError> {
    // Interdire les composants dangereux
    if path.contains("..") || path.starts_with('/') || path.contains('\0') {
        return Err(UmcError::PathTraversal(path.to_string()));
    }
    Ok(())
}
```

### SafePickleParser (PyTorch)

Le parser pickle de PyTorch n'exécute JAMAIS de code. Il parse uniquement les structures de données avec une whitelist de types autorisés.

```
Types autorisés (whitelist exhaustive) :
  torch.FloatStorage, torch.HalfStorage, torch.BFloat16Storage
  torch.IntStorage, torch.LongStorage, torch.ByteStorage
  torch.ShortStorage, torch.DoubleStorage, torch.BoolStorage
  collections.OrderedDict
  _codecs.encode  (pour les bytes Python)

Profondeur maximale : 32 niveaux

TOUT type non dans la whitelist → UmcError::PickleUnsafeType
```

### Protection SSRF (URLs externes)

```rust
fn validate_url_security(url: &str) -> Result<(), UmcError> {
    // HTTPS obligatoire uniquement
    if parsed.scheme() != "https" {
        return Err(UmcError::InsecureUrl);
    }
    // Blacklist des ranges IP privées
    let blocked = ["169.254.", "10.", "172.16.", ..., "127."];
    for range in &blocked {
        if host.starts_with(range) {
            return Err(UmcError::SsrfAttempt { host });
        }
    }
    Ok(())
}
```

### Fuzzing Obligatoire en CI

Pour chaque format qui parse des fichiers binaires :
- cargo-fuzz (ou AFL++) doit avoir une cible
- Minimum 60 secondes en CI à chaque PR
- Minimum 24 heures en CI quotidien
- 0 crash autorisé

Formats prioritaires : GGUF, ONNX, PyTorch (pickle), SafeTensors, TFLite

---

## 26. RÈGLES DE PERFORMANCE

### Seuils Non Négociables en CI

Ces seuils sont vérifiés automatiquement. Toute régression > 5% bloque la fusion.

| Modèle | Taille | Conversion | Temps max | RAM max |
|--------|--------|------------|-----------|---------|
| Phi-2 | 1.6 Go | GGUF → ONNX | 10s | 2 Go |
| Mistral 7B | 4.1 Go | SafeTensors → GGUF | 30s | 2.5 Go |
| Llama 3.1 8B | 4.9 Go | GGUF → SafeTensors | 35s | 2.5 Go |

Machine de référence pour les benchmarks : AMD EPYC 7763, 256 Go RAM, NVMe SSD.  
Résultats documentés sur 3 machines différentes (serveur haut de gamme, workstation 16 Go, laptop).

### Règles SIMD

```
JAMAIS de flags SIMD globaux dans .cargo/config.toml.
(crash sur AMD Zen 3, Intel Alder Lake E-cores, WASM)

Toujours la détection runtime :
  is_x86_feature_detected!("avx512f") → chemin AVX-512
  is_x86_feature_detected!("avx2")    → chemin AVX2    [PRÉFÉRÉ x86]
  is_x86_feature_detected!("sse4.1")  → chemin SSE4.1
  fallback scalaire                    → TOUJOURS présent

  is_aarch64_feature_detected!("neon") → chemin NEON   [PRÉFÉRÉ ARM]
  fallback scalaire                     → TOUJOURS présent
```

### Règles de Benchmarking

- Utiliser Criterion (warmup + statistiques)
- Mesurer : temps de conversion, RAM peak (RSS), débit Mo/s
- Exécuter sur au minimum 3 tailles de modèle
- Documenter la machine de référence exactement
- Publier les scripts de benchmark : reproductibles par quiconque
- Mettre à jour les benchmarks si un concurrent améliore ses performances

---

## 27. RÈGLES DE GESTION DES ERREURS

### Anatomie d'un Message d'Erreur UMC

```
Structure obligatoire :
  1. Ce qui s'est passé (spécifique, pas générique)
  2. Où (fichier, tenseur, offset si pertinent)
  3. Comment corriger (action concrète)

Exemples :

❌ INTERDIT :
  "Erreur lors de la conversion"
  "Format non supporté"
  "Erreur interne"

✅ OBLIGATOIRE :
  "Format inconnu pour 'model.bin'.
   Magic bytes lus : [0x00, 0x01, 0x02, 0x03].
   Utilisez 'umc formats' pour voir les formats supportés, ou
   '--format <FORMAT>' pour spécifier manuellement."

  "Tenseur 'model.layers.15.mlp.gate_proj.weight' hors limites.
   Offset déclaré : 4294967296 octets.
   Taille du fichier : 4000000000 octets.
   Le fichier est probablement corrompu. Vérifiez avec : 'umc doctor model.gguf'"

  "Outil externe 'trtexec' requis mais introuvable dans PATH.
   Installez TensorRT : https://developer.nvidia.com/tensorrt
   Ou convertissez vers ONNX : 'umc convert model.gguf model.onnx'"
```

### Codes d'Erreur

Chaque erreur a un code unique, stable entre les versions, documenté dans la référence API.

| Code | Catégorie | Signification |
|------|-----------|---------------|
| `UMC_E001` | Format | Magic bytes invalides |
| `UMC_E002` | Format | Version non supportée |
| `UMC_E003` | Format | Format inconnu |
| `UMC_E010` | Tenseur | Checksum invalide |
| `UMC_E011` | Tenseur | Tenseur hors limites |
| `UMC_E012` | Tenseur | Shape incohérente |
| `UMC_E020` | Sécurité | SecurityViolation (tensor_count, etc.) |
| `UMC_E021` | Sécurité | ZIP bomb détectée |
| `UMC_E022` | Sécurité | Path traversal détecté |
| `UMC_E023` | Sécurité | SSRF attempt |
| `UMC_E030` | Conversion | Pas de chemin de conversion |
| `UMC_E031` | Conversion | Divergence hors seuil |
| `UMC_E032` | Conversion | Opérateur non supporté (bloquant) |
| `UMC_E040` | Pipeline | Thread panic |
| `UMC_E041` | Pipeline | Deadlock détecté |
| `UMC_E042` | Pipeline | Timeout |
| `UMC_E050` | Ressource | RAM insuffisante |
| `UMC_E051` | Ressource | Disque insuffisant |
| `UMC_E060` | Outil ext | Outil manquant |
| `UMC_E061` | Outil ext | Outil échoué |

### Hiérarchie des Erreurs

```
ERREURS FATALES (conversion interrompue immédiatement) :
  - Magic bytes invalides
  - Checksum de tenseur incorrect
  - Shape incohérente
  - SecurityViolation
  - Espace disque insuffisant (détecté avant conversion)

AVERTISSEMENTS (conversion continue avec notification) :
  - Opérateur stocké en Custom (non exécutable par la cible)
  - Dtype converti avec perte documentée
  - Champ de métadonnée stocké en ExtensionStore
  - Divergence dans les tolérances

JAMAIS silencieux :
  - Tout avertissement est visible dans le rapport de sortie et le certificat
  - Tout avertissement est enregistré dans l'Audit Trail
```

---

## 28. RÈGLES DU CERTIFICAT ET DE LA CERTIFICATION

### Ce que le Certificat Prouve (et ne Prouve Pas)

**Ce que le certificat prouve** :
- UMC version {N} a effectué la conversion
- Les fichiers source et cible ont les SHA256 documentés
- Les validations documentées ont été effectuées
- La divergence maximale mesurée est {δ}

**Ce que le certificat ne prouve PAS** :
- La "valeur légale" au sens juridique (la loi varie par juridiction)
- La correction fonctionnelle pour VOTRE cas d'usage spécifique
- Que le modèle produira les mêmes résultats sur du matériel différent

**Ce langage est OBLIGATOIRE dans le trust_statement** : jamais de revendication de "valeur légale" ou de "certification FDA" sans processus de validation tiers approprié.

### Règles de Signature

- Algorithme : ed25519
- La clé privée est dans un fichier séparé (jamais dans le code source)
- La clé privée est chargée au démarrage et stockée dans AppState
- La signature est calculée sur SHA256(contenu_JSON) (pas sur le contenu brut)
- La clé publique est dans le certificat ET sur umc.dev/public-key

### Révocation

Un certificat peut être révoqué si :
- Un bug est découvert dans la version d'UMC qui l'a produit
- Le fichier source ou cible est compromis
- La clé de signature est compromise

La révocation est notifiée proactivement (email) et accessible via `/v1/certificates/{id}`.  
La révocation ne supprime pas le certificat — elle l'annote avec `revoked_at` et `revoked_reason`.

---

# PARTIE VI — DIMENSIONS DE COMPLÉTUDE

---

## 29. LES 12 DIMENSIONS DE COMPLÉTUDE UMC

UMC est "complet" uniquement quand les 12 dimensions suivantes sont satisfaites.

### Dimension 1 — Couverture Exhaustive des Formats

Chaque format supporté est traité à 100% de sa spécification :
- Toutes les versions (GGUF v1/v2/v3, ONNX opset 1-21)
- Tous les dtypes documentés
- Tous les opérateurs (ou décompositions documentées)
- Toutes les métadonnées (mappées ou ExtensionStore)
- Tous les tokenizers associés
- Tous les schémas de quantification

Score de complétude affiché publiquement par format.

### Dimension 2 — Couverture Exhaustive des Conversions

66 chemins directs testés et documentés.  
Des centaines de chemins indirects via le graphe Dijkstra.  
Chaque chemin : test automatisé en CI, borne de divergence, documentation.

### Dimension 3 — Validation à Tous les Niveaux

Les 10 mécanismes de vérification sont tous opérationnels.

### Dimension 4 — Performance Adaptative

UMC gère tous les scénarios : modèle < 1 Go, modèle > 200 Go, RAM limitée, disque lent, fichier sur S3.

### Dimension 5 — Gestion Exhaustive des Erreurs

Toutes les erreurs documentées et gérées avec des messages précis et des suggestions d'action.

### Dimension 6 — Interfaces Multiples

CLI, API REST, SDK Python, SDK JavaScript, GitHub Action.

### Dimension 7 — Documentation Exhaustive

Chaque commande, chaque endpoint, chaque format, chaque conversion, chaque erreur est documenté.

### Dimension 8 — Sécurité de Bout en Bout

Parsing défensif, fuzzing, SSRF protection, auth JWT + API Key, Rate Limiting, Audit Trail.

### Dimension 9 — Extensibilité

Plugin System pour ajouter des formats sans modifier UMC core.

### Dimension 10 — Observabilité

Prometheus, Grafana, logs JSON, Audit Trail, dashboard public.

### Dimension 11 — Communauté

Open source Apache 2.0, GitHub, Discord, Bounty Program.

### Dimension 12 — Business Model Transparent

Core gratuit à jamais. Cloud et Enterprise payants. Revenus publiés.

---

## 30. FONCTIONNALITÉS AVANCÉES OBLIGATOIRES

Ces fonctionnalités distinguent UMC d'un simple outil de conversion et font sa valeur sur le long terme.

### Fonctionnalités de Confiance

**Undo Mode** :
- Chaque conversion est réversible si les fichiers sont disponibles.
- `umc undo {conversion-id}` : retrouve le chemin de conversion inverse et l'applique.
- Si le round-trip n'est pas parfait (format compilé) : explication précise et suggestion.

**Explain Mode** :
- `umc explain {conversion-id}` : déroulement complet de la conversion, chaque décision, chaque décomposition, chaque champ dans ExtensionStore.
- Utile pour le debugging, les audits, la formation.

**Compare Mode** :
- `umc compare model_v1.gguf model_v2.gguf` : tenseurs modifiés, divergences, métadonnées changées.
- Idéal pour valider un fine-tuning, une quantification, une conversion.

**Model Health Score** :
- Score 0-100 basé sur l'intégrité des tenseurs, la complétude des métadonnées, l'absence d'anomalies.
- Affiché dans `umc inspect` et sur le Hub.

### Fonctionnalités d'Analyse

**Format Recommendation Engine** :
```
umc recommend --hardware "iphone-15" --task "text-generation" --model-size 7B
→ Recommande CoreML INT8, 3.8 Go, commande exacte fournie
```

**Conversion Budget** :
- `umc budget set 50 EUR` → alertes à 80%, blocage à 100%.
- Évite les surprises sur la facture cloud.

**Dry-Run avec Estimation** :
- Avant chaque conversion, affichage de : format détecté, taille sortie estimée, RAM nécessaire, temps estimé, compatibilité, avertissements.

### Fonctionnalités de Traçabilité

**Provenance Explorer** :
- `umc lineage model-final.gguf` affiche l'arbre généalogique complet des conversions.
- Chaque nœud = un fichier avec son SHA256, son format, sa date.
- Chaque arête = une conversion avec son certificat.

**Conversion Receipt** :
- Résumé humain de chaque conversion (pas juste le JSON).
- Affiché dans le terminal + téléchargeable en PDF.

**Safety Scanner** :
- `umc scan model.pt` : vérifie l'absence de code exécutable malveillant (pickle injecté, code dans les métadonnées).
- Particulièrement important pour les formats pickle (PyTorch).
- Conversion automatique vers SafeTensors proposée pour neutraliser le risque.

### Fonctionnalités de Productivité

**Batch Mode** :
- `umc convert-batch --input-dir ./models/ --target onnx --jobs 8`
- Détection automatique, parallélisation, rapport de synthèse.

**Watch Mode** :
- `umc watch model.safetensors --targets onnx,gguf --output-dir ./converted/`
- Conversion automatique à chaque modification du fichier.

**Playbook System** :
```yaml
# playbook: llama-to-iphone.yaml
name: "Llama 3.1 → iPhone 15 Pro"
source: "*.gguf"
target: "coreml"
options:
  dtype: "int8"
  validate: "strict"
  certify: true
```
- `umc playbook llama-to-iphone.yaml`
- Playbooks partagés par la communauté.

**GitHub Action** :
```yaml
- uses: umc-dev/umc-action@v1
  with:
    source: models/*.safetensors
    targets: onnx,gguf,tflite
    certify: true
```

---

## 31. CHECK-LIST AVANT TOUTE DÉCISION DE CONVERSION

### Check-list Technique (Développement)

Avant d'implémenter un nouveau format :
- [ ] Spécification officielle lue en entier et documentée
- [ ] Toutes les versions identifiées et supportées (ou exclusions documentées)
- [ ] Mapping IR défini (chaque champ → IR ou ExtensionStore)
- [ ] Tests écrits avant le code (TDD)
- [ ] Test round-trip bit-identical sur fichier réel
- [ ] Fuzzing 60s en CI
- [ ] Benchmarks mesurés sur 3 machines
- [ ] Fiche de complétude publiée

Avant d'implémenter un nouveau chemin de conversion :
- [ ] Dry-run simulé (compatibilité, estimations)
- [ ] Borne de divergence calculée et documentée
- [ ] Test automatisé en CI
- [ ] Documentation sur umc.dev/conversions/{source}/{target}
- [ ] Ajout dans la matrice de couverture

### Check-list Opérationnelle (Chaque Conversion)

UMC exécute automatiquement ces vérifications. Elles sont documentées ici pour la transparence.

```
AVANT :
  [✓] Dry-run (compatibilité, ressources, estimations)
  [✓] Vérification espace disque
  [✓] Vérification RAM disponible

PENDANT :
  [✓] Checksums L1 par tenseur (xxHash64)
  [✓] ExtensionStore rempli pour chaque champ non-mappable
  [✓] Watchdog actif (heartbeats, RAM, débit)
  [✓] Checkpoints toutes les 30s
  [✓] CancellationToken vérifié régulièrement

APRÈS :
  [✓] Validation numérique (divergence par tenseur)
  [✓] ExtensionStore Witness (préservation vérifiée)
  [✓] Conformity Check (validateur officiel du format)
  [✓] Round-trip (si --validate strict ou --certify)
  [✓] Certificat émis (si --certify et tout vert)
  [✓] Audit Trail mis à jour
  [✓] Fichier temp renommé atomiquement
```

---

# APPENDICE — FLUX COMPLET DE VÉRIFICATION

```
DÉBUT DE CONVERSION
       │
       ▼
[1] DRY-RUN AUTOMATIQUE
  ├── ✅ Compatible → continuer
  ├── ⚠️ Réserves (< 10% incompatible) → warning → continuer
  └── ❌ Incompatibilité extrême (> 10%) → BLOQUER (sauf --force)
       │
       ▼
[8] CHECKPOINT (toutes les 30s)
       │
       ▼
PIPELINE DE CONVERSION
  ├── [7] WATCHDOG (surveillance continue heartbeats/RAM/I/O)
  ├── [2] CHECKSUMS L1 (xxHash64 par tenseur, à chaque lecture)
  └── [3] EXTENSIONSTORE WITNESS (remplissage + vérification)
       │
       ▼
[4] VALIDATION NUMÉRIQUE
  ├── ✅ Tous tenseurs dans les seuils → continuer
  └── ❌ Un tenseur hors seuil → BLOQUER (sauf --force)
       │
       ▼
[6] CONFORMITY CHECK (validateur officiel du format cible)
  ├── ✅ Validateur OK → continuer
  └── ❌ Validateur échoue → BLOQUER + tentative de correction automatique
       │
       ▼
[5] ROUND-TRIP (si --validate strict ou --certify)
  ├── ✅ SHA256 identique → Niveau Bit-Identical
  ├── ✅ Divergence dans seuil → Niveau Sémantique
  └── ⚠️ SHA256 différent (documenté) → Niveau Sémantique avec note
       │
       ▼
[9] CERTIFICAT (si --certify et tout vert)
  ├── "full" : tous mécanismes parfaits
  └── "partial" : pertes documentées, seuils respectés
       │
       ▼
[10] AUDIT TRAIL
  └── Toujours écrit, même en cas d'échec
       │
       ▼
FICHIER FINAL REMIS À L'UTILISATEUR
```

---

# GLOSSAIRE TECHNIQUE

| Terme | Définition Précise |
|-------|-------------------|
| IR (Intermediate Representation) | Sur-ensemble évolutif de tous les formats supportés. Pivot universel entre formats. |
| ExtensionStore | Stockage limité (100 Mo) des champs exclusifs à chaque format, avec clés namespaced. Garantit zéro perte d'information. |
| GraphTemplate | Template de reconstruction de graphe pour les formats weights-only (GGUF, SafeTensors). |
| WeightsOnly | Format sans graphe de calcul explicite (GGUF, SafeTensors, AWQ...). |
| CanonicalQuantization | Représentation canonique de la quantification, pivot entre tous les schémas. |
| SecurityBounds | Limites hardcodées sur tous les champs lus depuis les fichiers. Anti-DoS. |
| CancellationToken | Mécanisme d'annulation coopérative que tous les threads vérifient régulièrement. |
| Atomic Rename | Pattern write-to-temp + rename atomique. Le fichier est valide ou absent, jamais corrompu. |
| SSE (Server-Sent Events) | Alternative unidirectionnelle aux WebSockets pour le streaming de progression. |
| Dry-Run | Simulation exhaustive de la conversion avant exécution. Détecte tous les problèmes sans conversion. |
| Conformity Check | Vérification du fichier converti par les validateurs officiels du format cible. |
| Round-Trip | Test A → B → A. SHA256(A_original) == SHA256(A_reconstruit) si niveau 1. |
| ProvenanceChain | Journal d'audit immutable par hash chaining. Tamper-evident. |
| ConversionHints | Métadonnées supplémentaires transmises avec l'IR pour guider le saver cible (logique de paire). |
| Recipe Saver | Générateur de configuration pour les formats propriétaires non natifs (TensorRT, QNN, TVM). |
| Format Compilé | Format binaire optimisé pour un matériel spécifique, non destiné à être reconverti. Cible uniquement. |
| Best-effort | Conversion possible avec perte documentée, round-trip non garanti en SHA256. |

---

*UMC — Règles d'Excellence en Conversion v1.0*  
*Document normatif — Obligatoire pour tout contributeur et toute décision d'implémentation*  
*"UMC ne ment jamais. Chaque perte est mesurée, bornée, documentée, certifiée."*