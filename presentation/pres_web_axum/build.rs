// build.rs
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Emit the instructions
    EmitBuilder::builder().all_build().all_git().emit()?;
    Ok(())
}
