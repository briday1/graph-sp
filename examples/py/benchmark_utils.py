"""Benchmark utilities for Python examples.

Provides timing and memory tracking capabilities using time.perf_counter()
and tracemalloc for memory allocation statistics.
"""

import time
import tracemalloc
from typing import Optional, Dict, Any


class BenchmarkResult:
    """Result of a benchmark measurement."""
    
    def __init__(self, duration_ms: float, memory_info: Dict[str, Any]):
        self.duration_ms = duration_ms
        self.memory_info = memory_info
    
    def __repr__(self):
        return f"BenchmarkResult(duration_ms={self.duration_ms:.3f}, memory={self.memory_info})"


class Benchmark:
    """Context manager for benchmarking code execution."""
    
    def __init__(self, label: str = "Benchmark"):
        self.label = label
        self.start_time: Optional[float] = None
        self.start_memory: Optional[tuple] = None
        self.result: Optional[BenchmarkResult] = None
    
    def __enter__(self):
        """Start timing and memory tracking."""
        tracemalloc.start()
        self.start_time = time.perf_counter()
        self.start_memory = tracemalloc.get_traced_memory()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Stop timing and memory tracking."""
        end_time = time.perf_counter()
        current_memory, peak_memory = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        
        duration_ms = (end_time - self.start_time) * 1000
        
        # Calculate memory delta
        start_current, start_peak = self.start_memory
        current_delta = current_memory - start_current
        peak_delta = peak_memory - start_peak
        
        self.result = BenchmarkResult(
            duration_ms=duration_ms,
            memory_info={
                'current_kb': current_delta / 1024,
                'peak_kb': peak_delta / 1024,
            }
        )
        return False
    
    def print_result(self):
        """Print the benchmark result."""
        if self.result:
            print(f"⏱️  Runtime: {self.result.duration_ms:.3f}ms")
            print(f"💾 Memory: Current: {self.result.memory_info['current_kb']:.2f} KB, "
                  f"Peak: {self.result.memory_info['peak_kb']:.2f} KB")


def print_header(title: str):
    """Print a formatted header."""
    print(f"\n{'═' * 60}")
    print(f"  {title}")
    print(f"{'═' * 60}\n")


def print_section(title: str):
    """Print a formatted section."""
    print(f"\n{'─' * 60}")
    print(title)
    print(f"{'─' * 60}\n")
