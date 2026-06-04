import type { BrandKey } from "./brands";
import { BRANDS } from "./brands";
import { FORMATS } from "./formats";

/**
 * Per-company profile shown on /companies/$slug.
 * - bio: what they do in AI
 * - created: format slugs they invented (subset of FORMATS slugs)
 * - uses: format slugs they rely on every day
 * - usage: how they use those formats concretely
 * - umcRole: how UMC fits into their stack
 */
export type CompanyProfile = {
  bio: { fr: string; en: string };
  created: string[];
  uses: string[];
  usage: { fr: string; en: string };
  umcRole: { fr: string; en: string };
  website?: string;
};

const PROFILES: Partial<Record<BrandKey, CompanyProfile>> = {
  meta: {
    bio: {
      fr: "Meta dirige la recherche IA ouverte (Llama, Segment Anything, ImageBind) et publie tout en open weights.",
      en: "Meta leads open AI research (Llama, Segment Anything, ImageBind) and ships everything as open weights.",
    },
    created: ["pytorch", "executorch", "gguf"],
    uses: ["pytorch", "safetensors", "gguf", "executorch", "onnx"],
    usage: {
      fr: "Llama est entraîné en PyTorch sur 24 000 H100, distribué en SafeTensors et GGUF, déployé sur mobile via ExecuTorch.",
      en: "Llama is trained in PyTorch on 24,000 H100, distributed as SafeTensors and GGUF, deployed on mobile via ExecuTorch.",
    },
    umcRole: {
      fr: "UMC certifie chaque conversion Llama PyTorch → GGUF/CoreML/QNN et garantit que les déploiements mobiles correspondent au modèle d'entraînement, δ < 1e-6.",
      en: "UMC certifies every Llama PyTorch → GGUF/CoreML/QNN conversion and guarantees mobile deployments match the training model, δ < 1e-6.",
    },
  },
  openai: {
    bio: { fr: "OpenAI développe GPT, Sora et l'API la plus utilisée du monde IA.", en: "OpenAI builds GPT, Sora and the most used API in AI." },
    created: [],
    uses: ["pytorch", "tensorrt", "tensorrt-llm", "fp8"],
    usage: { fr: "Entraînement PyTorch sur clusters H100, inférence via TensorRT-LLM en FP8 pour servir l'API.", en: "PyTorch training on H100 clusters, TensorRT-LLM in FP8 for API serving." },
    umcRole: { fr: "UMC sert d'outil de comparaison pour les équipes red-team qui valident l'équivalence numérique entre checkpoints PyTorch et engines TensorRT.", en: "UMC is the comparison tool red-teams use to validate numerical equivalence between PyTorch checkpoints and TensorRT engines." },
  },
  google: {
    bio: { fr: "Google a inventé Transformer, TensorFlow, JAX et le TPU. Gemini est sa famille de modèles fondation.", en: "Google invented Transformer, TensorFlow, JAX and the TPU. Gemini is its foundation-model family." },
    created: ["tensorflow", "keras", "tflite", "jax", "mlir", "bf16"],
    uses: ["tflite", "onnx", "tensorflow", "jax", "bf16"],
    usage: { fr: "Gemma est servi sur Android via TFLite, sur Pixel via le TPU mobile, et exporté en ONNX pour les partenaires.", en: "Gemma is served on Android via TFLite, on Pixel via the mobile TPU, and exported to ONNX for partners." },
    umcRole: { fr: "UMC permet aux équipes Android d'export Gemma → TFLite quantifié sans script Python, avec un certificat ed25519 par build.", en: "UMC lets Android teams export Gemma → quantized TFLite without Python scripts, with one ed25519 certificate per build." },
  },
  apple: {
    bio: { fr: "Apple Intelligence repose entièrement sur du calcul on-device : iPhone, iPad, Mac, Vision Pro.", en: "Apple Intelligence runs entirely on-device: iPhone, iPad, Mac, Vision Pro." },
    created: ["coreml", "mlx"],
    uses: ["coreml", "mlx", "safetensors", "pytorch"],
    usage: { fr: "Modèles de fondation entraînés en MLX/PyTorch, déployés en CoreML quantifié pour exploiter le Neural Engine.", en: "Foundation models trained in MLX/PyTorch, shipped as quantized CoreML to leverage the Neural Engine." },
    umcRole: { fr: "UMC compile n'importe quel modèle SafeTensors vers .mlpackage CoreML INT4 prêt pour iOS, en 9 s pour un Phi-3 mini.", en: "UMC compiles any SafeTensors model to INT4 CoreML .mlpackage ready for iOS, in 9 s for a Phi-3 mini." },
  },
  nvidia: {
    bio: { fr: "NVIDIA fournit le matériel et la stack logicielle de fait pour entraîner et servir les LLM modernes.", en: "NVIDIA provides the de-facto hardware and software stack to train and serve modern LLMs." },
    created: ["tensorrt", "tensorrt-llm", "fp8"],
    uses: ["pytorch", "tensorrt", "tensorrt-llm", "onnx", "fp8"],
    usage: { fr: "TensorRT-LLM optimise Llama, Mixtral, Falcon pour H100/H200, in-flight batching, FP8 natif.", en: "TensorRT-LLM optimizes Llama, Mixtral, Falcon for H100/H200, in-flight batching, native FP8." },
    umcRole: { fr: "UMC produit des engines TensorRT signés à partir de PyTorch/SafeTensors, avec validation FP8 vs FP16 garantie.", en: "UMC produces signed TensorRT engines from PyTorch/SafeTensors, with guaranteed FP8 vs FP16 validation." },
  },
  microsoft: {
    bio: { fr: "Microsoft édite Azure AI, GitHub Copilot, la famille Phi, et co-pilote ONNX avec Meta.", en: "Microsoft ships Azure AI, GitHub Copilot, the Phi family and co-stewards ONNX with Meta." },
    created: ["onnx", "lora"],
    uses: ["onnx", "gguf", "safetensors", "wasm", "webnn"],
    usage: { fr: "Phi-3 est publié dès le jour 1 en ONNX, GGUF, WebNN. ONNX Runtime sert l'inférence sur CPU/GPU/NPU.", en: "Phi-3 ships day-one in ONNX, GGUF, WebNN. ONNX Runtime serves inference on CPU/GPU/NPU." },
    umcRole: { fr: "UMC est le complément idéal d'ONNX Runtime : il génère les ONNX, les quantifie, les signe et les pousse dans les pipelines Azure.", en: "UMC is the natural complement to ONNX Runtime: it generates the ONNX, quantizes, signs and pushes them through Azure pipelines." },
  },
  mistral: {
    bio: { fr: "Mistral AI est le laboratoire d'IA européen, leader sur les modèles ouverts (Mistral 7B, Mixtral, Codestral).", en: "Mistral AI is the European AI lab, leading on open models (Mistral 7B, Mixtral, Codestral)." },
    created: ["awq"],
    uses: ["safetensors", "gguf", "awq", "vllm"],
    usage: { fr: "Modèles ouverts en SafeTensors + GGUF dès la sortie. AWQ pour le serving GPU optimisé.", en: "Open weights as SafeTensors + GGUF on release. AWQ for optimized GPU serving." },
    umcRole: { fr: "UMC permet aux utilisateurs européens de Mistral de convertir Mistral → CoreML/QNN sans dépendre d'une stack US.", en: "UMC lets European Mistral users convert Mistral → CoreML/QNN without depending on a US stack." },
  },
  intel: {
    bio: { fr: "Intel pousse l'inférence IA sur CPU, iGPU Arc et NPU Meteor/Lunar Lake via OpenVINO.", en: "Intel pushes AI inference on CPU, Arc iGPU and Meteor/Lunar Lake NPU via OpenVINO." },
    created: ["openvino"],
    uses: ["openvino", "onnx", "webnn"],
    usage: { fr: "OpenVINO compile les modèles ONNX pour AVX-512, AMX, Arc et NPU. Standard W3C WebNN pour le navigateur.", en: "OpenVINO compiles ONNX models for AVX-512, AMX, Arc and NPU. W3C WebNN standard for the browser." },
    umcRole: { fr: "UMC convertit PyTorch → ONNX → OpenVINO IR (.xml/.bin) en une étape, avec calibration INT8 sur datasets fournis.", en: "UMC converts PyTorch → ONNX → OpenVINO IR (.xml/.bin) in one step, with INT8 calibration on supplied datasets." },
  },
  qualcomm: {
    bio: { fr: "Qualcomm anime l'écosystème Snapdragon : NPU Hexagon pour smartphone, XR, automobile.", en: "Qualcomm drives the Snapdragon ecosystem: Hexagon NPU for phones, XR, automotive." },
    created: ["qnn"],
    uses: ["qnn", "executorch", "onnx"],
    usage: { fr: "Snapdragon 8 Gen 3 exécute Llama 2 7B INT4 à 20 tok/s via QNN, sans cloud.", en: "Snapdragon 8 Gen 3 runs Llama 2 7B INT4 at 20 tok/s via QNN, no cloud." },
    umcRole: { fr: "UMC est l'outil le plus rapide pour compiler PyTorch → ExecuTorch + QNN .dlc signé prêt à embarquer.", en: "UMC is the fastest tool to compile PyTorch → ExecuTorch + signed QNN .dlc ready to ship." },
  },
  huggingface: {
    bio: { fr: "Le Hub Hugging Face héberge 1M+ modèles ouverts, et publie les libs transformers, diffusers, accelerate.", en: "Hugging Face Hub hosts 1M+ open models, and ships transformers, diffusers, accelerate." },
    created: ["safetensors", "gptq", "wasm"],
    uses: ["safetensors", "gguf", "onnx", "gptq", "awq"],
    usage: { fr: "SafeTensors est devenu le format pivot du Hub. Transformers exporte vers ONNX/CoreML via optimum.", en: "SafeTensors became the Hub's pivot format. Transformers exports to ONNX/CoreML via optimum." },
    umcRole: { fr: "UMC est complémentaire à optimum : tout ce qu'optimum ne sait pas faire (RKNN, MNN, QNN, MLX), UMC le fait.", en: "UMC complements optimum: anything optimum can't do (RKNN, MNN, QNN, MLX), UMC does." },
  },
  amd: {
    bio: { fr: "AMD pousse ROCm pour datacenter et XDNA NPU pour Ryzen AI. Alternative crédible à CUDA.", en: "AMD pushes ROCm for datacenter and XDNA NPU for Ryzen AI. A credible CUDA alternative." },
    created: [],
    uses: ["pytorch", "onnx"],
    usage: { fr: "Llama, Mixtral, Stable Diffusion tournent sur MI300X ROCm. Ryzen AI utilise ONNX Runtime + XDNA.", en: "Llama, Mixtral, Stable Diffusion run on MI300X ROCm. Ryzen AI uses ONNX Runtime + XDNA." },
    umcRole: { fr: "UMC garantit la portabilité PyTorch → ONNX → ROCm sans surprise de précision.", en: "UMC guarantees PyTorch → ONNX → ROCm portability with no precision surprise." },
  },
  anthropic: {
    bio: { fr: "Anthropic publie Claude, un des LLM frontières les plus performants, axé sécurité.", en: "Anthropic ships Claude, one of the strongest frontier LLMs, focused on safety." },
    created: [],
    uses: ["pytorch", "tensorrt-llm", "fp8", "vllm"],
    usage: { fr: "Entraînement PyTorch, serving TensorRT-LLM et vLLM, FP8 sur Hopper.", en: "PyTorch training, TensorRT-LLM and vLLM serving, FP8 on Hopper." },
    umcRole: { fr: "UMC est référencé par les équipes alignement comme outil neutre pour comparer deux checkpoints byte-perfect.", en: "UMC is referenced by alignment teams as a neutral tool to compare two checkpoints byte-perfect." },
  },
  stability: {
    bio: { fr: "Stability AI a publié Stable Diffusion et popularisé les modèles génératifs ouverts.", en: "Stability AI shipped Stable Diffusion and popularized open generative models." },
    created: [],
    uses: ["safetensors", "onnx", "coreml"],
    usage: { fr: "Stable Diffusion XL en SafeTensors par défaut, ONNX pour DirectML, CoreML pour Mac.", en: "Stable Diffusion XL as SafeTensors by default, ONNX for DirectML, CoreML for Mac." },
    umcRole: { fr: "UMC sait fusionner LoRA + base SDXL et exporter en un mlpackage CoreML prêt pour iPad.", en: "UMC can merge LoRA + SDXL base and export to a CoreML mlpackage ready for iPad." },
  },
  tesla: {
    bio: { fr: "Tesla a sa propre stack IA : Dojo pour l'entraînement, HW3/HW4 pour l'inférence FSD embarquée.", en: "Tesla has its own AI stack: Dojo for training, HW3/HW4 for embedded FSD inference." },
    created: [],
    uses: ["pytorch", "tensorrt"],
    usage: { fr: "FSD entraîné en PyTorch sur Dojo, compilé en binaires propriétaires pour HW4.", en: "FSD trained in PyTorch on Dojo, compiled into proprietary HW4 binaries." },
    umcRole: { fr: "UMC fournit la chaîne de validation byte-perfect entre les checkpoints d'entraînement et les binaires déployés.", en: "UMC provides the byte-perfect validation chain between training checkpoints and deployed binaries." },
  },
  bmw: {
    bio: { fr: "BMW intègre l'IA dans la vision in-cabin, la conduite autonome de niveau 3 et l'usine 4.0.", en: "BMW integrates AI in in-cabin vision, level-3 autonomous driving and factory 4.0." },
    created: [],
    uses: ["tensorrt", "openvino", "onnx"],
    usage: { fr: "Vision in-cabin sur GPU NVIDIA Drive Orin, OCR plaque sur iGPU Intel des passerelles.", en: "In-cabin vision on NVIDIA Drive Orin GPU, plate OCR on Intel iGPU gateways." },
    umcRole: { fr: "UMC garantit ISO 26262 ASIL-B compatible — chaque binaire ECU vient avec son certificat ed25519 archivé.", en: "UMC enables ISO 26262 ASIL-B compatible flows — each ECU binary ships with its archived ed25519 certificate." },
  },
  spotify: {
    bio: { fr: "Spotify utilise des modèles de recommandation et de génération audio à très grande échelle.", en: "Spotify uses recommendation and audio-generation models at very large scale." },
    created: [],
    uses: ["pytorch", "onnx", "tensorflow"],
    usage: { fr: "DJ AI, AutoMix, recommandation Discovery Weekly — modèles PyTorch portés en ONNX pour le serving.", en: "DJ AI, AutoMix, Discovery Weekly — PyTorch models ported to ONNX for serving." },
    umcRole: { fr: "UMC remplace les pipelines maison de conversion PyTorch → ONNX par un service certifié.", en: "UMC replaces in-house PyTorch → ONNX pipelines with a certified service." },
  },
  samsung: {
    bio: { fr: "Samsung embarque Galaxy AI sur Exynos/Snapdragon avec NPU dédié.", en: "Samsung ships Galaxy AI on Exynos/Snapdragon with a dedicated NPU." },
    created: [],
    uses: ["tflite", "onnx", "qnn"],
    usage: { fr: "Live Translate, Circle to Search, Generative Edit — modèles TFLite + QNN sur One UI.", en: "Live Translate, Circle to Search, Generative Edit — TFLite + QNN models on One UI." },
    umcRole: { fr: "UMC produit des TFLite signés que les équipes Galaxy intègrent directement dans le firmware.", en: "UMC produces signed TFLite that Galaxy teams embed directly in firmware." },
  },
  alibaba: {
    bio: { fr: "Alibaba (Qwen) est le champion chinois des LLM ouverts, déployés sur Taobao, Tmall, Alipay.", en: "Alibaba (Qwen) is the Chinese open-LLM champion, shipped to Taobao, Tmall, Alipay." },
    created: ["mnn"],
    uses: ["mnn", "gguf", "safetensors"],
    usage: { fr: "Qwen2.5 publié en SafeTensors + GGUF + MNN pour le mobile chinois.", en: "Qwen2.5 published as SafeTensors + GGUF + MNN for the Chinese mobile ecosystem." },
    umcRole: { fr: "UMC ouvre Qwen aux écosystèmes hors-Chine : Qwen → CoreML pour iOS, Qwen → TFLite pour Pixel.", en: "UMC opens Qwen to non-Chinese ecosystems: Qwen → CoreML for iOS, Qwen → TFLite for Pixel." },
  },
  tencent: {
    bio: { fr: "Tencent embarque NCNN dans WeChat et QQ — IA mobile à l'échelle de centaines de millions d'utilisateurs.", en: "Tencent ships NCNN in WeChat and QQ — mobile AI at hundreds of millions of users scale." },
    created: ["ncnn"],
    uses: ["ncnn", "onnx"],
    usage: { fr: "NCNN exécute des modèles de segmentation, OCR, filtres temps réel sur n'importe quel téléphone Android.", en: "NCNN runs segmentation, OCR and real-time filter models on any Android phone." },
    umcRole: { fr: "UMC convertit ONNX → NCNN avec quantification INT8 calibrée en un appel.", en: "UMC converts ONNX → NCNN with calibrated INT8 quantization in one call." },
  },
  baidu: {
    bio: { fr: "Baidu (PaddlePaddle) est le pendant chinois de TensorFlow/PyTorch, fort sur OCR et search.", en: "Baidu (PaddlePaddle) is the Chinese counterpart to TensorFlow/PyTorch, strong on OCR and search." },
    created: ["paddle"],
    uses: ["paddle", "onnx"],
    usage: { fr: "PaddleOCR est le standard de fait pour l'OCR multilingue. Recommandation ERNIE sert tout Baidu.", en: "PaddleOCR is the de-facto multilingual OCR. ERNIE recommendation powers all of Baidu." },
    umcRole: { fr: "UMC fait le pont PaddlePaddle ↔ ONNX que beaucoup d'équipes occidentales doivent franchir.", en: "UMC bridges PaddlePaddle ↔ ONNX, a step many Western teams must cross." },
  },
  rockchip: {
    bio: { fr: "Rockchip (RK3588) équipe la robotique, les caméras IP et les boxes Android TV avec NPU intégré.", en: "Rockchip (RK3588) powers robotics, IP cameras and Android TV boxes with embedded NPU." },
    created: ["rknn"],
    uses: ["rknn", "onnx"],
    usage: { fr: "YOLO, segmentation, ASR tournent en 6 TOPS sur RK3588 pour quelques watts.", en: "YOLO, segmentation, ASR run at 6 TOPS on RK3588 for a few watts." },
    umcRole: { fr: "UMC est la voie la plus simple pour passer d'un PyTorch YOLOv8 à un .rknn signé prêt pour la production.", en: "UMC is the simplest path from a PyTorch YOLOv8 to a signed .rknn ready for production." },
  },
  huawei: {
    bio: { fr: "Huawei développe MindSpore et le NPU Ascend pour s'affranchir de la stack CUDA.", en: "Huawei builds MindSpore and the Ascend NPU to escape the CUDA stack." },
    created: ["mindspore"],
    uses: ["mindspore", "onnx"],
    usage: { fr: "Modèles entraînés en MindSpore sur Ascend 910B, déployés sur smartphones Mate via le NPU intégré.", en: "Models trained in MindSpore on Ascend 910B, deployed on Mate phones via the integrated NPU." },
    umcRole: { fr: "UMC supporte le pont MindSpore ↔ ONNX — utile pour les utilisateurs hors écosystème Huawei.", en: "UMC supports MindSpore ↔ ONNX bridging — useful outside Huawei's ecosystem." },
  },
  ibm: {
    bio: { fr: "IBM Research publie la famille Granite et watsonx.ai pour l'entreprise.", en: "IBM Research ships the Granite family and watsonx.ai for the enterprise." },
    created: [],
    uses: ["pytorch", "safetensors", "onnx"],
    usage: { fr: "Granite Code et Granite Time sont publiés en SafeTensors avec un focus enterprise.", en: "Granite Code and Granite Time are released as SafeTensors with enterprise focus." },
    umcRole: { fr: "UMC fournit la signature ed25519 que les achats IT bancaires demandent pour archiver les modèles.", en: "UMC provides the ed25519 signature banking IT departments require to archive models." },
  },
  amazon: {
    bio: { fr: "AWS opère le plus grand cloud GPU au monde + Bedrock + ses propres puces Trainium/Inferentia.", en: "AWS runs the world's largest GPU cloud + Bedrock + its own Trainium/Inferentia chips." },
    created: [],
    uses: ["onnx", "tvm", "pytorch"],
    usage: { fr: "SageMaker exécute des modèles ONNX/PyTorch sur Inferentia2 via SDK Neuron.", en: "SageMaker runs ONNX/PyTorch models on Inferentia2 via Neuron SDK." },
    umcRole: { fr: "UMC complète SageMaker en garantissant que le checkpoint d'entraînement et le binaire Neuron sont byte-équivalents.", en: "UMC complements SageMaker by guaranteeing the training checkpoint and Neuron binary are byte-equivalent." },
  },
  xiaomi: {
    bio: { fr: "Xiaomi embarque l'IA dans HyperOS pour photo, voix et automobile (SU7).", en: "Xiaomi embeds AI in HyperOS for photography, voice, and automotive (SU7)." },
    created: [],
    uses: ["tflite", "ncnn", "onnx"],
    usage: { fr: "Modèles de retouche photo, traduction temps réel, ADAS sur Snapdragon + NPU MediaTek.", en: "Photo retouching, real-time translation, ADAS models on Snapdragon + MediaTek NPU." },
    umcRole: { fr: "UMC fournit une chaîne unifiée TFLite/NCNN signée que les firmware teams intègrent en CI.", en: "UMC provides a unified signed TFLite/NCNN chain that firmware teams integrate in CI." },
  },
  airbus: {
    bio: { fr: "Airbus utilise l'IA pour la maintenance prédictive, l'optimisation de trajectoires et l'inspection.", en: "Airbus uses AI for predictive maintenance, trajectory optimization and inspection." },
    created: [],
    uses: ["onnx", "openvino"],
    usage: { fr: "Modèles ONNX déployés sur gateways embarqués pour analyser les capteurs moteur en vol.", en: "ONNX models deployed on embedded gateways to analyze engine sensors in flight." },
    umcRole: { fr: "UMC fournit la traçabilité réglementaire EASA pour chaque version de modèle embarqué.", en: "UMC provides EASA regulatory traceability for every embedded model version." },
  },
  snapchat: {
    bio: { fr: "Snap construit des modèles de vision temps réel pour les filtres AR, déployés on-device.", en: "Snap builds real-time vision models for AR lenses, deployed on-device." },
    created: [],
    uses: ["coreml", "tflite", "onnx"],
    usage: { fr: "Lens Studio publie des modèles CoreML/TFLite quantifiés pour iOS et Android.", en: "Lens Studio publishes quantized CoreML/TFLite models for iOS and Android." },
    umcRole: { fr: "UMC accélère le cycle entraînement → publication Lens grâce à la conversion certifiée en un clic.", en: "UMC speeds up the training → Lens publish loop with one-click certified conversion." },
  },
  shopify: {
    bio: { fr: "Shopify intègre l'IA dans Sidekick pour les marchands : recommandations, copywriting, support.", en: "Shopify embeds AI in Sidekick for merchants: recommendations, copywriting, support." },
    created: [],
    uses: ["onnx", "pytorch"],
    usage: { fr: "Modèles de classification produit et de recommandation servis en ONNX dans le pipeline checkout.", en: "Product classification and recommendation models served as ONNX in the checkout pipeline." },
    umcRole: { fr: "UMC garantit la reproductibilité des modèles servis en multi-cloud (AWS + GCP).", en: "UMC guarantees reproducibility of models served across multi-cloud (AWS + GCP)." },
  },
  discord: {
    bio: { fr: "Discord utilise l'IA pour la modération, la détection de spam et la transcription vocale.", en: "Discord uses AI for moderation, spam detection and voice transcription." },
    created: [],
    uses: ["onnx", "pytorch"],
    usage: { fr: "AutoMod tourne sur des modèles PyTorch portés en ONNX pour low-latency.", en: "AutoMod runs on PyTorch models ported to ONNX for low latency." },
    umcRole: { fr: "UMC permet à Discord de pousser de nouvelles versions modérées chaque semaine sans casse silencieuse.", en: "UMC lets Discord ship new moderation versions weekly with no silent breakage." },
  },
  github: {
    bio: { fr: "GitHub (Microsoft) édite Copilot, le plus grand déploiement IA productif au monde.", en: "GitHub (Microsoft) ships Copilot, the world's largest productive AI deployment." },
    created: [],
    uses: ["onnx", "safetensors"],
    usage: { fr: "Copilot tourne sur des Codex/GPT optimisés ; les modèles Spaces utilisent ONNX Runtime.", en: "Copilot runs on optimized Codex/GPT; Spaces models use ONNX Runtime." },
    umcRole: { fr: "UMC est l'outil idéal pour les actions GitHub qui convertissent SafeTensors → GGUF en CI/CD.", en: "UMC is the perfect tool for GitHub Actions converting SafeTensors → GGUF in CI/CD." },
  },
  cohere: {
    bio: { fr: "Cohere fournit des LLM enterprise (Command, Embed) avec un focus RAG.", en: "Cohere ships enterprise LLMs (Command, Embed) with a RAG focus." },
    created: [],
    uses: ["pytorch", "safetensors", "onnx"],
    usage: { fr: "Command R+ déployé en multi-cloud, Embed v3 servi à faible latence via ONNX.", en: "Command R+ deployed multi-cloud, Embed v3 served low-latency via ONNX." },
    umcRole: { fr: "UMC valide les conversions entre datacenters pour garantir la cohérence des embeddings.", en: "UMC validates cross-datacenter conversions to guarantee embedding consistency." },
  },
  deepseek: {
    bio: { fr: "DeepSeek publie des modèles MoE ouverts (DeepSeek-V3, R1) compétitifs avec les leaders fermés.", en: "DeepSeek ships open MoE models (DeepSeek-V3, R1) competitive with closed leaders." },
    created: [],
    uses: ["safetensors", "gguf", "vllm"],
    usage: { fr: "Modèles MoE de 671B servis via vLLM, publiés aussi en GGUF pour usage local.", en: "671B MoE models served via vLLM, also published in GGUF for local use." },
    umcRole: { fr: "UMC supporte les MoE sparses et garantit le routing token correct entre formats.", en: "UMC supports sparse MoE and guarantees correct token routing across formats." },
  },
  ggerganov: {
    bio: { fr: "Georgi Gerganov est l'auteur de llama.cpp, whisper.cpp, GGML et GGUF. Référence absolue de l'inférence locale.", en: "Georgi Gerganov is the author of llama.cpp, whisper.cpp, GGML and GGUF. Absolute reference for local inference." },
    created: ["ggml", "gguf"],
    uses: ["gguf", "safetensors"],
    usage: { fr: "llama.cpp exécute n'importe quel LLM open-weights sur CPU/GPU/Metal grâce à GGUF.", en: "llama.cpp runs any open-weights LLM on CPU/GPU/Metal thanks to GGUF." },
    umcRole: { fr: "UMC reprend la spec GGUF v3 et produit des fichiers strictement compatibles llama.cpp, vérifiés round-trip.", en: "UMC follows GGUF v3 spec and produces files strictly compatible with llama.cpp, verified round-trip." },
  },
  apache: {
    bio: { fr: "Apache TVM est la fondation open-source d'un compilateur ML universel.", en: "Apache TVM is the open-source foundation for a universal ML compiler." },
    created: ["tvm"],
    uses: ["tvm", "onnx"],
    usage: { fr: "Auto-tuning de modèles ONNX vers n'importe quel matériel cible.", en: "Auto-tuning of ONNX models to any target hardware." },
    umcRole: { fr: "UMC sait préparer les inputs ONNX optimaux que TVM consomme ensuite.", en: "UMC prepares the optimal ONNX inputs TVM then consumes." },
  },
  deepmind: {
    bio: { fr: "Google DeepMind est à l'origine d'AlphaFold, Gemini et JAX.", en: "Google DeepMind is the origin of AlphaFold, Gemini and JAX." },
    created: ["jax"],
    uses: ["jax", "pytorch", "safetensors"],
    usage: { fr: "Recherche frontière sur TPU + JAX. Modèles partagés en SafeTensors sur le Hub.", en: "Frontier research on TPU + JAX. Models shared as SafeTensors on the Hub." },
    umcRole: { fr: "UMC offre la passerelle JAX → SafeTensors → tout le reste pour les modèles ouverts DeepMind.", en: "UMC provides the JAX → SafeTensors → everything else bridge for open DeepMind models." },
  },
  berkeley: {
    bio: { fr: "UC Berkeley (LMSYS, Sky Computing) a publié vLLM, Vicuna, Chatbot Arena.", en: "UC Berkeley (LMSYS, Sky Computing) shipped vLLM, Vicuna, Chatbot Arena." },
    created: ["vllm"],
    uses: ["vllm", "safetensors", "gguf"],
    usage: { fr: "vLLM est devenu le serving engine open-source de référence pour les LLM.", en: "vLLM has become the reference open-source serving engine for LLMs." },
    umcRole: { fr: "UMC prépare les checkpoints compatibles vLLM, y compris pour les MoE et quantizations exotiques.", en: "UMC prepares vLLM-compatible checkpoints, including MoE and exotic quantizations." },
  },
  w3c: {
    bio: { fr: "Le W3C définit les standards web. WebNN expose l'accélération IA native au navigateur.", en: "W3C defines web standards. WebNN exposes native AI acceleration to the browser." },
    created: ["webnn"],
    uses: ["webnn", "wasm", "onnx"],
    usage: { fr: "Chrome, Edge, Safari implémentent progressivement WebNN pour exposer le NPU au JS.", en: "Chrome, Edge, Safari progressively implement WebNN to expose the NPU to JS." },
    umcRole: { fr: "UMC exporte vers WebNN/ONNX/WASM pour viser les trois moteurs d'inférence web.", en: "UMC exports to WebNN/ONNX/WASM to target all three web inference engines." },
  },
  arm: {
    bio: { fr: "ARM conçoit les CPU NEON et les NPU Ethos qui équipent la majorité des téléphones.", en: "ARM designs the NEON CPUs and Ethos NPUs that power most phones." },
    created: [],
    uses: ["tflite", "onnx", "executorch"],
    usage: { fr: "Ethos-U/N NPU exécute des modèles TFLite quantifiés à très basse consommation.", en: "Ethos-U/N NPU runs quantized TFLite models at very low power." },
    umcRole: { fr: "UMC produit des TFLite calibrés INT8 prêts pour Ethos sans passer par l'outillage propriétaire ARM.", en: "UMC produces INT8-calibrated TFLite ready for Ethos without ARM's proprietary tooling." },
  },
};

