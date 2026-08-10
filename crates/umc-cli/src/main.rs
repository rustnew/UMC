use clap::{Parser, Subcommand};
use std::path::PathBuf;
use umc_core::FormatLoader;
use umc_core::{LoadOptions, UMC_VERSION};
use umc_detect::FormatRegistry;
use umc_formats::{GgufLoader, SafeTensorsLoader};
use umc_graph::{find_path, ConversionGraph};
use umc_pipeline::{ConversionPipeline, ConversionRequest};
use umc_validate::ValidationMode;

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "umc",
    version = UMC_VERSION,
    about = "UMC — Universal Model Converter\nThe ffmpeg of AI models.",
    long_about = None,
)]
struct Cli {
    /// Enable verbose output (set UMC_LOG=debug for full traces)
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a model from one format to another
    Convert {
        /// Input model file (format auto-detected from magic bytes)
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output model file
        #[arg(value_name = "OUTPUT")]
        output: PathBuf,

        /// Force source format (e.g. GGUF, SafeTensors, ONNX)
        #[arg(long, value_name = "FORMAT")]
        from: Option<String>,

        /// Force target format
        #[arg(long, value_name = "FORMAT")]
        to: Option<String>,

        /// Output dtype override (e.g. F32, F16, BF16)
        #[arg(long, value_name = "DTYPE")]
        dtype: Option<String>,

        /// Validation mode: none, structural, numeric, strict
        #[arg(long, default_value = "structural")]
        validate: String,

        /// Skip tensor data (metadata inspection only)
        #[arg(long)]
        metadata_only: bool,
    },

    /// Inspect a model file (print metadata and tensor info)
    Inspect {
        /// Model file to inspect
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Force format
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,

        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        output: String,

        /// Number of tensors to display (0 = all)
        #[arg(long, default_value = "20")]
        max_tensors: usize,
    },

    /// Show all supported formats
    Formats,

