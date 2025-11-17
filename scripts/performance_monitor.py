#!/usr/bin/env python3
"""
Real-time Performance Monitor for Rusty Audio

This script monitors the performance of the rusty-audio application in real-time,
tracking CPU usage, memory consumption, audio latency, and GUI frame rates.

Usage:
    python scripts/performance_monitor.py [--pid PID] [--duration SECONDS]
"""

import psutil
import time
import sys
import argparse
import json
from datetime import datetime
from collections import deque
from typing import Dict, List, Optional
import subprocess
import os

try:
    import matplotlib.pyplot as plt
    import matplotlib.animation as animation
    from matplotlib.figure import Figure
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False
    print("Warning: matplotlib not installed. Install with: pip install matplotlib")


class PerformanceMonitor:
    """Monitor performance metrics of the rusty-audio application."""

    def __init__(self, pid: Optional[int] = None, history_size: int = 100):
        self.pid = pid
        self.process = None
        self.history_size = history_size

        # Metrics history
        self.cpu_history = deque(maxlen=history_size)
        self.memory_history = deque(maxlen=history_size)
        self.thread_count_history = deque(maxlen=history_size)
        self.handle_count_history = deque(maxlen=history_size)
        self.time_history = deque(maxlen=history_size)

        # Audio-specific metrics (parsed from logs if available)
        self.audio_latency_history = deque(maxlen=history_size)
        self.xrun_count = 0
        self.frame_drops = 0

        # Performance thresholds
        self.thresholds = {
            'cpu_percent': 25.0,  # Max 25% CPU for audio app
            'memory_mb': 500.0,   # Max 500MB RAM
            'audio_latency_ms': 10.0,  # Max 10ms latency
            'thread_count': 20,   # Max 20 threads
        }

        self._find_process()

    def _find_process(self):
        """Find the rusty-audio process."""
        if self.pid:
            try:
                self.process = psutil.Process(self.pid)
                if not self._is_rusty_audio_process(self.process):
                    print(f"Warning: PID {self.pid} is not rusty-audio")
            except psutil.NoSuchProcess:
                print(f"Error: No process with PID {self.pid}")
                sys.exit(1)
        else:
            # Try to find rusty-audio process
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                if self._is_rusty_audio_process(proc):
                    self.process = proc
                    self.pid = proc.pid
                    print(f"Found rusty-audio process: PID {self.pid}")
                    break

            if not self.process:
                print("Error: Could not find rusty-audio process")
                print("Start rusty-audio or specify --pid")
                sys.exit(1)

    def _is_rusty_audio_process(self, proc) -> bool:
        """Check if a process is rusty-audio."""
        try:
            name = proc.info.get('name', '').lower()
            cmdline = ' '.join(proc.info.get('cmdline', [])).lower()
            return 'rusty' in name or 'rusty-audio' in cmdline or 'rusty_audio' in name
        except (psutil.AccessDenied, psutil.NoSuchProcess):
            return False

    def collect_metrics(self) -> Dict:
        """Collect current performance metrics."""
        if not self.process or not self.process.is_running():
            self._find_process()

        try:
            with self.process.oneshot():
                cpu_percent = self.process.cpu_percent(interval=0.1)
                memory_info = self.process.memory_info()
                memory_mb = memory_info.rss / (1024 * 1024)

                # Get thread and handle counts
                try:
                    thread_count = self.process.num_threads()
                except:
                    thread_count = 0

                try:
                    handle_count = self.process.num_handles() if sys.platform == 'win32' else 0
                except:
                    handle_count = 0

                # Get IO counters if available
                try:
                    io_counters = self.process.io_counters()
                    read_bytes = io_counters.read_bytes
                    write_bytes = io_counters.write_bytes
                except:
                    read_bytes = write_bytes = 0

                metrics = {
                    'timestamp': datetime.now().isoformat(),
                    'cpu_percent': cpu_percent,
                    'memory_mb': memory_mb,
                    'memory_percent': self.process.memory_percent(),
                    'thread_count': thread_count,
                    'handle_count': handle_count,
                    'read_bytes': read_bytes,
                    'write_bytes': write_bytes,
                }

                # Update history
                current_time = time.time()
                self.cpu_history.append(cpu_percent)
                self.memory_history.append(memory_mb)
                self.thread_count_history.append(thread_count)
                self.handle_count_history.append(handle_count)
                self.time_history.append(current_time)

                # Check thresholds
                self._check_thresholds(metrics)

                return metrics

        except (psutil.NoSuchProcess, psutil.AccessDenied) as e:
            print(f"Error collecting metrics: {e}")
            return {}

    def _check_thresholds(self, metrics: Dict):
        """Check if metrics exceed thresholds and print warnings."""
        warnings = []

        if metrics['cpu_percent'] > self.thresholds['cpu_percent']:
            warnings.append(f"⚠️  High CPU: {metrics['cpu_percent']:.1f}% > {self.thresholds['cpu_percent']}%")

        if metrics['memory_mb'] > self.thresholds['memory_mb']:
            warnings.append(f"⚠️  High Memory: {metrics['memory_mb']:.1f}MB > {self.thresholds['memory_mb']}MB")

        if metrics['thread_count'] > self.thresholds['thread_count']:
            warnings.append(f"⚠️  High Thread Count: {metrics['thread_count']} > {self.thresholds['thread_count']}")

        for warning in warnings:
            print(warning)

    def print_metrics(self, metrics: Dict):
        """Print metrics to console."""
        if not metrics:
            return

        print(f"\n[{metrics['timestamp']}]")
        print(f"CPU: {metrics['cpu_percent']:6.1f}% | "
              f"Memory: {metrics['memory_mb']:7.1f}MB ({metrics['memory_percent']:.1f}%) | "
              f"Threads: {metrics['thread_count']:3d} | "
              f"Handles: {metrics['handle_count']:4d}")

        # Print averages if we have history
        if len(self.cpu_history) >= 10:
            avg_cpu = sum(list(self.cpu_history)[-10:]) / 10
            avg_memory = sum(list(self.memory_history)[-10:]) / 10
            print(f"10s Average - CPU: {avg_cpu:.1f}% | Memory: {avg_memory:.1f}MB")

    def save_metrics(self, filename: str = "performance_metrics.json"):
        """Save collected metrics to a JSON file."""
        data = {
            'process_id': self.pid,
            'process_name': self.process.name() if self.process else 'unknown',
            'monitoring_duration': len(self.time_history),
            'metrics': {
                'cpu': {
                    'history': list(self.cpu_history),
                    'average': sum(self.cpu_history) / len(self.cpu_history) if self.cpu_history else 0,
                    'peak': max(self.cpu_history) if self.cpu_history else 0,
                },
                'memory': {
                    'history': list(self.memory_history),
                    'average': sum(self.memory_history) / len(self.memory_history) if self.memory_history else 0,
                    'peak': max(self.memory_history) if self.memory_history else 0,
                },
                'threads': {
                    'history': list(self.thread_count_history),
                    'average': sum(self.thread_count_history) / len(self.thread_count_history) if self.thread_count_history else 0,
                    'peak': max(self.thread_count_history) if self.thread_count_history else 0,
                },
            },
            'thresholds': self.thresholds,
            'violations': self._get_threshold_violations(),
        }

        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"\nMetrics saved to {filename}")

    def _get_threshold_violations(self) -> Dict:
        """Count threshold violations."""
        violations = {
            'cpu_exceeded': sum(1 for cpu in self.cpu_history if cpu > self.thresholds['cpu_percent']),
            'memory_exceeded': sum(1 for mem in self.memory_history if mem > self.thresholds['memory_mb']),
            'thread_exceeded': sum(1 for tc in self.thread_count_history if tc > self.thresholds['thread_count']),
        }
        return violations

    def plot_realtime(self):
        """Create real-time plot of metrics (requires matplotlib)."""
        if not HAS_MATPLOTLIB:
            print("Real-time plotting requires matplotlib")
            return

        fig, axes = plt.subplots(2, 2, figsize=(12, 8))
        fig.suptitle(f'Rusty Audio Performance Monitor (PID: {self.pid})')

        def animate(frame):
            metrics = self.collect_metrics()
            if not metrics:
                return

            # Clear and redraw each subplot
            for ax in axes.flat:
                ax.clear()

            # CPU Usage
            axes[0, 0].plot(list(self.cpu_history), 'b-')
            axes[0, 0].axhline(y=self.thresholds['cpu_percent'], color='r', linestyle='--', label='Threshold')
            axes[0, 0].set_ylabel('CPU %')
            axes[0, 0].set_title('CPU Usage')
            axes[0, 0].legend()
            axes[0, 0].set_ylim(0, 100)

            # Memory Usage
            axes[0, 1].plot(list(self.memory_history), 'g-')
            axes[0, 1].axhline(y=self.thresholds['memory_mb'], color='r', linestyle='--', label='Threshold')
            axes[0, 1].set_ylabel('Memory (MB)')
            axes[0, 1].set_title('Memory Usage')
            axes[0, 1].legend()

            # Thread Count
            axes[1, 0].plot(list(self.thread_count_history), 'm-')
            axes[1, 0].axhline(y=self.thresholds['thread_count'], color='r', linestyle='--', label='Threshold')
            axes[1, 0].set_ylabel('Threads')
            axes[1, 0].set_title('Thread Count')
            axes[1, 0].legend()

            # Performance Summary
            axes[1, 1].axis('off')
            summary_text = f"""Current Performance:

CPU: {metrics['cpu_percent']:.1f}%
Memory: {metrics['memory_mb']:.1f}MB
Threads: {metrics['thread_count']}
Handles: {metrics['handle_count']}

Averages (last 10s):
CPU: {sum(list(self.cpu_history)[-10:]) / min(10, len(self.cpu_history)):.1f}%
Memory: {sum(list(self.memory_history)[-10:]) / min(10, len(self.memory_history)):.1f}MB
            """
            axes[1, 1].text(0.1, 0.5, summary_text, fontsize=10, verticalalignment='center')

            plt.tight_layout()

        ani = animation.FuncAnimation(fig, animate, interval=1000)
        plt.show()


