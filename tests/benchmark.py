import os
import time
import rustwalk

SAMPLE_DIR = Path.home() / "projects"


def benchmark_walk(path: str = SAMPLE_DIR):
    print(f"Benchmarking walk on: {path}")

    start = time.time()
    results = rustwalk.walk(path)
    elapsed = time.time() - start

    print(f"Found {len(results)} entries in {elapsed:.4f} seconds")
    print(f"Speed: {len(results) / elapsed:.2f} entries/second")
    return results


def benchmark_walk_files(path: str = SAMPLE_DIR):
    print(f"\nBenchmarking walk_files on: {path}")

    start = time.time()
    results = rustwalk.walk_files(path)
    elapsed = time.time() - start

    print(f"Found {len(results)} files in {elapsed:.4f} seconds")
    print(f"Speed: {len(results) / elapsed:.2f} files/second")
    return results


def benchmark_walk_with_metadata(path: str = SAMPLE_DIR):
    print(f"\nBenchmarking walk_with_metadata on: {path}")

    start = time.time()
    results = rustwalk.walk_with_metadata(path, max_depth=5)
    elapsed = time.time() - start

    print(f"Found {len(results)} entries in {elapsed:.4f} seconds")
    print(f"Speed: {len(results) / elapsed:.2f} entries/second")

    # Show some metadata
    if results:
        entry = results[0]
        print(f"\nExample entry:")
        print(f"  Path: {entry.path}")
        print(f"  Is file: {entry.is_file}")
        print(f"  Is dir: {entry.is_dir}")
        print(f"  Depth: {entry.depth}")

    return results


def benchmark_walk_parallel(path: str = SAMPLE_DIR):
    print(f"\nBenchmarking walk_parallel on: {path}")

    start = time.time()
    results = rustwalk.walk_parallel(path, num_threads=4)
    elapsed = time.time() - start

    print(f"Found {len(results)} entries in {elapsed:.4f} seconds")
    print(f"Speed: {len(results) / elapsed:.2f} entries/second")
    return results


def compare_with_os_walk(path: str = SAMPLE_DIR):
    print(f"\nComparing with os.walk on: {path}")

    # rustwalk
    start = time.time()
    rustwalk_results = rustwalk.walk(path)
    rustwalk_time = time.time() - start

    # os.walk
    start = time.time()
    os_walk_results = []
    for root, dirs, files in os.walk(path):
        os_walk_results.append(root)
        for d in dirs:
            os_walk_results.append(os.path.join(root, d))
        for f in files:
            os_walk_results.append(os.path.join(root, f))
    os_walk_time = time.time() - start

    print(f"rustwalk: {len(rustwalk_results)} entries in {rustwalk_time:.4f}s")
    print(f"os.walk:  {len(os_walk_results)} entries in {os_walk_time:.4f}s")
    print(f"Speedup:  {os_walk_time / rustwalk_time:.2f}x")


if __name__ == "__main__":
    # Use current directory or specify a path
    test_path = SAMPLE_DIR

    print("=" * 60)
    print("FastWalk Benchmark Suite")
    print("=" * 60)

    benchmark_walk(test_path)
    benchmark_walk_files(test_path)
    benchmark_walk_with_metadata(test_path)
    benchmark_walk_parallel(test_path)
    compare_with_os_walk(test_path)

    print("\n" + "=" * 60)
    print("Benchmark complete!")
    print("=" * 60)
