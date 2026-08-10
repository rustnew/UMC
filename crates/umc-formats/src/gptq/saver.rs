use std::path::Path;
use umc_core::{FormatSaver, ProgressCallback, SaveOptions};
use umc_core::{UmcError, UniversalIR};

pub struct GptqSaver;

impl FormatSaver for GptqSaver {
    fn format_name(&self) -> &'static str {
        "GPTQ"
    }
    fn default_extension(&self) -> &'static str {
        "safetensors"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        let mut ir_clone = ir.clone();
        ir_clone
            .metadata
            .insert("__format__", umc_core::ir::MetaValue::String("GPTQ".into()));
        ir_clone.metadata.insert(
            "quant_method",
            umc_core::ir::MetaValue::String("gptq".into()),
        );
        if let Some(bits) = ir.metadata.get_i64("quantization.bits") {
            ir_clone
                .metadata
                .insert("bits", umc_core::ir::MetaValue::I64(bits));
        }
        if let Some(gs) = ir.metadata.get_i64("quantization.group_size") {
            ir_clone
                .metadata
                .insert("group_size", umc_core::ir::MetaValue::I64(gs));
        }
        let saver = crate::safetensors::SafeTensorsSaver;
        saver.save(&ir_clone, path, opts, progress)
    }
}
