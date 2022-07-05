# lite-json

[![Crates.io](https://img.shields.io/crates/v/lite-json)](https://crates.io/crates/lite-json)
[![GitHub](https://img.shields.io/github/license/xlc/lite-json)](https://github.com/xlc/lite-json/blob/master/LICENSE)

Simple JSON parser written with Rust. Wasm / no_std ready.

## How to Add in Cargo.toml

### std
```toml
[dependencies]
lite-json = "0.2.0"
```

### no_std
```toml
[dependencies]
lite-json = { version = "0.2.0", default-features = false, defaults = ["no_std"] }
```

## Example Usage

### Creating JSON

This example will create a lite-json structure, then print it as a JSON string.

```rs
use lite_json::Serialize;
use lite_json::json::{JsonValue, NumberValue};

fn main()
{
	// We will create a bunch of elements that we will put into a JSON Object.
	let mut object_elements = vec!();

	// Create a boolean value and add it to our vector.
	let boolean_value = true;
	let object_key = "boolean".chars().collect();
	object_elements.push((object_key, JsonValue::Boolean(boolean_value)));

	// Create an array value and add it to our vector.
	let array_value = vec!(JsonValue::Boolean(true), JsonValue::Boolean(false), JsonValue::Boolean(true));
	let object_key = "array".chars().collect();
	object_elements.push((object_key, JsonValue::Array(array_value)));

	// Create a string value and add it to our vector.
	let string_value = "Hello World!".chars().collect();
	let object_key = "string".chars().collect();
	object_elements.push((object_key, JsonValue::String(string_value)));

	// Create a number value and add it to our vector.
	let number_value = NumberValue
	{
		integer: 1234,
		fraction: 0,
		fraction_length: 0,
		exponent: 0,
	};
	let object_key = "number".chars().collect();
	object_elements.push((object_key, JsonValue::Number(number_value)));

	// Create a null value and add it to our vector.
	let object_key = "null".chars().collect();
	object_elements.push((object_key, JsonValue::Null));

	// Create the object value from the vector of elements.
	let object_value = JsonValue::Object(object_elements);

	// Convert the object to a JSON string.
	let json = object_value.format(4);
	let json_output = std::str::from_utf8(&json).unwrap();

	println!("{}", json_output);
}
```

This will output:
```json
{
    "boolean": true,
    "array": [
        true,
        false,
        true
    ],
    "string": "Hello World!",
    "number": 1234,
    "null": null
}
```

### Parsing JSON

This example will parse a JSON string into a lite-json structure.

```rs
use lite_json::json_parser::parse_json;

fn main()
{
	// This is the JSON string we will use.
	let json_string =
	r#"
		{
			"boolean": true,
			"array":
			[
				true,
				false,
				true
			],
			"string": "Hello World!",
			"number": 1234,
			"null": null
		}
	"#;

	// Parse the JSON and print the resulting lite-json structure.
	let json_data = parse_json(json_string).expect("Invalid JSON specified!");
	println!("{:?}", json_data);
}
```

### Parsing JSON with Options

The parser options allows you to set the max depth of parsing nested objects. This code will result in an error because the max nest level is set to `1`, but the depth of our JSON is `2` due to the presence of a nested array.

Note: This example requires the `lite-parser` crate to be added to `Cargo.toml`.

```rs
use lite_json::json_parser::parse_json_with_options;
use lite_parser::parser::ParserOptions;

fn main()
{
	// This is the JSON string we will use.
	let json_string =
	r#"
		{
			"boolean": true,
			"array":
			[
				true,
				false,
				true
			],
			"string": "Hello World!",
			"number": 1234,
			"null": null
		}
	"#;

	let parser_options = ParserOptions
	{
		max_nest_level: Some(1)
	};

	// Parse the JSON and print the resulting lite-json structure.
	let json_data = parse_json_with_options(json_string, parser_options).expect("Invalid JSON specified!");
	println!("{:?}", json_data);
}
```
