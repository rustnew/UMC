use std::path::Path;
use umc_core::{FormatSaver, ProgressCallback, SaveOptions};
use umc_core::{UmcError, UniversalIR};

pub struct AwqSaver;

impl FormatSaver for AwqSaver {
    fn format_name(&self) -> &'static str {
        "AWQ"
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
        // AWQ output = SafeTensors with AWQ-specific __metadata__
        // We delegate to SafeTensors saver and inject AWQ metadata via IR
        let mut ir_clone = ir.clone();
        ir_clone
            .metadata
            .insert("__format__", umc_core::ir::MetaValue::String("AWQ".into()));
        ir_clone
            .metadata
            .insert("quant_type", umc_core::ir::MetaValue::String("awq".into()));
        if let Some(bits) = ir.metadata.get_i64("quantization.bits") {
            ir_clone
                .metadata
                .insert("w_bit", umc_core::ir::MetaValue::I64(bits));
        }
        if let Some(gs) = ir.metadata.get_i64("quantization.group_size") {
            ir_clone
                .metadata
                .insert("q_group_size", umc_core::ir::MetaValue::I64(gs));
        }

        let saver = crate::safetensors::SafeTensorsSaver;
        saver.save(&ir_clone, path, opts, progress)
    }
}
