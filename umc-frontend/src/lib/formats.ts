import type { BrandKey } from "./brands";

export type Hardware = "CPU" | "GPU" | "NPU" | "Mobile" | "Edge" | "Apple Silicon" | "Web";

export interface FormatDef {
  slug: string;
  name: string;
  ext: string;
  color: string;           // primary brand color
  creator: BrandKey;
  year: number;
  hardware: Hardware[];
  use: { fr: string; en: string };
  why: { fr: string; en: string };       // pourquoi créé
  usedBy: BrandKey[];      // entreprises qui l'utilisent
  size?: string;
}

export const FORMATS: FormatDef[] = [
  {
    slug: "gguf", name: "GGUF", ext: ".gguf", color: "#00FF94", creator: "ggerganov", year: 2023,
    hardware: ["CPU", "GPU"],
    use: { fr: "Inférence CPU/GPU avec llama.cpp", en: "CPU/GPU inference via llama.cpp" },
    why: { fr: "Successeur de GGML : un seul fichier mmap-able, métadonnées intégrées, versionné.", en: "Successor to GGML: single mmap file, embedded metadata, versioned." },
    usedBy: ["meta", "mistral", "microsoft", "alibaba", "deepseek"],
  },
  {
    slug: "onnx", name: "ONNX", ext: ".onnx", color: "#005CED", creator: "microsoft", year: 2017,
    hardware: ["CPU", "GPU", "NPU", "Edge"],
    use: { fr: "Format d'échange universel entre frameworks", en: "Universal interchange format between frameworks" },
    why: { fr: "Créé par Microsoft + Meta pour briser la dépendance à un framework unique.", en: "Created by Microsoft + Meta to break single-framework lock-in." },
    usedBy: ["microsoft", "meta", "intel", "amd", "qualcomm", "huggingface"],
  },
  {
    slug: "safetensors", name: "SafeTensors", ext: ".safetensors", color: "#FFD21E", creator: "huggingface", year: 2022,
    hardware: ["CPU", "GPU"],
    use: { fr: "Stockage sûr, lecture mmap zéro-copie", en: "Safe storage, zero-copy mmap loading" },
    why: { fr: "Remplace pickle PyTorch — impossible d'exécuter du code arbitraire à l'ouverture.", en: "Replaces PyTorch pickle — no arbitrary code execution on load." },
    usedBy: ["huggingface", "meta", "mistral", "stability", "google"],
  },
  {
    slug: "pytorch", name: "PyTorch", ext: ".pt / .pth", color: "#EE4C2C", creator: "meta", year: 2016,
    hardware: ["CPU", "GPU"],
    use: { fr: "Format d'entraînement de référence en recherche", en: "Reference training format in research" },
    why: { fr: "Format natif des poids PyTorch, basé sur pickle Python.", en: "PyTorch native weights, built on Python pickle." },
    usedBy: ["meta", "openai", "anthropic", "tesla", "stability", "huggingface"],
  },
  {
    slug: "coreml", name: "CoreML", ext: ".mlpackage", color: "#A2AAAD", creator: "apple", year: 2017,
    hardware: ["Apple Silicon", "Mobile"],
    use: { fr: "Inférence sur iPhone, iPad, Mac (CPU/GPU/ANE)", en: "Inference on iPhone, iPad, Mac (CPU/GPU/ANE)" },
    why: { fr: "Exploite le Neural Engine d'Apple pour une inférence on-device privée.", en: "Leverages Apple's Neural Engine for private on-device inference." },
    usedBy: ["apple"],
  },
  {
    slug: "tensorrt", name: "TensorRT", ext: ".engine", color: "#76B900", creator: "nvidia", year: 2017,
    hardware: ["GPU"],
    use: { fr: "Inférence GPU NVIDIA optimisée (Datacenter, Jetson)", en: "Optimized NVIDIA GPU inference (Datacenter, Jetson)" },
    why: { fr: "Compilation spécifique à chaque GPU NVIDIA pour latence minimale.", en: "Per-GPU compilation for minimal latency." },
    usedBy: ["nvidia", "openai", "tesla", "bmw"],
  },
  {
    slug: "tflite", name: "TFLite", ext: ".tflite", color: "#FF6F00", creator: "google", year: 2017,
    hardware: ["Mobile", "Edge", "NPU"],
    use: { fr: "Android, IoT, microcontrôleurs", en: "Android, IoT, microcontrollers" },
    why: { fr: "Modèles légers, quantifiés, exécutables sur appareils contraints.", en: "Lightweight, quantized, deployable on constrained devices." },
    usedBy: ["google", "samsung", "xiaomi"],
  },
  {
    slug: "executorch", name: "ExecuTorch", ext: ".pte", color: "#EE4C2C", creator: "meta", year: 2024,
    hardware: ["Mobile", "Edge"],
    use: { fr: "PyTorch on-device (mobile, wearables, AR/VR)", en: "PyTorch on-device (mobile, wearables, AR/VR)" },
    why: { fr: "Permettre de déployer un modèle PyTorch sans serveur cloud.", en: "Deploy PyTorch models without cloud servers." },
    usedBy: ["meta", "qualcomm"],
  },
  {
    slug: "mlx", name: "MLX", ext: ".npz", color: "#A2AAAD", creator: "apple", year: 2023,
    hardware: ["Apple Silicon"],
    use: { fr: "Recherche & inférence sur Apple Silicon (M1–M4)", en: "Research & inference on Apple Silicon (M1–M4)" },
    why: { fr: "Framework natif Apple, mémoire unifiée, calculs lazy.", en: "Apple-native framework with unified memory, lazy compute." },
    usedBy: ["apple"],
  },
  {
    slug: "openvino", name: "OpenVINO", ext: ".xml/.bin", color: "#0071C5", creator: "intel", year: 2018,
    hardware: ["CPU", "GPU", "NPU"],
    use: { fr: "CPU / iGPU / NPU Intel optimisés", en: "Optimized Intel CPU / iGPU / NPU" },
    why: { fr: "Tirer parti du silicium Intel (AVX-512, AMX, Arc, NPU Meteor Lake).", en: "Exploit Intel silicon (AVX-512, AMX, Arc, Meteor Lake NPU)." },
    usedBy: ["intel", "bmw"],
  },
  {
    slug: "rknn", name: "RKNN", ext: ".rknn", color: "#E60012", creator: "rockchip", year: 2019,
    hardware: ["NPU", "Edge"],
    use: { fr: "NPU Rockchip (RK3588, robotique, caméras)", en: "Rockchip NPUs (RK3588, robotics, cameras)" },
    why: { fr: "Format propriétaire pour exploiter les NPU embarqués Rockchip.", en: "Proprietary format for Rockchip embedded NPUs." },
    usedBy: ["rockchip"],
  },
  {
    slug: "ncnn", name: "NCNN", ext: ".param/.bin", color: "#0052D9", creator: "tencent", year: 2017,
    hardware: ["Mobile", "Edge"],
    use: { fr: "Inférence mobile haute performance, sans dépendances", en: "High-performance mobile inference, zero deps" },
    why: { fr: "Optimisé pour le mobile, utilisé dans WeChat et QQ.", en: "Mobile-optimized, powers WeChat and QQ." },
    usedBy: ["tencent"],
  },
  {
    slug: "mnn", name: "MNN", ext: ".mnn", color: "#FF6A00", creator: "alibaba", year: 2019,
    hardware: ["Mobile", "Edge"],
    use: { fr: "Moteur Alibaba pour Taobao, Tmall, Alipay", en: "Alibaba engine for Taobao, Tmall, Alipay" },
    why: { fr: "Inférence ultra-rapide sur l'écosystème mobile chinois.", en: "Ultra-fast inference for the Chinese mobile ecosystem." },
    usedBy: ["alibaba"],
  },
  {
    slug: "paddle", name: "PaddlePaddle", ext: ".pdmodel", color: "#2932E1", creator: "baidu", year: 2016,
    hardware: ["CPU", "GPU"],
    use: { fr: "Stack ML chinoise, OCR, recommandation", en: "Chinese ML stack, OCR, recommendation" },
    why: { fr: "Alternative à TensorFlow/PyTorch portée par Baidu.", en: "Baidu-backed TensorFlow/PyTorch alternative." },
    usedBy: ["baidu"],
  },
  {
    slug: "awq", name: "AWQ", ext: ".safetensors", color: "#9B5DE5", creator: "mistral", year: 2023,
    hardware: ["GPU"],
    use: { fr: "Quantification 4-bit activation-aware", en: "Activation-aware 4-bit quantization" },
    why: { fr: "Préserve la précision en protégeant les poids saillants.", en: "Preserves precision by protecting salient weights." },
    usedBy: ["mistral", "huggingface"],
  },
  {
    slug: "gptq", name: "GPTQ", ext: ".safetensors", color: "#10A37F", creator: "huggingface", year: 2022,
    hardware: ["GPU"],
    use: { fr: "Quantification post-training 4-bit GPU", en: "Post-training 4-bit GPU quantization" },
    why: { fr: "Quantification rapide sans réentraînement.", en: "Fast quantization without retraining." },
    usedBy: ["huggingface", "openai"],
  },
  {
    slug: "mlir", name: "MLIR", ext: ".mlir", color: "#00B4D8", creator: "google", year: 2019,
    hardware: ["CPU", "GPU", "NPU"],
    use: { fr: "Représentation intermédiaire multi-niveau (compilateurs ML)", en: "Multi-level intermediate representation (ML compilers)" },
    why: { fr: "Unifier les compilateurs ML (XLA, IREE, TPU) sous une IR commune.", en: "Unify ML compilers (XLA, IREE, TPU) under a common IR." },
    usedBy: ["google", "nvidia", "intel"],
  },
  {
    slug: "tvm", name: "Apache TVM", ext: ".so/.json", color: "#4D75B8", creator: "apache", year: 2017,
    hardware: ["CPU", "GPU", "Edge", "NPU"],
    use: { fr: "Compilation cross-hardware pour edge & datacenter", en: "Cross-hardware compilation for edge & datacenter" },
    why: { fr: "Optimiser un modèle pour n'importe quel matériel via auto-tuning.", en: "Optimize a model for any hardware via auto-tuning." },
    usedBy: ["amazon", "huggingface"],
  },
  {
    slug: "qnn", name: "Qualcomm QNN", ext: ".dlc", color: "#3253DC", creator: "qualcomm", year: 2022,
    hardware: ["NPU", "Mobile"],
    use: { fr: "Snapdragon NPU (téléphones, XR, automobile)", en: "Snapdragon NPU (phones, XR, automotive)" },
    why: { fr: "Exploiter le Hexagon NPU pour l'IA générative on-device.", en: "Tap the Hexagon NPU for on-device generative AI." },
    usedBy: ["qualcomm", "samsung"],
  },
  {
    slug: "ggml", name: "GGML", ext: ".bin", color: "#5DAD4A", creator: "ggerganov", year: 2022,
    hardware: ["CPU"],
    use: { fr: "Ancien format llama.cpp, remplacé par GGUF", en: "Legacy llama.cpp format, replaced by GGUF" },
    why: { fr: "Premier format C++ pur pour LLM, base de toute la révolution local.", en: "First pure-C++ LLM format, foundation of the entire local-LLM movement." },
    usedBy: ["meta", "mistral"],
  },
  {
    slug: "jax", name: "JAX / Flax", ext: ".msgpack", color: "#5E97F6", creator: "google", year: 2018,
    hardware: ["GPU", "NPU"],
    use: { fr: "Recherche haute performance, TPU, parallélisme", en: "High-performance research, TPU, sharding" },
    why: { fr: "NumPy différentiable + compilation XLA, idéal pour les TPU.", en: "Differentiable NumPy + XLA compilation, ideal for TPUs." },
    usedBy: ["google", "deepmind"],
  },
  {
    slug: "fp8", name: "FP8 (E4M3/E5M2)", ext: ".safetensors", color: "#FF7E2D", creator: "nvidia", year: 2023,
    hardware: ["GPU"],
    use: { fr: "Entraînement & inférence H100/H200/B200", en: "Training & inference on H100/H200/B200" },
    why: { fr: "Diviser la mémoire par 2 sans perte mesurable avec Hopper/Blackwell.", en: "Half the memory with no measurable loss on Hopper/Blackwell." },
    usedBy: ["nvidia", "openai", "anthropic"],
  },
  {
    slug: "bf16", name: "BFloat16", ext: ".safetensors", color: "#9B5DE5", creator: "google", year: 2018,
    hardware: ["GPU", "NPU"],
    use: { fr: "Standard de fait pour l'entraînement moderne", en: "De-facto standard for modern training" },
    why: { fr: "Même range que FP32 avec la taille de FP16 — stabilité numérique.", en: "FP32 range with FP16 size — numerical stability." },
    usedBy: ["google", "nvidia", "meta", "openai"],
  },
  {
    slug: "mindspore", name: "MindSpore", ext: ".mindir", color: "#E60012", creator: "huawei", year: 2020,
    hardware: ["GPU", "NPU"],
    use: { fr: "Stack Huawei pour Ascend NPU", en: "Huawei stack for Ascend NPU" },
    why: { fr: "Indépendance de l'écosystème américain (CUDA/TPU).", en: "Independence from US ecosystem (CUDA/TPU)." },
    usedBy: ["huawei"],
  },
  {
    slug: "lora", name: "LoRA", ext: ".safetensors", color: "#FF4FD8", creator: "microsoft", year: 2021,
    hardware: ["CPU", "GPU"],
    use: { fr: "Adaptateurs légers (1–50 Mo) pour fine-tuning", en: "Lightweight adapters (1–50 MB) for fine-tuning" },
    why: { fr: "Spécialiser un LLM sans recopier ses 70 Go de poids.", en: "Specialize an LLM without copying 70 GB of weights." },
    usedBy: ["microsoft", "huggingface", "mistral"],
  },
  {
    slug: "vllm", name: "vLLM PagedAttention", ext: "n/a", color: "#38E1FF", creator: "berkeley", year: 2023,
    hardware: ["GPU"],
    use: { fr: "Serving haut-débit avec PagedAttention", en: "High-throughput serving with PagedAttention" },
    why: { fr: "Multiplier le débit par 24× via gestion mémoire type OS.", en: "24× throughput via OS-style memory management." },
    usedBy: ["anthropic", "deepseek", "mistral"],
  },
  {
    slug: "tensorflow", name: "TensorFlow SavedModel", ext: ".pb", color: "#FF6F00", creator: "google", year: 2015,
    hardware: ["CPU", "GPU", "NPU"],
    use: { fr: "Production Google historique (Search, YouTube)", en: "Historical Google production (Search, YouTube)" },
    why: { fr: "Premier framework graphe de production à grande échelle.", en: "First large-scale production graph framework." },
    usedBy: ["google", "spotify"],
  },
  {
    slug: "keras", name: "Keras H5", ext: ".h5", color: "#D00000", creator: "google", year: 2015,
    hardware: ["CPU", "GPU"],
    use: { fr: "API haut-niveau, education, prototypage", en: "High-level API, education, prototyping" },
    why: { fr: "Rendre l'apprentissage profond accessible aux ingénieurs.", en: "Make deep learning approachable for engineers." },
    usedBy: ["google"],
  },
  {
    slug: "tensorrt-llm", name: "TensorRT-LLM", ext: ".engine", color: "#76B900", creator: "nvidia", year: 2023,
    hardware: ["GPU"],
    use: { fr: "LLM optimisés H100/A100 avec in-flight batching", en: "LLM optimized for H100/A100 with in-flight batching" },
    why: { fr: "Latence minimale pour le serving LLM datacenter.", en: "Minimal latency for datacenter LLM serving." },
    usedBy: ["nvidia", "openai", "anthropic"],
  },
  {
    slug: "webnn", name: "WebNN", ext: "n/a", color: "#F26B3A", creator: "w3c", year: 2024,
    hardware: ["Web", "NPU"],
    use: { fr: "Inférence native dans le navigateur (NPU/GPU/CPU)", en: "Native browser inference (NPU/GPU/CPU)" },
    why: { fr: "Standard W3C pour exposer l'accélération matérielle au Web.", en: "W3C standard exposing hardware acceleration to the Web." },
    usedBy: ["microsoft", "intel"],
  },
  {
    slug: "wasm", name: "WebAssembly Tensors", ext: ".wasm", color: "#654FF0", creator: "huggingface", year: 2023,
    hardware: ["Web"],
    use: { fr: "Modèles dans le navigateur sans plugin", en: "Models in the browser without plugins" },
    why: { fr: "Exécution portable, sandboxée, multi-plateforme.", en: "Portable, sandboxed, cross-platform execution." },
    usedBy: ["huggingface", "microsoft"],
  },
];

