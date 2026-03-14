// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! GPU Detection and Management
#![allow(dead_code)] // Hardware detection API used at runtime
//!
//! Provides GPU detection and VRAM tracking for AI model management.
//! Maintains primal self-knowledge - only reports on THIS instance's GPU.

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// GPU information for a single GPU
///
/// Supports ANY GPU hardware from modern RTX to legacy Tesla cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU model name (e.g., "RTX 3090", "Tesla K80")
    pub model: String,
    /// Total VRAM in GB
    pub vram_total_gb: u32,
    /// Available VRAM in GB
    pub vram_available_gb: u32,
    /// GPU index (for multi-GPU systems)
    pub index: u32,
    /// GPU vendor (NVIDIA, AMD, Intel, etc.)
    pub vendor: String,

    // Performance characteristics (for heterogeneous hardware optimization)
    /// Compute capability (e.g., "3.5" for K80, "8.6" for RTX 3090)
    pub compute_capability: Option<String>,
    /// Architecture generation (e.g., "Kepler", "Ampere")
    pub architecture: Option<String>,
    /// Memory bandwidth in GB/s
    pub memory_bandwidth_gb_s: Option<u32>,
    /// Estimated AI inference performance (tokens/sec for reference model)
    pub estimated_tokens_per_sec: Option<f32>,
    /// Power draw in watts
    pub power_draw_watts: Option<u32>,
    /// Efficiency (tokens/sec/watt)
    pub efficiency_tokens_per_watt: Option<f32>,
}

/// GPU capabilities of THIS instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalGpuCapabilities {
    /// List of GPUs on THIS machine
    pub gpus: Vec<GpuInfo>,
    /// Total VRAM across all GPUs on THIS machine (GB)
    pub total_vram_gb: u32,
    /// GPU-accelerated AI supported
    pub ai_acceleration: bool,
}

/// Detect GPUs on THIS instance
///
/// Uses NVML for NVIDIA GPUs, `ROCm` for AMD GPUs, and system detection for others.
/// Supports heterogeneous hardware with multiple vendors.
///
/// # Primal Self-Knowledge
///
/// This function ONLY detects GPUs on the current machine.
/// It has NO knowledge of other instances or their GPUs.
///
/// # Supported Hardware
///
/// - **NVIDIA**: RTX 5090, 4090, 3090, 3080, 3070, 2070, Tesla K80, P100, V100, A100, etc.
/// - **AMD**: Radeon RX 7900/6900/5700 series, MI100/MI200 series, etc.
/// - **Intel**: Arc series (future)
/// - **Neuromorphic**: Akida brainchips (future)
pub async fn detect_local_gpus() -> Result<Option<LocalGpuCapabilities>, PrimalError> {
    debug!("Detecting local GPU capabilities (heterogeneous hardware support)...");

    // Try all vendors concurrently for fastest detection
    let (nvidia_result, amd_result) = tokio::join!(detect_nvidia_gpus(), detect_amd_gpus());

    // Collect all detected GPUs
    let mut all_gpus = Vec::new();
    let mut total_vram = 0u32;

    // Add NVIDIA GPUs
    if let Ok(Some(nvidia_caps)) = nvidia_result {
        info!(
            "Detected {} NVIDIA GPU(s), total VRAM: {}GB",
            nvidia_caps.gpus.len(),
            nvidia_caps.total_vram_gb
        );
        total_vram += nvidia_caps.total_vram_gb;
        all_gpus.extend(nvidia_caps.gpus);
    }

    // Add AMD GPUs
    if let Ok(Some(amd_caps)) = amd_result {
        info!(
            "Detected {} AMD GPU(s), total VRAM: {}GB",
            amd_caps.gpus.len(),
            amd_caps.total_vram_gb
        );
        total_vram += amd_caps.total_vram_gb;
        all_gpus.extend(amd_caps.gpus);
    }

    // Future GPU vendor support:
    // - Metal (Apple Silicon): macOS GPU acceleration
    // - Intel Arc: Intel discrete GPUs
    // - Akida brainchip: Neuromorphic AI accelerators

    if all_gpus.is_empty() {
        debug!("No GPUs detected on this instance");
        return Ok(None);
    }

    info!(
        "Total heterogeneous GPU setup: {} device(s), {}GB total VRAM",
        all_gpus.len(),
        total_vram
    );

    Ok(Some(LocalGpuCapabilities {
        gpus: all_gpus,
        total_vram_gb: total_vram,
        ai_acceleration: true,
    }))
}

