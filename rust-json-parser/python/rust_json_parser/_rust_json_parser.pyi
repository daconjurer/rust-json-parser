"""Type stubs for the rust_json_parser native module."""

from typing import Any, overload

JsonType = dict[str, Any] | list[Any] | str | float | bool | None

def parse_json(input: str) -> JsonType:
    """Parse a JSON string and return the corresponding Python object.

    Args:
        input: A string containing valid JSON.

    Returns:
        The parsed JSON as a Python object (dict, list, str, float, bool, or None).

    Raises:
        ValueError: If the input is not valid JSON.

    Examples:
        >>> parse_json('{"name": "Alice", "age": 30}')
        {'name': 'Alice', 'age': 30.0}

        >>> parse_json('[1, 2, 3]')
        [1.0, 2.0, 3.0]
    """
    ...

def parse_json_file(path: str) -> JsonType:
    """Parse a JSON file and return the corresponding Python object.

    Args:
        path: Path to a file containing valid JSON.

    Returns:
        The parsed JSON as a Python object (dict, list, str, float, bool, or None).

    Raises:
        ValueError: If the file contents are not valid JSON.
        OSError: If the file cannot be read.

    Examples:
        >>> parse_json_file("config.json")
        {'key': 'value'}
    """
    ...

@overload
def dumps(obj: JsonType) -> str: ...
@overload
def dumps(obj: JsonType, indent: int) -> str: ...
@overload
def dumps(obj: JsonType, indent: None = None) -> str: ...
def dumps(obj: JsonType, indent: int | None = None) -> str:
    """Serialize a Python object to a JSON string.

    Args:
        obj: A Python object to serialize (dict, list, str, float, int, bool, or None).
        indent: Optional number of spaces for pretty-printing. If None, output is compact.

    Returns:
        A JSON string representation of the object.

    Raises:
        TypeError: If the object contains types that cannot be serialized to JSON.

    Examples:
        >>> dumps({"name": "Alice", "age": 30})
        '{"name": "Alice", "age": 30}'

        >>> print(dumps({"key": "value"}, indent=2))
        {
          "key": "value"
        }
    """
    ...

def benchmark_performance(
    input: str,
    rounds: int = 1000,
    warmup: int = 10,
) -> dict[str, float]:
    """Benchmark parse_json against json.loads and simplejson.loads.

    Args:
        input: A JSON string to parse.
        rounds: Number of timed iterations per parser (default: 1000).
        warmup: Number of untimed warmup iterations per parser (default: 10).

    Returns:
        A dict with median per-iteration times in seconds:
        {"pure-rust": float, "rust": float, "json": float, "simplejson": float}.
    """
    ...
