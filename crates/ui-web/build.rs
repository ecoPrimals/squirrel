use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use glob::glob;

fn main() {
    println!("cargo:rerun-if-changed=web");
    
    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("web_assets");
    
    // Create the destination directory if it doesn't exist
    fs::create_dir_all(&dest_path).unwrap();
    
    // Copy all web assets to the output directory
    copy_dir_recursive("web", &dest_path);
    
    // Generate a file that includes asset paths
    generate_asset_paths(&dest_path);
    
    println!("cargo:warning=Web assets copied to: {}", dest_path.display());
}

fn copy_dir_recursive(src_dir: impl AsRef<Path>, dest_dir: impl AsRef<Path>) {
    let src_dir = src_dir.as_ref();
    let dest_dir = dest_dir.as_ref();
    
    // Create the destination directory if it doesn't exist
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir).unwrap();
    }
    
    // Copy all files in the directory
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dest_path = dest_dir.join(file_name);
        
        if path.is_dir() {
            // Recursively copy subdirectories
            copy_dir_recursive(&path, &dest_path);
        } else {
            // Copy the file
            fs::copy(&path, &dest_path).unwrap();
            println!("cargo:warning=Copied: {} -> {}", path.display(), dest_path.display());
        }
    }
}

fn generate_asset_paths(assets_dir: impl AsRef<Path>) {
    let assets_dir = assets_dir.as_ref();
    let out_dir = env::var("OUT_DIR").unwrap();
    let asset_paths_file = Path::new(&out_dir).join("asset_paths.rs");
    
    // Find all assets
    let mut asset_paths = Vec::new();
    
    // Get all files recursively
    let pattern = assets_dir.join("**/*.*").to_string_lossy().to_string();
    for entry in glob(&pattern).unwrap() {
        if let Ok(path) = entry {
            if path.is_file() {
                let rel_path = path.strip_prefix(assets_dir).unwrap();
                asset_paths.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    
    // Generate Rust code for asset paths
    let mut asset_paths_code = String::new();
    asset_paths_code.push_str("/// Auto-generated map of asset paths\n");
    asset_paths_code.push_str("pub fn asset_paths() -> std::collections::HashMap<&'static str, &'static [u8]> {\n");
    asset_paths_code.push_str("    let mut map = std::collections::HashMap::new();\n");
    
    // Add each asset to the map
    for path in asset_paths {
        let full_path = assets_dir.join(&path);
        asset_paths_code.push_str(&format!(
            "    map.insert(\"{}\", include_bytes!(\"{}\"));\n",
            path.replace("\\", "/"),
            full_path.to_string_lossy().replace("\\", "/")
        ));
    }
    
    asset_paths_code.push_str("    map\n");
    asset_paths_code.push_str("}\n");
    
    // Write the generated code to a file
    fs::write(asset_paths_file, asset_paths_code).unwrap();
} 