/// Detect NVIDIA GPUs using NVML
async fn detect_nvidia_gpus() -> Result<Option<LocalGpuCapabilities>, PrimalError> {
    // Try to use nvml crate if available
    #[cfg(feature = "nvml")]
    {
        use nvml_wrapper::Nvml;

        match Nvml::init() {
            Ok(nvml) => {
                let device_count = nvml.device_count().unwrap_or(0);
                if device_count == 0 {
                    return Ok(None);
                }

                let mut gpus = Vec::new();
                let mut total_vram = 0u64;

                for i in 0..device_count {
                    if let Ok(device) = nvml.device_by_index(i) {
                        if let Ok(name) = device.name() {
                            if let Ok(memory_info) = device.memory_info() {
                                let vram_total_gb =
                                    (memory_info.total / (1024 * 1024 * 1024)) as u32;
                                let vram_free_gb = (memory_info.free / (1024 * 1024 * 1024)) as u32;

                                total_vram += vram_total_gb as u64;

                                // Get additional GPU info for performance prediction
                                let compute_capability = device
                                    .cuda_compute_capability()
                                    .ok()
                                    .map(|cc| format!("{}.{}", cc.major, cc.minor));

                                let architecture = compute_capability
                                    .as_ref()
                                    .and_then(|cc| architecture_from_compute_capability(cc));

                                let power_draw =
                                    device.power_usage().ok().map(|p| (p / 1000) as u32); // Convert mW to W

                                let memory_bandwidth = device.memory_info().ok().and_then(|_| {
                                    // Estimate bandwidth from model if not directly available
                                    estimate_bandwidth(&name)
                                });

                                // Estimate performance based on GPU model and architecture
                                let estimated_tokens_per_sec =
                                    estimate_performance(&name, &architecture);

                                let efficiency = estimated_tokens_per_sec
                                    .zip(power_draw)
                                    .map(|(tokens, watts)| tokens / watts as f32);

                                gpus.push(GpuInfo {
                                    model: name,
                                    vram_total_gb,
                                    vram_available_gb: vram_free_gb,
                                    index: i,
                                    vendor: "NVIDIA".to_string(),
                                    compute_capability,
                                    architecture,
                                    memory_bandwidth_gb_s: memory_bandwidth,
                                    estimated_tokens_per_sec,
                                    power_draw_watts: power_draw,
                                    efficiency_tokens_per_watt: efficiency,
                                });
                            }
                        }
                    }
                }

                if !gpus.is_empty() {
                    return Ok(Some(LocalGpuCapabilities {
                        gpus,
                        total_vram_gb: total_vram as u32,
                        ai_acceleration: true,
                    }));
                }
            }
            Err(e) => {
                debug!("NVML init failed: {:?}", e);
            }
        }
    }

    // Fallback: Try to detect via system calls
    detect_nvidia_fallback().await
}

/// Map compute capability to architecture name
fn architecture_from_compute_capability(cc: &str) -> Option<String> {
    let major: u32 = cc.split('.').next()?.parse().ok()?;

    Some(match major {
        9 => "Hopper".to_string(),       // H100
        8 => "Ampere".to_string(),       // RTX 30/40 series, A100
        7 => "Volta/Turing".to_string(), // V100, RTX 20 series
        6 => "Pascal".to_string(),       // GTX 10 series, P100
        5 => "Maxwell".to_string(),      // GTX 9 series
        3 => "Kepler".to_string(),       // K series, GTX 7 series
        2 => "Fermi".to_string(),        // Very old
        _ => format!("Unknown (CC {cc})"),
    })
}

