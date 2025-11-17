use std::env;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to re-run this build script if registry/ changes
    println!("cargo:rerun-if-changed=registry/");
    
    // The registry directory will be embedded using include_dir! in the actual code
    // For now, we just ensure the directory exists
    let registry_path = PathBuf::from("registry");
    if !registry_path.exists() {
        std::fs::create_dir_all(&registry_path)
            .expect("Failed to create registry directory");
    }
    
    println!("cargo:rustc-env=REGISTRY_DIR={}", registry_path.display());
}
