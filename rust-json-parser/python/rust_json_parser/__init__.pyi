"""Type stubs for rust_json_parser package."""

from typing import Any

JsonType = dict[str, Any] | list[Any] | str | float | bool | None

def parse_json(input: str) -> JsonType:
    """Parse a JSON string and return the corresponding Python object."""
    ...

def parse_json_file(path: str) -> JsonType:
    """Parse a JSON file and return the corresponding Python object."""
    ...

def dumps(obj: JsonType, indent: int | None = None) -> str:
    """Serialize a Python object to a JSON string."""
    ...

def benchmark_performance(
    input: str,
    rounds: int = 1000,
    warmup: int = 10,
) -> dict[str, float]:
    """Benchmark parse_json against json.loads and simplejson.loads."""
    ...

__all__: list[str]
