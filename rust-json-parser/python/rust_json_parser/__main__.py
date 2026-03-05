import argparse
import os
import sys
from pathlib import Path

from rust_json_parser import (
    benchmark_performance,
    dumps,
    parse_json,
    parse_json_file,
)

BENCHMARK_ROUNDS = 1000
WARMUP_ROUNDS = 10


def _human_size(nbytes: int) -> str:
    for unit in ("bytes", "KB", "MB"):
        if nbytes < 1024 or unit == "MB":
            return f"{nbytes:.0f} {unit}" if unit == "bytes" else f"{nbytes:.1f} {unit}"
        nbytes /= 1024
    return f"{nbytes:.1f} MB"


def _auto_rounds(size: int, requested: int) -> int:
    """Scale rounds down for large files to keep runtime reasonable."""
    if size > 1_000_000:
        return max(10, requested // 100)
    if size > 100_000:
        return max(50, requested // 10)
    return requested


def _comparison(label: str, other_time: float, rust_time: float) -> str:
    if other_time >= rust_time:
        pct = (other_time / rust_time - 1) * 100
        return f"  {label:<22} {other_time:.9f}s  (Rust with bindings is {pct:.0f}% faster)"
    pct = (rust_time / other_time - 1) * 100
    return f"  {label:<22} {other_time:.9f}s  ({label.rstrip(':')} is {pct:.0f}% faster than Rust with Python bindings)"


def _benchmark_file(path: str, rounds: int, warmup: int) -> None:
    raw = open(path).read()
    size = os.path.getsize(path)
    rounds = _auto_rounds(size, rounds)
    name = os.path.basename(path)

    times = benchmark_performance(raw, rounds=rounds, warmup=warmup)

    print(f"\n{name} ({_human_size(size)}, {rounds} rounds):")
    print(f"  {'Rust with bindings:':<22} {times['rust']:.9f}s")
    print(_comparison("Rust:", times["pure-rust"], times["rust"]))
    print(_comparison("Python json (C):", times["json"], times["rust"]))
    print(_comparison("simplejson:", times["simplejson"], times["rust"]))


def run_benchmark(test_data_dir: str, rounds: int, warmup: int) -> None:
    files = sorted(Path(test_data_dir).glob("*.json"))
    if not files:
        print(f"No JSON files found in {test_data_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Benchmarking {len(files)} files (including pure Rust implementation)...")

    for f in files:
        _benchmark_file(str(f), rounds, warmup)

    print()


def main():
    parser = argparse.ArgumentParser(
        description="Parse and pretty-print JSON using rust-json-parser",
    )
    parser.add_argument(
        "input",
        nargs="?",
        help="JSON file path or inline JSON string (reads stdin if omitted)",
    )
    parser.add_argument(
        "--indent",
        type=int,
        default=2,
        help="indentation level for output (default: 2)",
    )
    parser.add_argument(
        "--benchmark",
        nargs="?",
        const="test-data",
        metavar="DIR",
        help="run performance comparisons against json and simplejson (default dir: test-data)",
    )
    parser.add_argument(
        "--rounds",
        type=int,
        default=BENCHMARK_ROUNDS,
        help=f"number of benchmark iterations per file (default: {BENCHMARK_ROUNDS})",
    )
    parser.add_argument(
        "--warmup",
        type=int,
        default=WARMUP_ROUNDS,
        help=f"number of warmup iterations per parser (default: {WARMUP_ROUNDS})",
    )
    args = parser.parse_args()

    if args.benchmark is not None:
        run_benchmark(args.benchmark, args.rounds, args.warmup)
        return

    if args.input is None:
        if sys.stdin.isatty():
            parser.error(
                "no input provided (pass a file, a JSON string, or pipe to stdin)"
            )
        raw = sys.stdin.read()
        result = parse_json(raw)
    elif Path(args.input).is_file():
        result = parse_json_file(args.input)
    else:
        result = parse_json(args.input)

    print(dumps(result, indent=args.indent))


if __name__ == "__main__":
    main()