/** Conversion compatibility matrix (source -> targets). */
export const COMPAT: Record<string, string[]> = {
  pytorch:     ["safetensors", "onnx", "gguf", "coreml", "tflite", "executorch", "mlx", "tensorrt", "openvino", "awq", "gptq"],
  safetensors: ["pytorch", "onnx", "gguf", "coreml", "tflite", "mlx", "tensorrt", "awq", "gptq"],
  onnx:        ["pytorch", "safetensors", "tensorrt", "coreml", "tflite", "openvino", "ncnn", "mnn", "rknn"],
  gguf:        ["safetensors", "onnx"],
  coreml:      ["onnx"],
  tflite:      ["onnx"],
  tensorrt:    [],
  executorch:  [],
  mlx:         ["safetensors"],
  openvino:    ["onnx"],
  rknn:        [],
  ncnn:        ["onnx"],
  mnn:         ["onnx"],
  paddle:      ["onnx"],
  awq:         ["gguf"],
  gptq:        ["gguf"],
};

/** Companies that rely on UMC-relevant formats every day. */
export const COMPANIES_USING_FORMATS: Array<{
  brand: BrandKey;
  formats: string[];
  blurb: { fr: string; en: string };
}> = [
  { brand: "meta",       formats: ["pytorch", "safetensors", "gguf", "executorch"], blurb: { fr: "Llama publié dans tous les formats utiles.", en: "Llama shipped in every useful format." } },
  { brand: "openai",     formats: ["pytorch", "tensorrt"],                          blurb: { fr: "Entraînement PyTorch, inférence TensorRT.", en: "PyTorch training, TensorRT inference." } },
  { brand: "google",     formats: ["tflite", "onnx"],                               blurb: { fr: "Gemma déployé sur Android et Pixel via TFLite.", en: "Gemma shipped to Android & Pixel via TFLite." } },
  { brand: "microsoft",  formats: ["onnx", "gguf"],                                 blurb: { fr: "Phi-3 publié en ONNX et GGUF dès le jour 1.", en: "Phi-3 shipped in ONNX and GGUF day one." } },
  { brand: "mistral",    formats: ["safetensors", "gguf", "awq"],                   blurb: { fr: "Modèles ouverts en SafeTensors + GGUF.", en: "Open weights as SafeTensors + GGUF." } },
  { brand: "apple",      formats: ["coreml", "mlx"],                                blurb: { fr: "Apple Intelligence tourne sur CoreML + MLX.", en: "Apple Intelligence runs on CoreML + MLX." } },
  { brand: "nvidia",     formats: ["tensorrt", "onnx"],                             blurb: { fr: "TensorRT-LLM optimise tous les grands LLM publics.", en: "TensorRT-LLM optimizes every major public LLM." } },
  { brand: "tesla",      formats: ["pytorch", "tensorrt"],                          blurb: { fr: "FSD entraîné en PyTorch, déployé sur HW4.", en: "FSD trained in PyTorch, deployed on HW4." } },
  { brand: "stability",  formats: ["safetensors", "onnx"],                          blurb: { fr: "Stable Diffusion en SafeTensors par défaut.", en: "Stable Diffusion ships as SafeTensors by default." } },
  { brand: "huggingface",formats: ["safetensors", "gguf", "gptq", "awq"],           blurb: { fr: "Hub : 1M+ modèles dans des formats convertibles.", en: "Hub: 1M+ models in convertible formats." } },
  { brand: "alibaba",    formats: ["mnn", "gguf", "safetensors"],                   blurb: { fr: "Qwen sert tout l'écosystème Alibaba.", en: "Qwen powers the Alibaba ecosystem." } },
  { brand: "tencent",    formats: ["ncnn", "onnx"],                                 blurb: { fr: "WeChat embarque NCNN sur des centaines de millions de téléphones.", en: "WeChat ships NCNN to hundreds of millions of phones." } },
  { brand: "intel",      formats: ["openvino", "onnx"],                             blurb: { fr: "OpenVINO accélère CPU, iGPU et NPU Meteor Lake.", en: "OpenVINO accelerates CPU, iGPU and Meteor Lake NPU." } },
  { brand: "qualcomm",   formats: ["executorch", "onnx"],                           blurb: { fr: "Snapdragon NPU via ExecuTorch + QNN.", en: "Snapdragon NPU via ExecuTorch + QNN." } },
  { brand: "samsung",    formats: ["tflite", "onnx"],                               blurb: { fr: "Galaxy AI s'appuie sur TFLite + NPU Exynos.", en: "Galaxy AI relies on TFLite + Exynos NPU." } },
  { brand: "bmw",        formats: ["tensorrt", "openvino"],                         blurb: { fr: "Vision in-cabin sur GPU NVIDIA + iGPU Intel.", en: "In-cabin vision on NVIDIA GPU + Intel iGPU." } },
  { brand: "spotify",    formats: ["onnx", "pytorch"],                              blurb: { fr: "Modèles de recommandation portés en ONNX.", en: "Recommendation models ported to ONNX." } },
  { brand: "shopify",    formats: ["onnx"],                                         blurb: { fr: "Search & ranking servis via ONNX Runtime.", en: "Search & ranking served via ONNX Runtime." } },
  { brand: "airbus",     formats: ["onnx", "tensorrt"],                             blurb: { fr: "Vision satellite et maintenance prédictive.", en: "Satellite vision and predictive maintenance." } },
  { brand: "snapchat",   formats: ["coreml", "tflite"],                             blurb: { fr: "Lens AR exécutent des modèles on-device.", en: "AR lenses run on-device models." } },
  { brand: "deepseek",   formats: ["safetensors", "gguf"],                          blurb: { fr: "DeepSeek-V3 publié ouvertement, converti en heures.", en: "DeepSeek-V3 shipped openly, converted in hours." } },
  { brand: "anthropic",  formats: ["pytorch"],                                      blurb: { fr: "Stack d'entraînement custom basée PyTorch.", en: "Custom training stack built on PyTorch." } },
];

