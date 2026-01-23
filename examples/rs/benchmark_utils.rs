// Benchmark utilities for examples
// Provides timing and memory tracking capabilities

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub duration_ms: f64,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub description: String,
}

impl MemoryInfo {
    pub fn best_effort() -> Self {
        // Best-effort memory tracking
        // On Linux, we could read /proc/self/status for RSS
        // For cross-platform simplicity, we provide a placeholder
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return MemoryInfo {
                                description: format!("RSS: {} kB", parts[1]),
                            };
                        }
                    }
                }
            }
        }
        
        MemoryInfo {
            description: "Memory tracking unavailable".to_string(),
        }
    }
}

pub struct Benchmark {
    start: Instant,
    label: String,
}

impl Benchmark {
    pub fn start(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }
    
    pub fn finish(self) -> BenchmarkResult {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let memory_info = MemoryInfo::best_effort();
        
        BenchmarkResult {
            duration_ms,
            memory_info,
        }
    }
    
    pub fn finish_and_print(self) -> BenchmarkResult {
        let result = self.finish();
        println!("⏱️  Runtime: {:.3}ms", result.duration_ms);
        println!("💾 Memory: {}", result.memory_info.description);
        result
    }
}

/// Format a section header
pub fn print_header(title: &str) {
    println!("\n{}", "═".repeat(60));
    println!("  {}", title);
    println!("{}\n", "═".repeat(60));
}

/// Format a subsection
pub fn print_section(title: &str) {
    println!("\n{}", "─".repeat(60));
    println!("{}", title);
    println!("{}\n", "─".repeat(60));
}