    /// Find the conversion path between two formats
    Path {
        /// Source format
        from: String,
        /// Target format
        to: String,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    let result = run(cli.command);
    if let Err(e) = result {
        eprintln!("\n\x1b[31mError:\x1b[0m {}\n", e);
        std::process::exit(1);
    }
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = if verbose {
        EnvFilter::new("umc=debug")
    } else {
        EnvFilter::from_env("UMC_LOG").add_directive("umc=info".parse().unwrap())
    };
    let _ = fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

fn run(cmd: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::Convert {
            input,
            output,
            from,
            to,
            dtype,
            validate,
            metadata_only,
        } => cmd_convert(input, output, from, to, dtype, validate, metadata_only),
        Commands::Inspect {
            file,
            format,
            output,
            max_tensors,
        } => cmd_inspect(file, format, output, max_tensors),
        Commands::Formats => cmd_formats(),
        Commands::Path { from, to } => cmd_path(from, to),
    }
}

// ── convert ───────────────────────────────────────────────────────────────────

fn cmd_convert(
    input: PathBuf,
    output: PathBuf,
    from: Option<String>,
    to: Option<String>,
    _dtype: Option<String>,
    validate: String,
    metadata_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let validation_mode = match validate.as_str() {
        "none" => ValidationMode::None,
        "structural" => ValidationMode::Structural,
        "numeric" => ValidationMode::Numeric,
        "strict" => ValidationMode::Strict,
        other => {
            eprintln!("Unknown validation mode '{}'. Using 'structural'.", other);
            ValidationMode::Structural
        }
    };

    let pipeline = ConversionPipeline::new();
    let progress = umc_core::ProgressCallback::stderr();

    let mut req = ConversionRequest::new(&input, &output);
    req.source_format = from;
    req.target_format = to;
    req.validation_mode = validation_mode;
    req.load_options.metadata_only = metadata_only;

    eprintln!(
        "\n\x1b[1mUMC v{}\x1b[0m — Universal Model Converter",
        UMC_VERSION
    );
    eprintln!("  Input:  {}", input.display());
    eprintln!("  Output: {}", output.display());
    eprintln!();

    let result = pipeline.convert(req, &progress)?;
    eprintln!();
    eprintln!("\x1b[32m✓\x1b[0m {}", result.summary());

    if !result.warnings.is_empty() {
        eprintln!("\n\x1b[33mWarnings:\x1b[0m");
        for w in &result.warnings {
            eprintln!("  ⚠  {}", w);
        }
    }

    if let Some(cert) = result.certificate {
        eprintln!("\n\x1b[32mCertificate\x1b[0m");
        eprintln!("  Body hash:  {}", cert.body_hash);
        eprintln!("  Signature:  {}", cert.signature);
    }

    eprintln!();
    Ok(())
}

// ── inspect ───────────────────────────────────────────────────────────────────

fn cmd_inspect(
    file: PathBuf,
    format: Option<String>,
    output_fmt: String,
    max_tensors: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = FormatRegistry::new();
    let detected = if let Some(f) = format {
        f
    } else {
        registry.detect(&file)?.format
    };

    let loader: Box<dyn FormatLoader> = match detected.as_str() {
        "GGUF" => Box::new(GgufLoader),
        "SafeTensors" => Box::new(SafeTensorsLoader),
        other => {
            return Err(format!("No loader for format '{}'", other).into());
        }
    };

    let mut opts = LoadOptions::default();
    opts.metadata_only = false;
    let ir = loader.load(&file, &opts, &umc_core::ProgressCallback::noop())?;

    if output_fmt == "json" {
        print_inspect_json(&ir, &detected, max_tensors)?;
    } else {
        print_inspect_text(&ir, &detected, max_tensors);
    }

    Ok(())
}

fn print_inspect_text(ir: &umc_core::UniversalIR, format: &str, max_tensors: usize) {
    println!("\n\x1b[1mModel Inspection\x1b[0m");
    println!("  Format:        {}", format);
    println!("  Architecture:  {}", ir.architecture.architecture);
    println!("  Model type:    {}", ir.architecture.model_type);
    println!("  Hidden size:   {}", ir.architecture.hidden_size);
    println!("  Layers:        {}", ir.architecture.num_layers);
    println!("  Heads:         {}", ir.architecture.num_heads);
    if let Some(kv) = ir.architecture.num_kv_heads {
        println!("  KV heads:      {}", kv);
    }
    println!("  Vocab size:    {}", ir.architecture.vocab_size);
    println!(
        "  Max context:   {}",
        ir.architecture.max_position_embeddings
    );
    println!("  Tensors:       {}", ir.tensors.len());
    println!("  Parameters:    {:.2}B", ir.num_parameters() as f64 / 1e9);
    println!("  RAM usage:     {:.1} MiB", ir.tensors.ram_usage_mb());

    if let Some(ref q) = ir.quantization {
        println!("  Quantization:  {:?}", q.scheme);
    }

    println!("\n\x1b[1mMetadata\x1b[0m");
    for (k, v) in ir.metadata.iter().take(30) {
        println!("  {:50} = {:?}", k, v);
    }
    if ir.metadata.len() > 30 {
        println!("  … and {} more entries", ir.metadata.len() - 30);
    }

    println!(
        "\n\x1b[1mTensors\x1b[0m (showing {} of {})",
        max_tensors.min(ir.tensors.len()),
        ir.tensors.len()
    );
    for (name, tensor) in ir.tensors.iter().take(max_tensors) {
        let shape_str: Vec<String> = tensor.shape.iter().map(|d| d.to_string()).collect();
        println!(
            "  {:60} {:10}  [{:}]",
            name,
            tensor.dtype.as_str(),
            shape_str.join(", ")
        );
    }
    if ir.tensors.len() > max_tensors {
        println!("  … and {} more tensors", ir.tensors.len() - max_tensors);
    }
    println!();
}

fn print_inspect_json(
    ir: &umc_core::UniversalIR,
    format: &str,
    max_tensors: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let tensors: Vec<serde_json::Value> = ir
        .tensors
        .iter()
        .take(max_tensors)
        .map(|(name, t)| {
            serde_json::json!({
                "name": name,
                "dtype": t.dtype.as_str(),
                "shape": t.shape,
                "num_elements": t.num_elements(),
            })
        })
        .collect();

    let output = serde_json::json!({
        "format": format,
        "architecture": ir.architecture.architecture,
        "hidden_size": ir.architecture.hidden_size,
        "num_layers": ir.architecture.num_layers,
        "num_heads": ir.architecture.num_heads,
        "vocab_size": ir.architecture.vocab_size,
        "tensor_count": ir.tensors.len(),
        "parameters_billions": ir.num_parameters() as f64 / 1e9,
        "tensors": tensors,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// ── formats ───────────────────────────────────────────────────────────────────

fn cmd_formats() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\x1b[1mUMC Supported Formats\x1b[0m\n");
    println!(
        "  {:<20} {:<12} {:<12} {}",
        "Format", "Load", "Save", "Notes"
    );
    println!("  {}", "-".repeat(70));

    let formats = [
        (
            "GGUF",
            "✓ native",
            "planned",
            "GGUF v1/v2/v3, all quant types",
        ),
        (
            "SafeTensors",
            "✓ native",
            "✓ native",
            "HuggingFace SafeTensors",
        ),
        ("ONNX", "planned", "planned", "ONNX opset 13-21"),
        (
            "PyTorch",
            "planned",
            "planned",
            "PyTorch .pt/.pth (safe pickle)",
        ),
        ("TFLite", "planned", "planned", "TFLite FlatBuffers"),
        ("KerasH5", "planned", "—", "Keras H5 (read-only)"),
        ("GGML", "planned", "—", "Legacy GGML (read-only)"),
        (
            "TFSavedModel",
            "planned",
            "planned",
            "TensorFlow SavedModel",
        ),
        ("AWQ", "planned", "—", "AWQ quantized models"),
        ("GPTQ", "planned", "—", "GPTQ quantized models"),
        ("Diffusers", "planned", "planned", "HuggingFace Diffusers"),
    ];

    for (name, load, save, notes) in &formats {
        println!("  {:<20} {:<12} {:<12} {}", name, load, save, notes);
    }
    println!();
    Ok(())
}

// ── path ─────────────────────────────────────────────────────────────────────

fn cmd_path(from: String, to: String) -> Result<(), Box<dyn std::error::Error>> {
    let graph = ConversionGraph::default_graph();
    let path = find_path(&graph, &from, &to)?;

    println!("\n\x1b[1mConversion path: {} → {}\x1b[0m", from, to);
    println!("  Hops:       {}", path.hop_count());
    println!("  Total cost: {:.1}s (estimated)", path.total_cost);
    println!(
        "  All native: {}",
        if path.all_native() {
            "yes"
        } else {
            "no (external tool required)"
        }
    );
    println!("\n  Steps:");
    for (i, hop) in path.hops.iter().enumerate() {
        let native_tag = if hop.native {
            "\x1b[32m[native]\x1b[0m"
        } else {
            "\x1b[33m[external]\x1b[0m"
        };
        println!(
            "    {}. {} → {}  {}  {}",
            i + 1,
            hop.source,
            hop.target,
            native_tag,
            hop.description
        );
    }
    println!();
    Ok(())
}
