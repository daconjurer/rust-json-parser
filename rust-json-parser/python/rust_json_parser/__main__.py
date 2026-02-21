import argparse
import sys
from pathlib import Path

from rust_json_parser import (
    dumps,
    parse_json,
    parse_json_file,
)


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
    args = parser.parse_args()

    if args.input is None:
        if sys.stdin.isatty():
            parser.error("no input provided (pass a file, a JSON string, or pipe to stdin)")
        raw = sys.stdin.read()
        result = parse_json(raw)
    elif Path(args.input).is_file():
        result = parse_json_file(args.input)
    else:
        result = parse_json(args.input)

    print(dumps(result, indent=args.indent))


if __name__ == "__main__":
    main()