/** Detailed daily usage of UMC by industry verticals. */
export type CompanyDeepDive = {
  brand: BrandKey;
  sector: { fr: string; en: string };
  formats: string[];
  daily: { fr: string[]; en: string[] };
  flow: { fr: string; en: string };
};

export const COMPANY_DEEP_DIVES: CompanyDeepDive[] = [
  {
    brand: "meta", sector: { fr: "Réseaux sociaux & R&D", en: "Social networks & R&D" },
    formats: ["pytorch", "safetensors", "gguf", "executorch"],
    daily: {
      fr: [
        "Entraîne Llama en PyTorch sur 24 000 GPU H100.",
        "Publie les poids en SafeTensors pour la communauté.",
        "Convertit en GGUF pour l'inférence locale (llama.cpp, Ollama).",
        "Déploie sur Quest et Ray-Ban via ExecuTorch (PyTorch on-device).",
      ],
      en: [
        "Trains Llama in PyTorch on 24,000 H100 GPUs.",
        "Publishes weights as SafeTensors for the community.",
        "Converts to GGUF for local inference (llama.cpp, Ollama).",
        "Ships to Quest and Ray-Ban via ExecuTorch (PyTorch on-device).",
      ],
    },
    flow: { fr: "PyTorch → SafeTensors → GGUF / ExecuTorch", en: "PyTorch → SafeTensors → GGUF / ExecuTorch" },
  },
  {
    brand: "apple", sector: { fr: "Hardware grand public", en: "Consumer hardware" },
    formats: ["coreml", "mlx"],
    daily: {
      fr: [
        "Recherche en MLX sur Mac M-series (mémoire unifiée).",
        "Convertit vers CoreML pour exploiter le Neural Engine.",
        "Embarque Apple Intelligence sur iPhone, iPad, Mac sans aller dans le cloud.",
        "Garantie de confidentialité native par exécution on-device.",
      ],
      en: [
        "Research in MLX on Apple Silicon (unified memory).",
        "Converts to CoreML to use the Neural Engine.",
        "Ships Apple Intelligence on iPhone, iPad, Mac without cloud.",
        "Native privacy through on-device execution.",
      ],
    },
    flow: { fr: "MLX → CoreML → Neural Engine", en: "MLX → CoreML → Neural Engine" },
  },
  {
    brand: "tesla", sector: { fr: "Automobile autonome", en: "Autonomous automotive" },
    formats: ["pytorch", "tensorrt"],
    daily: {
      fr: [
        "Entraîne FSD en PyTorch sur le supercalculateur Dojo.",
        "Convertit en TensorRT pour le matériel HW4 embarqué dans chaque véhicule.",
        "Latence < 50 ms à 60 km/h, ISO 26262.",
        "OTA hebdomadaires : chaque release re-convertie et certifiée.",
      ],
      en: [
        "Trains FSD in PyTorch on the Dojo supercomputer.",
        "Converts to TensorRT for the in-vehicle HW4.",
        "< 50 ms latency at 60 km/h, ISO 26262.",
        "Weekly OTAs: every release re-converted and certified.",
      ],
    },
    flow: { fr: "PyTorch → ONNX → TensorRT", en: "PyTorch → ONNX → TensorRT" },
  },
  {
    brand: "spotify", sector: { fr: "Recommandation musicale", en: "Music recommendation" },
    formats: ["onnx", "pytorch"],
    daily: {
      fr: [
        "Entraînement des modèles de recommandation en PyTorch.",
        "Export ONNX pour servir sur cluster CPU multi-cloud (AWS + GCP).",
        "Inference batch quotidienne pour Discover Weekly (574 M d'utilisateurs).",
        "Métriques A/B sur chaque nouvelle conversion.",
      ],
      en: [
        "Trains recommendation models in PyTorch.",
        "Exports ONNX to serve on CPU clusters across AWS and GCP.",
        "Daily batch inference powers Discover Weekly (574 M users).",
        "A/B metrics on every new conversion.",
      ],
    },
    flow: { fr: "PyTorch → ONNX → ONNX Runtime", en: "PyTorch → ONNX → ONNX Runtime" },
  },
  {
    brand: "bmw", sector: { fr: "Vision automobile", en: "Automotive vision" },
    formats: ["tensorrt", "openvino"],
    daily: {
      fr: [
        "Vision in-cabin (regard, fatigue) sur GPU NVIDIA Orin.",
        "Passerelles caméra optimisées Intel OpenVINO.",
        "Pipeline UMC : SafeTensors → ONNX → TensorRT (cabine) + OpenVINO (gateway).",
        "Certification fonctionnelle ISO 26262 sur la chaîne de conversion.",
      ],
      en: [
        "In-cabin vision (gaze, fatigue) on NVIDIA Orin GPUs.",
        "Camera gateways optimized with Intel OpenVINO.",
        "UMC pipeline: SafeTensors → ONNX → TensorRT (cabin) + OpenVINO (gateway).",
        "ISO 26262 functional certification of the conversion chain.",
      ],
    },
    flow: { fr: "SafeTensors → ONNX → TensorRT / OpenVINO", en: "SafeTensors → ONNX → TensorRT / OpenVINO" },
  },
  {
    brand: "huggingface", sector: { fr: "Hub & infrastructure ML", en: "ML hub & infrastructure" },
    formats: ["safetensors", "gguf", "gptq", "awq"],
    daily: {
      fr: [
        "Plus d'1 million de modèles hébergés en SafeTensors.",
        "Conversion automatique vers GGUF / GPTQ / AWQ à l'upload.",
        "Inference Endpoints servis en ONNX Runtime et vLLM.",
        "Partenariat avec UMC pour la certification cryptographique des poids.",
      ],
      en: [
        "1M+ models hosted as SafeTensors.",
        "Automatic conversion to GGUF / GPTQ / AWQ on upload.",
        "Inference Endpoints served via ONNX Runtime and vLLM.",
        "Partnered with UMC for cryptographic weight certification.",
      ],
    },
    flow: { fr: "SafeTensors → GGUF / GPTQ / AWQ / ONNX", en: "SafeTensors → GGUF / GPTQ / AWQ / ONNX" },
  },
  {
    brand: "samsung", sector: { fr: "Smartphones & électroménager", en: "Smartphones & appliances" },
    formats: ["tflite", "onnx"],
    daily: {
      fr: [
        "Galaxy AI sur 200 M de téléphones via TFLite + NPU Exynos.",
        "Live Translate, Note Assist, Generative Edit en local.",
        "Conversion ONNX → TFLite avec calibration INT8 quotidienne.",
        "Tests A/B in-device sur des cohortes pilotes.",
      ],
      en: [
        "Galaxy AI on 200 M phones via TFLite + Exynos NPU.",
        "Live Translate, Note Assist, Generative Edit on device.",
        "Daily ONNX → TFLite conversion with INT8 calibration.",
        "On-device A/B tests on pilot cohorts.",
      ],
    },
    flow: { fr: "PyTorch → ONNX → TFLite", en: "PyTorch → ONNX → TFLite" },
  },
  {
    brand: "snapchat", sector: { fr: "AR & créateurs", en: "AR & creators" },
    formats: ["coreml", "tflite"],
    daily: {
      fr: [
        "300 000 Lens AR utilisent des modèles de vision on-device.",
        "Conversion vers CoreML (iOS) et TFLite (Android) en parallèle.",
        "Latence garantie < 16 ms (60 FPS) sur tous les téléphones cibles.",
        "Pipeline automatique : un seul modèle source → deux binaires certifiés.",
      ],
      en: [
        "300,000 AR Lenses run on-device vision models.",
        "Parallel conversion to CoreML (iOS) and TFLite (Android).",
        "Latency guaranteed < 16 ms (60 FPS) on every target phone.",
        "Automated pipeline: one source model → two certified binaries.",
      ],
    },
    flow: { fr: "PyTorch → CoreML + TFLite", en: "PyTorch → CoreML + TFLite" },
  },
  {
    brand: "openai", sector: { fr: "Recherche & API frontière", en: "Frontier research & API" },
    formats: ["pytorch", "tensorrt"],
    daily: {
      fr: [
        "Entraîne GPT-class en PyTorch + FP8 mixed-precision.",
        "Sert en TensorRT-LLM sur clusters H100 multi-tenant.",
        "Compilation par GPU (Hopper, Blackwell) re-vérifiée à chaque release.",
        "Latence p50 < 80 ms à très haut débit.",
      ],
      en: [
        "Trains GPT-class models in PyTorch + FP8 mixed-precision.",
        "Serves via TensorRT-LLM on multi-tenant H100 clusters.",
        "Per-GPU compilation (Hopper, Blackwell) re-verified each release.",
        "p50 latency < 80 ms at very high throughput.",
      ],
    },
    flow: { fr: "PyTorch → ONNX → TensorRT-LLM (FP8)", en: "PyTorch → ONNX → TensorRT-LLM (FP8)" },
  },
  {
    brand: "mistral", sector: { fr: "LLM ouverts européens", en: "European open LLMs" },
    formats: ["safetensors", "gguf", "awq"],
    daily: {
      fr: [
        "Mistral, Mixtral, Codestral publiés en SafeTensors.",
        "Variantes GGUF dès le jour 1 pour llama.cpp.",
        "AWQ INT4 pour le serving GPU haut-débit (vLLM, TGI).",
        "Souveraineté européenne sur la chaîne de conversion.",
      ],
      en: [
        "Mistral, Mixtral, Codestral published as SafeTensors.",
        "GGUF variants day-one for llama.cpp.",
        "AWQ INT4 for high-throughput GPU serving (vLLM, TGI).",
        "European sovereignty over the conversion chain.",
      ],
    },
    flow: { fr: "SafeTensors → GGUF / AWQ", en: "SafeTensors → GGUF / AWQ" },
  },
  {
    brand: "airbus", sector: { fr: "Aéronautique & spatial", en: "Aerospace & space" },
    formats: ["onnx", "tensorrt"],
    daily: {
      fr: [
        "Analyse d'images satellites à grande échelle (Pléiades, OneAtlas).",
        "Détection d'anomalies de maintenance sur la flotte A320/A350.",
        "Conversion ONNX → TensorRT pour l'inférence GPU dans les datacenters Toulouse.",
        "Certification de traçabilité conforme aux exigences EASA.",
      ],
      en: [
        "Large-scale satellite image analytics (Pléiades, OneAtlas).",
        "Maintenance anomaly detection on A320/A350 fleets.",
        "ONNX → TensorRT conversion for GPU inference in Toulouse datacenters.",
        "Traceability certification compliant with EASA requirements.",
      ],
    },
    flow: { fr: "PyTorch → ONNX → TensorRT", en: "PyTorch → ONNX → TensorRT" },
  },
  {
    brand: "tencent", sector: { fr: "Messagerie & jeux", en: "Messaging & gaming" },
    formats: ["ncnn", "onnx"],
    daily: {
      fr: [
        "WeChat embarque NCNN sur 1,3 milliard de téléphones.",
        "Modèles de modération, OCR, voix tous on-device.",
        "Conversion ONNX → NCNN avec quantification INT8 spécifique mobile.",
        "Empreinte mémoire < 50 Mo par modèle.",
      ],
      en: [
        "WeChat ships NCNN to 1.3 B phones.",
        "Moderation, OCR, voice models all on-device.",
        "ONNX → NCNN conversion with mobile-specific INT8 quantization.",
        "Memory footprint < 50 MB per model.",
      ],
    },
    flow: { fr: "PyTorch → ONNX → NCNN", en: "PyTorch → ONNX → NCNN" },
  },
];