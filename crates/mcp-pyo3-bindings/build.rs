use std::process::Command;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;

/// Auto-detects Python installations on the system
fn find_python_installations() -> Vec<PathBuf> {
    let mut installations = Vec::new();
    
    // Check common environment variables
    for var in &["PYTHONHOME", "VIRTUAL_ENV", "CONDA_PREFIX"] {
        if let Ok(path) = env::var(var) {
            installations.push(PathBuf::from(&path));
        }
    }
    
    // Platform-specific default locations
    if cfg!(target_os = "windows") {
        // Windows common locations
        for dir in &[
            r"C:\Python312",
            r"C:\Python311",
            r"C:\Python310",
            r"C:\Python39",
            r"C:\Program Files\Python312",
            r"C:\Program Files\Python311",
            r"C:\Program Files (x86)\Python312",
            r"C:\Program Files (x86)\Python311",
            r"C:\Anaconda3",
            r"C:\ProgramData\Anaconda3",
            r"C:\Users\All Users\Miniconda3",
        ] {
            installations.push(PathBuf::from(dir));
        }
        
        // Check Windows user directory for Miniconda/Anaconda
        if let Ok(userprofile) = env::var("USERPROFILE") {
            installations.push(PathBuf::from(&userprofile).join("Anaconda3"));
            installations.push(PathBuf::from(&userprofile).join("Miniconda3"));
        }
    } else if cfg!(target_os = "macos") {
        // macOS common locations
        for dir in &[
            "/usr/local/bin/python3",
            "/usr/bin/python3",
            "/opt/homebrew/bin/python3",
            "/opt/python/bin/python3",
            "/usr/local/opt/python/libexec/bin/python",
            "/usr/local/Cellar/python",
            "/opt/homebrew/Cellar/python",
            "/opt/anaconda3",
            "/opt/miniconda3",
        ] {
            installations.push(PathBuf::from(dir));
        }
        
        // Check brew-installed pythons
        let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/Users"));
        installations.push(PathBuf::from(&home_dir).join("anaconda3"));
        installations.push(PathBuf::from(&home_dir).join("miniconda3"));
    } else {
        // Linux common locations
        for dir in &[
            "/usr/bin/python3",
            "/usr/local/bin/python3",
            "/opt/python3",
            "/opt/anaconda3",
            "/opt/miniconda3",
        ] {
            installations.push(PathBuf::from(dir));
        }
        
        // Check home directory for Anaconda/Miniconda
        if let Ok(home) = env::var("HOME") {
            installations.push(PathBuf::from(&home).join("anaconda3"));
            installations.push(PathBuf::from(&home).join("miniconda3"));
        }
    }

    // Add project-specific locations
    if let Ok(cargo_manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let cargo_manifest_path = PathBuf::from(cargo_manifest_dir);
        let workspace_root = cargo_manifest_path
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new(&cargo_manifest_path));
        
        installations.push(workspace_root.join("pyo3-test-env"));
        installations.push(workspace_root.join("venv"));
        installations.push(workspace_root.join("env"));
        installations.push(workspace_root.join(".venv"));
        installations.push(workspace_root.join(".env"));
    }
    
    // Try to get active conda environment
    if let Ok(conda_prefix) = env::var("CONDA_PREFIX") {
        installations.push(PathBuf::from(conda_prefix));
    }

    // Try to add the current Python from PATH
    if let Some(python_path) = find_python_executable() {
        if let Some(parent) = python_path.parent() {
            installations.push(parent.to_path_buf());
            // Also add parent directory of the executable directory (for lib)
            if let Some(grandparent) = parent.parent() {
                installations.push(grandparent.to_path_buf());
            }
        }
    }
    
    // Filter to only existing paths
    installations.retain(|path| path.exists());
    
    installations
}

/// Find the Python executable from environment variables or PATH
fn find_python_executable() -> Option<PathBuf> {
    // Check common Python executable names
    let possible_commands = if cfg!(target_os = "windows") {
        vec!["python.exe", "python3.exe", "python312.exe", "python311.exe"]
    } else {
        vec!["python3.12", "python3.11", "python3.10", "python3", "python"]
    };

    for cmd in possible_commands {
        if let Ok(output) = Command::new(cmd)
            .arg("-c")
            .arg("import sys; print(sys.executable)")
            .output() 
        {
            if output.status.success() {
                if let Ok(path_str) = String::from_utf8(output.stdout) {
                    let path_str = path_str.trim();
                    if !path_str.is_empty() {
                        return Some(PathBuf::from(path_str));
                    }
                }
            }
        }
    }
    
    None
}

