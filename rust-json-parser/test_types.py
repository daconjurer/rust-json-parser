from rust_json_parser import parse_json, dumps

result = parse_json('{"key": "value"}')  # Should show: JsonType
output = dumps({"test": 123})            # Should show: str
output_pretty = dumps({"test": 123}, indent=2)  # Should show: str