/** Default profile generated for brands without a hand-written entry. */
function fallback(brand: BrandKey): CompanyProfile {
  const name = BRANDS[brand].name;
  const usedByThem = FORMATS.filter((f) => f.usedBy.includes(brand)).map((f) => f.slug);
  const created = FORMATS.filter((f) => f.creator === brand).map((f) => f.slug);
  return {
    bio: {
      fr: `${name} fait partie de l'écosystème IA et utilise plusieurs des formats supportés par UMC.`,
      en: `${name} is part of the AI ecosystem and uses several of the formats UMC supports.`,
    },
    created,
    uses: usedByThem.length ? usedByThem : ["onnx"],
    usage: {
      fr: `${name} déploie ses modèles à travers les formats listés ci-dessous.`,
      en: `${name} deploys its models across the formats listed below.`,
    },
    umcRole: {
      fr: `UMC permet à ${name} et à ses partenaires de convertir entre ces formats avec certificat ed25519 et δ < 1e-6.`,
      en: `UMC lets ${name} and its partners convert between these formats with an ed25519 certificate and δ < 1e-6.`,
    },
  };
}

export function getCompanyProfile(brand: BrandKey): CompanyProfile {
  return PROFILES[brand] ?? fallback(brand);
}

export function hasCompanyProfile(slug: string): slug is BrandKey {
  return slug in BRANDS;
}