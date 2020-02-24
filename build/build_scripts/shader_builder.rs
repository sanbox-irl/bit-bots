use super::{serialization_util, Error};
use log::info;
use serialization_util::SerializedMetaData;
use shaderc::{Compiler, ShaderKind};
use std::collections::HashMap;

const GEN_DIR: &str = "assets/gen/shaders";
const INPUT_DIR: &str = "assets/shaders";

pub fn build() -> Result<(), Error> {
    std::fs::create_dir_all(GEN_DIR)?;

    let mut manifest: HashMap<String, SerializedMetaData> =
        serialization_util::load(&shader_path("manifest.yaml")).unwrap_or_default();

    let mut recompiled_shaders = false;
    let mut compiler = Compiler::new().unwrap();

    for entry in std::fs::read_dir(INPUT_DIR)? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let in_path: std::path::PathBuf = entry.path();

            // Needs to end in .vert or .frag.
            let shader_type = in_path
                .extension()
                .and_then(|ext| match ext.to_string_lossy().as_ref() {
                    "vert" => Some(ShaderKind::Vertex),
                    "frag" => Some(ShaderKind::Fragment),
                    _ => None,
                });

            if let Some(shader_type) = shader_type {
                let filename: &str = in_path.file_name().unwrap().to_str().unwrap();
                let current_metadata = std::fs::metadata(&in_path)?;

                let recompile_this_shader = match manifest.get(filename) {
                    Some(ser_metadata) => {
                        if current_metadata.created()? != ser_metadata.created
                            || current_metadata.modified()? != ser_metadata.modified
                        {
                            info!("{} has been changed.", filename);
                            manifest.insert(filename.to_string(), SerializedMetaData::new(current_metadata)?);
                            true
                        } else {
                            false
                        }
                    }

                    None => {
                        info!("{} was not present in manifest.", filename);
                        manifest.insert(filename.to_string(), SerializedMetaData::new(current_metadata)?);
                        true
                    }
                };

                if recompile_this_shader {
                    recompiled_shaders = true;
                    let source = std::fs::read_to_string(&in_path)?;

                    let compiled_file =
                        compiler.compile_into_spirv(&source, shader_type, filename, "main", None)?;

                    std::fs::write(
                        &shader_path(&format!("{}.spv", filename)),
                        &compiled_file.as_binary_u8(),
                    )?;
                }
            }
        }
    }

    if recompiled_shaders {
        serialization_util::save(&manifest, &shader_path("manifest.yaml"))?;
    } else {
        info!("No shaders were recompiled.");
    }

    Ok(())
}

fn shader_path(sub_dir: &str) -> String {
    format!("{}/{}", GEN_DIR, sub_dir)
}