/// Get Python version from an executable path
fn get_python_version(python_exe: &Path) -> Option<String> {
    Command::new(python_exe)
        .args(["-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

/// Get the site packages directory for a Python installation
fn get_site_packages_dir(python_exe: &Path) -> Option<PathBuf> {
    Command::new(python_exe)
        .args(["-c", "import site; print(site.getsitepackages()[0])"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Some(PathBuf::from(path))
            } else {
                None
            }
        })
}

/// Find library directories for a Python installation
fn find_python_lib_dirs(python_exe: &Path) -> Vec<PathBuf> {
    let mut lib_dirs = Vec::new();
    
    // Get Python prefix
    let prefix = Command::new(python_exe)
        .args(["-c", "import sys; print(sys.prefix)"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });
    
    if let Some(prefix) = prefix {
        let prefix_path = PathBuf::from(prefix);
        
        // Common library directories
        for lib_dir in &["lib", "lib64", "libs", "DLLs"] {
            let dir = prefix_path.join(lib_dir);
            if dir.exists() {
                lib_dirs.push(dir);
            }
        }
        
        // Get Python version
        if let Some(version) = get_python_version(python_exe) {
            // Add version-specific lib directories
            for lib_dir in &["lib", "lib64"] {
                let py_lib_dir = prefix_path.join(lib_dir).join(format!("python{}", version));
                if py_lib_dir.exists() {
                    lib_dirs.push(py_lib_dir.clone());
                    
                    // Check for lib-dynload subdirectory
                    let dynload_dir = py_lib_dir.join("lib-dynload");
                    if dynload_dir.exists() {
                        lib_dirs.push(dynload_dir);
                    }
                }
            }
        }
        
        // Try to get LIBDIR directly from Python
        let libdir = Command::new(python_exe)
            .args(["-c", "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() && PathBuf::from(&path).exists() {
                        Some(PathBuf::from(path))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        
        if let Some(dir) = libdir {
            if !lib_dirs.contains(&dir) {
                lib_dirs.push(dir);
            }
        }
    }
    
    lib_dirs
}

/// Find Python library files within the provided directories
fn find_python_lib_files(lib_dirs: &[PathBuf], version: &str) -> Vec<(PathBuf, String)> {
    let mut lib_files = Vec::new();
    
    let patterns = if cfg!(target_os = "windows") {
        vec![
            format!("python{}.dll", version.replace(".", "")),
            format!("python{}.dll", version),
            format!("python{}.dll", version.split('.').next().unwrap_or("3")),
            "python3.dll".to_string(),
            "python.dll".to_string(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            format!("libpython{}.dylib", version.replace(".", "")),
            format!("libpython{}.dylib", version),
            format!("libpython{}.dylib", version.split('.').next().unwrap_or("3")),
            format!("libpython{}.a", version.replace(".", "")),
            format!("libpython{}.a", version),
            "libpython3.dylib".to_string(),
            "libpython.dylib".to_string(),
        ]
    } else {
        // Linux
        vec![
            format!("libpython{}.so", version.replace(".", "")),
            format!("libpython{}.so", version),
            format!("libpython{}.so", version.split('.').next().unwrap_or("3")),
            format!("libpython{}.so.1.0", version.replace(".", "")),
            format!("libpython{}.so.1.0", version),
            "libpython3.so".to_string(),
            "libpython.so".to_string(),
        ]
    };
    
    for dir in lib_dirs {
        if dir.exists() {
            for pattern in &patterns {
                let lib_path = dir.join(pattern);
                if lib_path.exists() {
                    lib_files.push((lib_path.clone(), pattern.clone()));
                }
            }
        }
    }
    
    lib_files
}

/// Find a Python library in the provided installation directories
fn find_lib(installation_dirs: &[PathBuf]) -> Option<(PathBuf, String)> {
    for dir in installation_dirs {
        if let Some(python_exe) = find_executable_in_dir(dir) {
            if let Some(version) = get_python_version(&python_exe) {
                // Get library directories for this Python installation
                let lib_dirs = find_python_lib_dirs(&python_exe);
                
                // Find library files in those directories
                let lib_files = find_python_lib_files(&lib_dirs, &version);
                
                if !lib_files.is_empty() {
                    return Some(lib_files[0].clone());
                }
            }
        }
    }
    
    None
}

/// Find Python executable in a directory
fn find_executable_in_dir(dir: &Path) -> Option<PathBuf> {
    let exe_names = if cfg!(target_os = "windows") {
        vec!["python.exe", "python3.exe"]
    } else {
        vec!["python3", "python"]
    };
    
    for name in exe_names {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
        
        // Also check in the bin subdirectory
        let bin_path = dir.join("bin").join(name);
        if bin_path.exists() {
            return Some(bin_path);
        }
    }
    
    None
}

/// Check for specific Python library files and add direct links if found
fn add_specific_library_links() {
    // On Linux, we need to remove the 'lib' prefix and '.so' suffix when passing to rustc-link-lib
    // For example, libpython3.12.so should become python3.12
    let specific_lib_paths = [
        "/home/southgate/miniconda3/lib/libpython3.12.so",
        "/home/southgate/miniconda3/lib/libpython3.12.so.1.0",
        "/home/southgate/miniconda3/lib/libpython3.so",
        "/home/southgate/miniconda3/pkgs/python-3.12.9-h5148396_0/lib/libpython3.12.so",
        "/home/southgate/miniconda3/pkgs/python-3.12.9-h5148396_0/lib/libpython3.12.so.1.0",
    ];
    
    let mut found_lib = false;
    
    for path in &specific_lib_paths {
        let path = Path::new(path);
        if path.exists() {
            println!("cargo:warning=Found Python library at: {}", path.display());
            
            // Add the directory
            if let Some(dir) = path.parent() {
                println!("cargo:rustc-link-search={}", dir.display());
            }
            
            // Extract the library name without 'lib' prefix and '.so' suffix
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.starts_with("lib") && file_name.contains(".so") {
                    // Extract the base name between "lib" and ".so"
                    let mut lib_name = file_name.strip_prefix("lib").unwrap_or(file_name);
                    
                    // Remove the .so suffix and any version info (.1.0)
                    if let Some(dot_idx) = lib_name.find(".so") {
                        lib_name = &lib_name[..dot_idx];
                    }
                    
                    println!("cargo:rustc-link-lib={}", lib_name);
                    found_lib = true;
                }
            }
        }
    }
    
    // Also try to find libraries in the miniconda directory dynamically
    if !found_lib {
        let miniconda_home = PathBuf::from("/home/southgate/miniconda3");
        if miniconda_home.exists() {
            let lib_dir = miniconda_home.join("lib");
            if lib_dir.exists() {
                println!("cargo:rustc-link-search={}", lib_dir.display());
                
                // Try to find Python libraries in this directory
                if let Ok(entries) = fs::read_dir(&lib_dir) {
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with("libpython3") && name.ends_with(".so") {
                                println!("cargo:warning=Found dynamic Python library: {}", path.display());
                                
                                // Extract the library name without 'lib' prefix and '.so' suffix
                                if let Some(stripped) = name.strip_prefix("lib") {
                                    let lib_name = if let Some(dot_idx) = stripped.find(".so") {
                                        &stripped[..dot_idx]
                                    } else {
                                        stripped
                                    };
                                    
                                    println!("cargo:rustc-link-lib={}", lib_name);
                                    found_lib = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // If we found specific libraries, don't use the generic link approach in the main function
    if found_lib {
        println!("cargo:warning=Using specific Python library found in Miniconda");
    }
}

/// Main function that generates Rust code with Python information
fn main() {
    // Tell Cargo when to rerun the build script
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PYTHON_EXECUTABLE");
    println!("cargo:rerun-if-env-changed=PYTHONHOME");
    println!("cargo:rerun-if-env-changed=VIRTUAL_ENV");
    println!("cargo:rerun-if-env-changed=CONDA_PREFIX");
    
    // Detect Python information
    let python_executable = detect_python_executable();
    let python_path = PathBuf::from(&python_executable);
    let python_version = match get_python_version(&python_path) {
        Some(ver) => ver,
        None => "3".to_string()
    };
    let python_sys_prefix = get_python_sys_prefix(&python_path);
    let library_dirs = find_library_dirs(&python_path, &python_version);
    
    // Output library search paths for the linker
    for dir in &library_dirs {
        println!("cargo:rustc-link-search={}", dir);
    }
    
    // Try specific library files first - this is the most reliable approach
    add_specific_library_links();
    
    // Add generic link to Python library (fallback)
    // When linking on Linux, we need to preserve dots in the version numbers
    // For example, for Python 3.12, we need both python3.12 and python3
    println!("cargo:rustc-link-lib=python{}", python_version);
    println!("cargo:rustc-link-lib=python3");
    
    // Generate Rust code with the Python information
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("python_lib_info.rs");
    let mut f = File::create(dest_path).unwrap();
    
    // Write the Python information to the generated file
    writeln!(&mut f, "/// The Python executable path").unwrap();
    writeln!(&mut f, "pub fn get_python_executable() -> String {{").unwrap();
    writeln!(&mut f, "    \"{}\".to_string()", python_executable.replace('\\', "\\\\")).unwrap();
    writeln!(&mut f, "}}").unwrap();
    
    writeln!(&mut f, "\n/// The Python version").unwrap();
    writeln!(&mut f, "pub fn get_python_version() -> String {{").unwrap();
    writeln!(&mut f, "    \"{}\".to_string()", python_version).unwrap();
    writeln!(&mut f, "}}").unwrap();
    
    writeln!(&mut f, "\n/// The Python sys.prefix directory").unwrap();
    writeln!(&mut f, "pub fn get_python_sys_prefix() -> String {{").unwrap();
    writeln!(&mut f, "    \"{}\".to_string()", python_sys_prefix.replace('\\', "\\\\")).unwrap();
    writeln!(&mut f, "}}").unwrap();
    
    writeln!(&mut f, "\n/// The Python library directories").unwrap();
    writeln!(&mut f, "pub fn get_python_library_dirs() -> Vec<String> {{").unwrap();
    writeln!(&mut f, "    vec![").unwrap();
    for dir in &library_dirs {
        writeln!(&mut f, "        \"{}\".to_string(),", dir.replace('\\', "\\\\")).unwrap();
    }
    writeln!(&mut f, "    ]").unwrap();
    writeln!(&mut f, "}}").unwrap();
}

/// Detects the Python executable to use
fn detect_python_executable() -> String {
    // Check if PYTHON_EXECUTABLE environment variable is set
    if let Ok(python_path) = env::var("PYTHON_EXECUTABLE") {
        if !python_path.is_empty() {
            return python_path;
        }
    }
    
    // Find Python executable through PATH
    if let Some(python_path) = find_python_executable() {
        return python_path.to_string_lossy().to_string();
    }
    
    // Default to python3 if nothing else works
    "python3".to_string()
}

/// Get Python sys.prefix from the executable
fn get_python_sys_prefix(python_exe: &Path) -> String {
    Command::new(python_exe)
        .args(["-c", "import sys; print(sys.prefix)"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            // Default value if we can't get it from Python
            if cfg!(target_os = "windows") {
                "C:\\Python3".to_string()
            } else {
                "/usr".to_string()
            }
        })
}

/// Find all potential library directories for Python
fn find_library_dirs(python_exe: &Path, python_version: &str) -> Vec<String> {
    let mut lib_dirs = Vec::new();
    
    // Get sys.prefix
    let sys_prefix = get_python_sys_prefix(python_exe);
    let prefix_path = PathBuf::from(&sys_prefix);
    
    // Add standard library directories based on platform
    if cfg!(target_os = "windows") {
        // Windows
        lib_dirs.push(format!("{}\\libs", sys_prefix));
        lib_dirs.push(format!("{}\\bin", sys_prefix));
        lib_dirs.push(sys_prefix.clone());
    } else if cfg!(target_os = "macos") {
        // macOS
        lib_dirs.push(format!("{}/lib", sys_prefix));
        lib_dirs.push(format!("{}/lib/python{}", sys_prefix, python_version));
        lib_dirs.push(format!("{}/lib/python{}/lib-dynload", sys_prefix, python_version));
        lib_dirs.push(sys_prefix.clone());
        
        // Add framework locations
        let framework_paths = [
            "/Library/Frameworks/Python.framework/Versions/Current/lib",
            "/System/Library/Frameworks/Python.framework/Versions/Current/lib",
        ];
        for path in &framework_paths {
            lib_dirs.push(path.to_string());
        }
    } else {
        // Linux
        lib_dirs.push(format!("{}/lib", sys_prefix));
        lib_dirs.push(format!("{}/lib64", sys_prefix));
        lib_dirs.push(format!("{}/lib/python{}", sys_prefix, python_version));
        lib_dirs.push(format!("{}/lib64/python{}", sys_prefix, python_version));
        lib_dirs.push(format!("{}/lib/python{}/lib-dynload", sys_prefix, python_version));
        lib_dirs.push(sys_prefix.clone());
        
        // Add standard system library paths
        let system_lib_paths = [
            "/usr/lib",
            "/usr/lib64",
            "/usr/local/lib",
            "/usr/local/lib64",
        ];
        for path in &system_lib_paths {
            lib_dirs.push(path.to_string());
        }
        
        // Add known miniconda paths (directly found on this system)
        let miniconda_paths = [
            "/home/southgate/miniconda3/lib",
            "/home/southgate/miniconda3/pkgs/python-3.12.9-h5148396_0/lib",
        ];
        for path in &miniconda_paths {
            lib_dirs.push(path.to_string());
        }
    }
    
    // Get additional library paths directly from Python
    if let Ok(output) = Command::new(python_exe)
        .args(["-c", "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')"])
        .output()
    {
        if output.status.success() {
            let libdir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !libdir.is_empty() && !lib_dirs.contains(&libdir) {
                lib_dirs.push(libdir);
            }
        }
    }
    
    // Try to get conda environment lib directory
    if let Ok(conda_prefix) = env::var("CONDA_PREFIX") {
        let conda_lib = format!("{}/lib", conda_prefix);
        if !lib_dirs.contains(&conda_lib) {
            lib_dirs.push(conda_lib);
        }
    }
    
    // Filter out duplicates and non-existent directories
    let mut result = Vec::new();
    for dir in lib_dirs {
        if !result.contains(&dir) && PathBuf::from(&dir).exists() {
            result.push(dir);
        }
    }
    
    result
} 