/// Estimate memory bandwidth from GPU model
fn estimate_bandwidth(model: &str) -> Option<u32> {
    // Bandwidth in GB/s (approximate)
    let bandwidth = if model.contains("RTX 5090") {
        1792 // GDDR7
    } else if model.contains("RTX 4090") {
        1008
    } else if model.contains("RTX 3090") {
        936
    } else if model.contains("RTX 3080") {
        760
    } else if model.contains("RTX 3070") || model.contains("RTX 2070") || model.contains("RX 5700")
    {
        448
    } else if model.contains("V100") {
        900
    } else if model.contains("A100") {
        1555
    } else if model.contains("K80") {
        240 // Old but still usable!
    } else if model.contains("M40") {
        288
    } else if model.contains("GTX 1080") {
        320
    } else if model.contains("GTX 1060") {
        192
    } else if model.contains("RX 7900") {
        960 // AMD
    } else if model.contains("RX 6900") {
        512
    } else {
        return None; // Unknown
    };

    Some(bandwidth)
}

/// Estimate AI inference performance from GPU model
fn estimate_performance(model: &str, architecture: &Option<String>) -> Option<f32> {
    // Tokens per second for reference 7B model (rough estimates)
    let tokens_per_sec = if model.contains("RTX 5090") {
        150.0 // Cutting edge
    } else if model.contains("RTX 4090") {
        120.0
    } else if model.contains("RTX 3090") {
        80.0
    } else if model.contains("RTX 3080") {
        65.0
    } else if model.contains("RTX 3070") {
        45.0
    } else if model.contains("RTX 2070") {
        30.0
    } else if model.contains("A100") {
        200.0 // Data center beast
    } else if model.contains("V100") {
        90.0
    } else if model.contains("K80") {
        8.0 // Old but works!
    } else if model.contains("M40") {
        15.0
    } else if model.contains("RX 7900") {
        70.0 // AMD RDNA3
    } else if model.contains("RX 6900") {
        55.0
    } else if model.contains("RX 5700") {
        35.0
    } else if model.contains("MI200") {
        180.0 // AMD datacenter
    } else if model.contains("MI100") {
        120.0
    } else {
        // Estimate from architecture if available
        if let Some(arch) = architecture {
            match arch.as_str() {
                "Hopper" => 180.0,
                "Ampere" => 75.0,
                "Volta/Turing" => 50.0,
                "Pascal" => 25.0,
                "RDNA3" => 65.0,
                "RDNA2" => 50.0,
                _ => return None,
            }
        } else {
            return None;
        }
    };

    Some(tokens_per_sec)
}

/// Estimate power draw from GPU model
fn estimate_power(model: &str) -> Option<u32> {
    // Power in watts (TDP) — different GPU models can share TDP values
    let power = if model.contains("RTX 5090") {
        575 // Power hungry beast
    } else if model.contains("RTX 4090") {
        450
    } else if model.contains("RTX 3090") {
        350
    } else if model.contains("RTX 3080") {
        320
    } else if model.contains("RTX 3070") {
        220
    } else if model.contains("RTX 2070") {
        215
    } else if model.contains("A100") {
        400
    } else if model.contains("V100")
        || model.contains("K80")
        || model.contains("RX 6900")
        || model.contains("MI100")
    {
        300
    } else if model.contains("RX 7900") {
        355
    } else if model.contains("RX 5700") {
        225
    } else if model.contains("MI200") {
        560
    } else {
        return None;
    };

    Some(power)
}

