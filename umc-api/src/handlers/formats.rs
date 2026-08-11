use actix_web::HttpResponse;

use crate::{errors::ApiError, models::FormatInfo};

/// GET /v1/formats — list all supported formats
pub async fn list_formats() -> Result<HttpResponse, ApiError> {
    let formats = all_formats();
    Ok(HttpResponse::Ok().json(serde_json::json!({ "formats": formats })))
}

/// GET /v1/formats/graph — conversion graph edges
pub async fn conversion_graph() -> Result<HttpResponse, ApiError> {
    let edges = build_graph_edges();
    Ok(HttpResponse::Ok().json(serde_json::json!({ "edges": edges })))
}

fn all_formats() -> Vec<FormatInfo> {
    vec![
        FormatInfo {
            slug: "gguf".into(),
            name: "GGUF".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["gguf".into()],
            description: "GPT-Generated Unified Format — llama.cpp native".into(),
        },
        FormatInfo {
            slug: "safetensors".into(),
            name: "SafeTensors".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["safetensors".into()],
            description: "HuggingFace safe tensor format".into(),
        },
        FormatInfo {
            slug: "onnx".into(),
            name: "ONNX".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["onnx".into()],
            description: "Open Neural Network Exchange format".into(),
        },
        FormatInfo {
            slug: "pytorch".into(),
            name: "PyTorch".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["pt".into(), "pth".into(), "bin".into()],
            description: "PyTorch checkpoint format".into(),
        },
        FormatInfo {
            slug: "awq".into(),
            name: "AWQ".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["safetensors".into()],
            description: "Activation-aware Weight Quantization".into(),
        },
        FormatInfo {
            slug: "gptq".into(),
            name: "GPTQ".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["safetensors".into()],
            description: "Generative Pre-trained Transformer Quantization".into(),
        },
        FormatInfo {
            slug: "tflite".into(),
            name: "TFLite".into(),
            can_read: true,
            can_write: true,
            native: true,
            extensions: vec!["tflite".into()],
            description: "TensorFlow Lite FlatBuffer format".into(),
        },
        FormatInfo {
            slug: "coreml".into(),
            name: "CoreML".into(),
            can_read: false,
            can_write: true,
            native: false,
            extensions: vec!["mlpackage".into(), "mlmodel".into()],
            description: "Apple CoreML format (config generation)".into(),
        },
        FormatInfo {
            slug: "tensorrt".into(),
            name: "TensorRT".into(),
            can_read: false,
            can_write: true,
            native: false,
            extensions: vec!["engine".into()],
            description: "NVIDIA TensorRT engine (config generation)".into(),
        },
        FormatInfo {
            slug: "openvino".into(),
            name: "OpenVINO".into(),
            can_read: false,
            can_write: true,
            native: false,
            extensions: vec!["xml".into()],
            description: "Intel OpenVINO IR format".into(),
        },
        FormatInfo {
            slug: "executorch".into(),
            name: "ExecuTorch".into(),
            can_read: false,
            can_write: true,
            native: false,
            extensions: vec!["pte".into()],
            description: "Meta ExecuTorch on-device format".into(),
        },
        FormatInfo {
            slug: "lora".into(),
            name: "LoRA Adapter".into(),
            can_read: true,
            can_write: false,
            native: true,
            extensions: vec!["safetensors".into(), "bin".into()],
            description: "Low-Rank Adaptation weights".into(),
        },
    ]
}

fn build_graph_edges() -> Vec<serde_json::Value> {
    // All native read→write combinations
    let readers = &[
        "gguf",
        "safetensors",
        "onnx",
        "pytorch",
        "awq",
        "gptq",
        "tflite",
    ];
    let writers = &[
        "gguf",
        "safetensors",
        "onnx",
        "pytorch",
        "awq",
        "gptq",
        "tflite",
        "coreml",
        "tensorrt",
        "openvino",
        "executorch",
    ];

    let mut edges = vec![];
    for &src in readers {
        for &dst in writers {
            if src != dst {
                edges.push(serde_json::json!({
                    "from": src,
                    "to": dst,
                    "cost": 1.0
                }));
            }
        }
    }
    edges
}
