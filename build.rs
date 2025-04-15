use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Tell cargo to rerun this script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("python_lib_info.rs");
    let mut file = File::create(&dest_path).unwrap();

    let python_executable = detect_python_executable();
    let python_version = get_python_version(&python_executable);
    let sys_prefix = get_python_sys_prefix(&python_executable);
    let lib_dirs = find_python_library_dirs(&python_executable, &python_version);

    // Output Python information for debugging
    println!("cargo:warning=Python executable: {}", python_executable);
    println!("cargo:warning=Python version: {}", python_version);
    println!("cargo:warning=Python sys.prefix: {}", sys_prefix);
    println!("cargo:warning=Python library dirs: {:?}", lib_dirs);

    // Write Python information to the generated Rust file
    writeln!(file, "pub fn get_python_version() -> &'static str {{").unwrap();
    writeln!(file, "    \"{}\"", python_version).unwrap();
    writeln!(file, "}}").unwrap();

    writeln!(file, "pub fn get_python_lib_dirs() -> Vec<String> {{").unwrap();
    writeln!(file, "    vec![").unwrap();
    for dir in &lib_dirs {
        writeln!(file, "        String::from(\"{}\"),", dir.replace("\\", "\\\\")).unwrap();
    }
    writeln!(file, "    ]").unwrap();
    writeln!(file, "}}").unwrap();

    // Add library search paths
    for dir in lib_dirs {
        println!("cargo:rustc-link-search={}", dir);
    }

    // Explicitly add Miniconda lib directory
    println!("cargo:rustc-link-search=/home/southgate/miniconda3/lib");

    // Link to the Python library
    // Use simplified library name without version on Linux (libpython3.so)
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=python3.12");
        // Fallback options
        println!("cargo:rustc-link-lib=python3");
    } else if cfg!(target_os = "windows") {
        let version_nodot = python_version.replace(".", "");
        println!("cargo:rustc-link-lib=python{}", version_nodot);
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=python{}", python_version);
    }
}

/// Detect Python executable by checking environment variable or searching common paths
fn detect_python_executable() -> String {
    // 1. Check environment variable
    if let Ok(executable) = env::var("PYTHON_EXECUTABLE") {
        return executable;
    }

    // 2. Try common names
    let candidates = if cfg!(target_os = "windows") {
        vec!["python3", "python", "python3.exe", "python.exe"]
    } else {
        vec!["python3", "python"]
    };

    for candidate in candidates {
        if let Ok(output) = Command::new(candidate)
            .arg("--version")
            .output() {
            if output.status.success() {
                return candidate.to_string();
            }
        }
    }

    // 3. Check specific paths in Miniconda/Anaconda
    if cfg!(target_os = "linux") {
        let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/southgate"));
        let miniconda_path = format!("{}/miniconda3/bin/python", home_dir);

        if let Ok(output) = Command::new(&miniconda_path)
            .arg("--version")
            .output() {
            if output.status.success() {
                return miniconda_path;
            }
        }
    }

    // Default fallback
    String::from("python3")
}

/// Get Python version by running python --version
fn get_python_version(python_executable: &str) -> String {
    let output = Command::new(python_executable)
        .args(["-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"])
        .output()
        .expect("Failed to run Python to get version");

    String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in Python version output")
        .trim()
        .to_string()
}

/// Get Python sys.prefix directory
fn get_python_sys_prefix(python_executable: &str) -> String {
    let output = Command::new(python_executable)
        .args(["-c", "import sys; print(sys.prefix)"])
        .output()
        .expect("Failed to get Python sys.prefix");

    String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in Python sys.prefix output")
        .trim()
        .to_string()
}

/// Find potential directories where Python libraries might be located
fn find_python_library_dirs(python_executable: &str, python_version: &str) -> Vec<String> {
    let mut dirs = Vec::new();
    
    // 1. Get sys.prefix/lib paths
    let sys_prefix = get_python_sys_prefix(python_executable);
    
    // 2. Get Python's library paths from sysconfig
    let output = Command::new(python_executable)
        .args(["-c", "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')"])
        .output()
        .expect("Failed to get Python LIBDIR");
    
    let lib_dir = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in Python LIBDIR output")
        .trim()
        .to_string();
    
    if !lib_dir.is_empty() {
        dirs.push(lib_dir);
    }
    
    // 3. Add standard library directories based on platform
    if cfg!(target_os = "linux") {
        // Standard Linux paths
        dirs.push(format!("{}/lib", sys_prefix));
        dirs.push(format!("/usr/lib"));
        dirs.push(format!("/usr/local/lib"));
        
        // Add Miniconda path
        let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/southgate"));
        dirs.push(format!("{}/miniconda3/lib", home_dir));
        
        // Add Python version-specific paths
        dirs.push(format!("/usr/lib/python{}", python_version));
        dirs.push(format!("/usr/local/lib/python{}", python_version));
    } else if cfg!(target_os = "macos") {
        // macOS paths
        dirs.push(format!("{}/lib", sys_prefix));
        dirs.push(format!("/usr/local/lib"));
        dirs.push(format!("/usr/local/opt/python/lib"));
        dirs.push(format!("/opt/homebrew/lib"));
    } else if cfg!(target_os = "windows") {
        // Windows paths
        dirs.push(format!("{}\\libs", sys_prefix));
        dirs.push(format!("{}\\Lib", sys_prefix));
    }
    
    // 4. Check if directories exist and contain Python library (filter out non-existent paths)
    dirs.into_iter()
        .filter(|dir| Path::new(dir).exists())
        .collect()
} 