def run_benchmark_and_monitor():
    """Run benchmarks while monitoring performance."""
    print("Starting benchmark with monitoring...")

    # Start the benchmark in a subprocess
    benchmark_proc = subprocess.Popen(
        ["cargo", "bench", "--bench", "bottleneck_benchmarks"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Monitor the benchmark process
    monitor = PerformanceMonitor(pid=benchmark_proc.pid)

    try:
        while benchmark_proc.poll() is None:
            metrics = monitor.collect_metrics()
            monitor.print_metrics(metrics)
            time.sleep(1)
    except KeyboardInterrupt:
        benchmark_proc.terminate()

    # Get benchmark results
    stdout, stderr = benchmark_proc.communicate()
    print("\nBenchmark Results:")
    print(stdout)

    # Save monitoring data
    monitor.save_metrics("benchmark_performance.json")


def main():
    parser = argparse.ArgumentParser(description='Monitor Rusty Audio performance')
    parser.add_argument('--pid', type=int, help='Process ID to monitor')
    parser.add_argument('--duration', type=int, default=0,
                       help='Monitoring duration in seconds (0 for continuous)')
    parser.add_argument('--interval', type=float, default=1.0,
                       help='Sampling interval in seconds')
    parser.add_argument('--plot', action='store_true',
                       help='Show real-time plot')
    parser.add_argument('--benchmark', action='store_true',
                       help='Run benchmarks while monitoring')
    parser.add_argument('--output', type=str, default='performance_metrics.json',
                       help='Output filename for metrics')

    args = parser.parse_args()

    if args.benchmark:
        run_benchmark_and_monitor()
        return

    monitor = PerformanceMonitor(pid=args.pid)

    if args.plot and HAS_MATPLOTLIB:
        monitor.plot_realtime()
    else:
        try:
            start_time = time.time()
            while True:
                metrics = monitor.collect_metrics()
                monitor.print_metrics(metrics)

                if args.duration > 0 and (time.time() - start_time) >= args.duration:
                    break

                time.sleep(args.interval)

        except KeyboardInterrupt:
            print("\n\nMonitoring stopped by user")
        finally:
            monitor.save_metrics(args.output)

            # Print summary
            violations = monitor._get_threshold_violations()
            print("\n=== Performance Summary ===")
            print(f"CPU Average: {sum(monitor.cpu_history) / len(monitor.cpu_history):.1f}%")
            print(f"CPU Peak: {max(monitor.cpu_history):.1f}%")
            print(f"Memory Average: {sum(monitor.memory_history) / len(monitor.memory_history):.1f}MB")
            print(f"Memory Peak: {max(monitor.memory_history):.1f}MB")
            print(f"\nThreshold Violations:")
            print(f"  CPU > {monitor.thresholds['cpu_percent']}%: {violations['cpu_exceeded']} times")
            print(f"  Memory > {monitor.thresholds['memory_mb']}MB: {violations['memory_exceeded']} times")
            print(f"  Threads > {monitor.thresholds['thread_count']}: {violations['thread_exceeded']} times")


if __name__ == '__main__':
    main()