/// Fallback NVIDIA detection using nvidia-smi
async fn detect_nvidia_fallback() -> Result<Option<LocalGpuCapabilities>, PrimalError> {
    debug!("Trying nvidia-smi fallback detection...");

    match tokio::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,memory.free",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut gpus = Vec::new();
            let mut total_vram = 0u32;

            for (index, line) in stdout.lines().enumerate() {
                let parts: Vec<&str> = line.split(',').map(str::trim).collect();
                if parts.len() >= 3 {
                    if let (Ok(total_mb), Ok(free_mb)) =
                        (parts[1].parse::<f64>(), parts[2].parse::<f64>())
                    {
                        let vram_total_gb = (total_mb / 1024.0).ceil() as u32;
                        let vram_free_gb = (free_mb / 1024.0).ceil() as u32;
                        total_vram += vram_total_gb;

                        let model_name = parts[0].to_string();
                        let architecture = detect_architecture_from_name(&model_name);
                        let memory_bandwidth = estimate_bandwidth(&model_name);
                        let estimated_tokens_per_sec =
                            estimate_performance(&model_name, &architecture);
                        let power_draw = estimate_power(&model_name);
                        let efficiency = estimated_tokens_per_sec
                            .zip(power_draw)
                            .map(|(tokens, watts)| tokens / watts as f32);

                        gpus.push(GpuInfo {
                            model: model_name,
                            vram_total_gb,
                            vram_available_gb: vram_free_gb,
                            index: index as u32,
                            vendor: "NVIDIA".to_string(),
                            compute_capability: None, // Not available from nvidia-smi basic query
                            architecture,
                            memory_bandwidth_gb_s: memory_bandwidth,
                            estimated_tokens_per_sec,
                            power_draw_watts: power_draw,
                            efficiency_tokens_per_watt: efficiency,
                        });
                    }
                }
            }

            if !gpus.is_empty() {
                return Ok(Some(LocalGpuCapabilities {
                    gpus,
                    total_vram_gb: total_vram,
                    ai_acceleration: true,
                }));
            }
        }
        Ok(_) => {
            debug!("nvidia-smi returned non-zero exit code");
        }
        Err(e) => {
            debug!("nvidia-smi not available: {:?}", e);
        }
    }

    Ok(None)
}

/// Detect architecture from GPU model name
fn detect_architecture_from_name(model: &str) -> Option<String> {
    // NVIDIA architectures
    if model.contains("RTX 50") || model.contains("H100") {
        return Some("Hopper".to_string());
    }
    if model.contains("RTX 40") || model.contains("RTX 30") || model.contains("A100") {
        return Some("Ampere".to_string());
    }
    if model.contains("RTX 20") || model.contains("V100") {
        return Some("Volta/Turing".to_string());
    }
    if model.contains("GTX 10") || model.contains("P100") {
        return Some("Pascal".to_string());
    }
    if model.contains("K80") || model.contains("K40") {
        return Some("Kepler".to_string());
    }

    // AMD architectures
    if model.contains("RX 7") {
        return Some("RDNA3".to_string());
    }
    if model.contains("RX 6") {
        return Some("RDNA2".to_string());
    }
    if model.contains("RX 5") {
        return Some("RDNA".to_string());
    }
    if model.contains("MI200") || model.contains("MI100") {
        return Some("CDNA".to_string());
    }

    None
}

