use cargo_gpu::install::Install;
use cargo_gpu::spirv_builder::{ShaderPanicStrategy, SpirvMetadata};
use std::path::PathBuf;

fn compile_shader(manifest_dir: &str, shader_name: &str, env_var_name: &str) -> anyhow::Result<()> {
    let crate_path: PathBuf = [manifest_dir, "..", shader_name].iter().copied().collect();

    let install = Install::from_shader_crate(crate_path.clone()).run()?;
    let mut builder = install.to_spirv_builder(crate_path, "spirv-unknown-vulkan1.3");
    builder.build_script.defaults = true;
    builder.shader_panic_strategy = ShaderPanicStrategy::SilentExit;
    builder.spirv_metadata = SpirvMetadata::Full;

    let compile_result = builder.build()?;
    let spv_path = compile_result.module.unwrap_single();
    println!("cargo::rustc-env={}={}", env_var_name, spv_path.display());
    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Compile all shader crates
    compile_shader(manifest_dir, "sierpinskie-triangle", "SHADER_TRIANGLE_SPV")?;
    compile_shader(manifest_dir, "sierpinskie-carpet", "SHADER_CARPET_SPV")?;
    compile_shader(manifest_dir, "koch-curve", "SHADER_KOCH_SPV")?;
    compile_shader(manifest_dir, "mandelbrotset", "SHADER_MANDELBROT_SPV")?;

    Ok(())
}
