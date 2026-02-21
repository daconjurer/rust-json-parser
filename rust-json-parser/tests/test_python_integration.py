import pytest

from rust_json_parser import (
    parse_json,
    parse_json_file,
    dumps,
)


class TestBasicParsing:
    def test_parse_simple_object(self):
        result = parse_json('{"name": "Alice"}')
        assert result["name"] == "Alice"

    def test_parse_nested_structure(self):
        result = parse_json('{"users": [{"id": 1}, {"id": 2}]}')
        assert len(result["users"]) == 2
        assert result["users"][0]["id"] == 1

    def test_parse_all_json_types(self):
        result = parse_json(
            '{"str": "hello", "num": 42, "bool": true, "null": null, "arr": [1,2], "obj": {}}'
        )
        assert result["str"] == "hello"
        assert result["num"] == 42.0
        assert result["bool"] is True
        assert result["null"] is None
        assert result["arr"] == [1.0, 2.0]
        assert result["obj"] == {}


class TestTypeConversions:
    def test_null_becomes_none(self):
        result = parse_json('{"value": null}')
        assert result["value"] is None

    def test_bool_stays_bool(self):
        result = parse_json('{"t": true, "f": false}')
        assert result["t"] is True
        assert result["f"] is False
        assert isinstance(result["t"], bool)

    def test_numbers_are_float(self):
        result = parse_json('{"int": 42, "float": 3.14}')
        assert result["int"] == 42.0
        assert result["float"] == 3.14


class TestErrorHandling:
    def test_parse_error_raises_value_error(self):
        with pytest.raises(ValueError):
            parse_json('{"unclosed": "string')

    def test_file_not_found_raises_io_error(self):
        with pytest.raises(IOError):
            parse_json_file('/nonexistent/file.json')

    def test_error_includes_position(self):
        try:
            parse_json('{"bad": }')
        except ValueError as e:
            assert "position" in str(e).lower()


class TestSerialization:
    def test_dumps_basic(self):
        result = dumps({"key": "value"})
        assert '"key"' in result
        assert '"value"' in result

    def test_dumps_with_indent(self):
        result = dumps({"key": "value"}, indent=2)
        assert "{\n  \"key\": \"value\"\n}" == result