/// Detect AMD GPUs using `ROCm`
///
/// # Heterogeneous Hardware Support
///
/// This function detects AMD GPUs for mixed NVIDIA/AMD setups.
/// Supports both consumer Radeon and datacenter MI series.
async fn detect_amd_gpus() -> Result<Option<LocalGpuCapabilities>, PrimalError> {
    debug!("Detecting AMD GPUs via ROCm...");

    // Try rocm-smi (AMD's equivalent to nvidia-smi)
    match tokio::process::Command::new("rocm-smi")
        .args(["--showmeminfo", "vram", "--csv"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut gpus = Vec::new();
            let mut total_vram = 0u32;

            // Parse CSV output
            for (index, line) in stdout.lines().enumerate().skip(1) {
                // Skip header
                let parts: Vec<&str> = line.split(',').map(str::trim).collect();
                if parts.len() >= 2 {
                    // Try to get GPU name
                    if let Ok(name_output) = tokio::process::Command::new("rocm-smi")
                        .args(["--showproductname", "--device", &index.to_string()])
                        .output()
                        .await
                    {
                        if name_output.status.success() {
                            let name_stdout = String::from_utf8_lossy(&name_output.stdout);
                            let model_name = name_stdout
                                .lines()
                                .find(|l| !l.contains("GPU") && !l.is_empty())
                                .unwrap_or("AMD GPU")
                                .trim()
                                .to_string();

                            // Parse VRAM from first output
                            if let Some(vram_str) = parts.get(1) {
                                if let Ok(vram_mb) =
                                    vram_str.replace("MB", "").trim().parse::<u32>()
                                {
                                    let vram_total_gb = vram_mb.div_ceil(1024); // Round up

                                    // Estimate free VRAM (assume 90% available if idle)
                                    let vram_free_gb = (vram_total_gb as f32 * 0.9) as u32;

                                    total_vram += vram_total_gb;

                                    let architecture = detect_architecture_from_name(&model_name);
                                    let memory_bandwidth = estimate_bandwidth(&model_name);
                                    let estimated_tokens_per_sec =
                                        estimate_performance(&model_name, &architecture);
                                    let power_draw = estimate_power(&model_name);
                                    let efficiency = estimated_tokens_per_sec
                                        .zip(power_draw)
                                        .map(|(tokens, watts)| tokens / watts as f32);

                                    gpus.push(GpuInfo {
                                        model: model_name,
                                        vram_total_gb,
                                        vram_available_gb: vram_free_gb,
                                        index: index as u32,
                                        vendor: "AMD".to_string(),
                                        compute_capability: None, // AMD uses gfx architecture version
                                        architecture,
                                        memory_bandwidth_gb_s: memory_bandwidth,
                                        estimated_tokens_per_sec,
                                        power_draw_watts: power_draw,
                                        efficiency_tokens_per_watt: efficiency,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if !gpus.is_empty() {
                info!("Detected {} AMD GPU(s) via rocm-smi", gpus.len());
                return Ok(Some(LocalGpuCapabilities {
                    gpus,
                    total_vram_gb: total_vram,
                    ai_acceleration: true,
                }));
            }
        }
        Ok(_) => {
            debug!("rocm-smi returned non-zero exit code or no AMD GPUs found");
        }
        Err(e) => {
            debug!("rocm-smi not available: {:?}", e);
        }
    }

    // Fallback: Try lspci for AMD GPUs
    detect_amd_fallback().await
}

/// Fallback AMD detection using lspci
async fn detect_amd_fallback() -> Result<Option<LocalGpuCapabilities>, PrimalError> {
    debug!("Trying lspci fallback for AMD GPUs...");

    match tokio::process::Command::new("lspci").output().await {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut gpus = Vec::new();
            let mut amd_index = 0u32;

            for line in stdout.lines() {
                if line.contains("VGA")
                    && (line.contains("AMD") || line.contains("ATI") || line.contains("Radeon"))
                {
                    // Extract GPU model name
                    let model_name = if let Some(bracket_start) = line.find('[') {
                        if let Some(bracket_end) = line[bracket_start..].find(']') {
                            line[bracket_start + 1..bracket_start + bracket_end].to_string()
                        } else {
                            "AMD GPU".to_string()
                        }
                    } else {
                        "AMD GPU".to_string()
                    };

                    // Estimate VRAM based on model (conservative estimates)
                    let vram_total_gb = if model_name.contains("7900") {
                        20 // RX 7900 XTX
                    } else if model_name.contains("6900") {
                        16
                    } else if model_name.contains("5700") {
                        8
                    } else if model_name.contains("MI200") {
                        64 // Datacenter
                    } else if model_name.contains("MI100") {
                        32
                    } else {
                        8 // Conservative default
                    };

                    let vram_free_gb = (vram_total_gb as f32 * 0.9) as u32;
                    let architecture = detect_architecture_from_name(&model_name);
                    let memory_bandwidth = estimate_bandwidth(&model_name);
                    let estimated_tokens_per_sec = estimate_performance(&model_name, &architecture);
                    let power_draw = estimate_power(&model_name);
                    let efficiency = estimated_tokens_per_sec
                        .zip(power_draw)
                        .map(|(tokens, watts)| tokens / watts as f32);

                    gpus.push(GpuInfo {
                        model: model_name,
                        vram_total_gb,
                        vram_available_gb: vram_free_gb,
                        index: amd_index,
                        vendor: "AMD".to_string(),
                        compute_capability: None,
                        architecture,
                        memory_bandwidth_gb_s: memory_bandwidth,
                        estimated_tokens_per_sec,
                        power_draw_watts: power_draw,
                        efficiency_tokens_per_watt: efficiency,
                    });

                    amd_index += 1;
                }
            }

            if !gpus.is_empty() {
                info!("Detected {} AMD GPU(s) via lspci fallback", gpus.len());
                let total_vram = gpus.iter().map(|g| g.vram_total_gb).sum();
                return Ok(Some(LocalGpuCapabilities {
                    gpus,
                    total_vram_gb: total_vram,
                    ai_acceleration: true,
                }));
            }
        }
        Ok(_) => {
            debug!("lspci returned non-zero exit code");
        }
        Err(e) => {
            debug!("lspci not available: {:?}", e);
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_detection_doesnt_panic() {
        // Should not panic even if no GPUs present
        let result = detect_local_gpus().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_nvidia_detection() {
        // Should not panic even if NVIDIA drivers not present
        let result = detect_nvidia_gpus().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_amd_detection() {
        // Should not panic even if AMD drivers not present
        let result = detect_amd_gpus().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_architecture_detection() {
        assert_eq!(
            detect_architecture_from_name("RTX 3090"),
            Some("Ampere".to_string())
        );
        assert_eq!(
            detect_architecture_from_name("RX 7900"),
            Some("RDNA3".to_string())
        );
        assert_eq!(
            detect_architecture_from_name("K80"),
            Some("Kepler".to_string())
        );
    }

    #[test]
    fn test_bandwidth_estimation() {
        assert_eq!(estimate_bandwidth("RTX 3090"), Some(936));
        assert_eq!(estimate_bandwidth("RX 7900"), Some(960));
        assert_eq!(estimate_bandwidth("K80"), Some(240));
    }

    #[test]
    fn test_performance_estimation() {
        let arch = Some("Ampere".to_string());
        assert!(estimate_performance("RTX 3090", &arch).is_some());
        assert!(estimate_performance("RX 7900", &Some("RDNA3".to_string())).is_some());
    }

    #[test]
    fn test_architecture_from_compute_capability() {
        assert_eq!(
            architecture_from_compute_capability("8.6"),
            Some("Ampere".to_string())
        );
        assert_eq!(
            architecture_from_compute_capability("9.0"),
            Some("Hopper".to_string())
        );
        assert_eq!(
            architecture_from_compute_capability("7.5"),
            Some("Volta/Turing".to_string())
        );
        assert_eq!(
            architecture_from_compute_capability("6.1"),
            Some("Pascal".to_string())
        );
        assert_eq!(
            architecture_from_compute_capability("3.7"),
            Some("Kepler".to_string())
        );
    }

    #[test]
    fn test_estimate_power() {
        assert_eq!(estimate_power("RTX 4090"), Some(450));
        assert_eq!(estimate_power("RTX 3090"), Some(350));
        assert_eq!(estimate_power("A100"), Some(400));
        assert_eq!(estimate_power("RX 7900"), Some(355));
        assert_eq!(estimate_power("unknown"), None);
    }

    #[test]
    fn test_gpu_info_serde() {
        let gpu = GpuInfo {
            model: "RTX 3090".to_string(),
            vram_total_gb: 24,
            vram_available_gb: 20,
            index: 0,
            vendor: "NVIDIA".to_string(),
            compute_capability: Some("8.6".to_string()),
            architecture: Some("Ampere".to_string()),
            memory_bandwidth_gb_s: Some(936),
            estimated_tokens_per_sec: Some(80.0),
            power_draw_watts: Some(350),
            efficiency_tokens_per_watt: Some(0.23),
        };
        let json = serde_json::to_string(&gpu).unwrap();
        let deserialized: GpuInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "RTX 3090");
        assert_eq!(deserialized.vram_total_gb, 24);
    }

    #[test]
    fn test_local_gpu_capabilities_serde() {
        let caps = LocalGpuCapabilities {
            gpus: vec![],
            total_vram_gb: 0,
            ai_acceleration: false,
        };
        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: LocalGpuCapabilities = serde_json::from_str(&json).unwrap();
        assert!(deserialized.gpus.is_empty());
        assert!(!deserialized.ai_acceleration);
    }
}
