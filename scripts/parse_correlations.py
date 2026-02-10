#!/usr/bin/env python3
"""
Parse parking zone correlations from JSON to Rust code.

Usage:
    python parse_correlations.py input.json > output.rs
    python parse_correlations.py address_correlations.json > entries.rs

The output can be directly pasted into android/src/static_data.rs
"""

import json
import sys


def escape_rust_string(s: str) -> str:
    """Escape a string for use in Rust code."""
    # Handle common escape sequences
    s = s.replace('\\', '\\\\')
    s = s.replace('"', '\\"')
    s = s.replace('\n', '\\n')
    s = s.replace('\r', '\\r')
    s = s.replace('\t', '\\t')
    return s


def parse_json_to_rust(json_file: str):
    """Parse JSON and output Rust code for StaticAddressEntry vector."""
    try:
        with open(json_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except FileNotFoundError:
        print(f"Error: File '{json_file}' not found", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON - {e}", file=sys.stderr)
        sys.exit(1)

    if not isinstance(data, list):
        print("Error: JSON must be an array of objects", file=sys.stderr)
        sys.exit(1)

    if not data:
        print("Error: JSON array is empty", file=sys.stderr)
        sys.exit(1)

    print("let entries = vec![")

    for i, entry in enumerate(data):
        # Required fields
        try:
            adress = escape_rust_string(entry['adress'])
            gata = escape_rust_string(entry['gata'])
            gatunummer = escape_rust_string(entry['gatunummer'])
            postnummer = int(entry['postnummer'])
            dag = int(entry['dag'])
            tid = escape_rust_string(entry['tid'])
        except (KeyError, ValueError, TypeError) as e:
            print(f"Error in entry {i}: Missing or invalid field - {e}", file=sys.stderr)
            sys.exit(1)

        # Optional fields
        info = escape_rust_string(entry.get('info', 'Parkering'))
        distance = float(entry.get('distance', 0.0))

        # Validate dag is 1-31
        if not 1 <= dag <= 31:
            print(f"Warning: Entry {i} has invalid dag={dag} (should be 1-31)", file=sys.stderr)

        # Validate tid format HHMM-HHMM
        if not (
            len(tid) == 9
            and tid[4] == '-'
            and tid[:4].isdigit()
            and tid[5:].isdigit()
        ):
            print(f"Warning: Entry {i} has invalid tid format '{tid}' (should be HHMM-HHMM)", file=sys.stderr)

        print(f"    StaticAddressEntry {{")
        print(f'        adress: "{adress}".to_string(),')
        print(f'        gata: "{gata}".to_string(),')
        print(f'        gatunummer: "{gatunummer}".to_string(),')
        print(f"        postnummer: {postnummer},")
        print(f"        dag: {dag},")
        print(f'        tid: "{tid}".to_string(),')
        print(f'        info: "{info}".to_string(),')
        print(f"        distance: {distance},")
        print(f"    }},")

    print("];")
    print()
    print(f"// Generated {len(data)} address entries")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python parse_correlations.py <input.json>", file=sys.stderr)
        print()
        print("Example:", file=sys.stderr)
        print("  cargo run --release correlate -c 20 -a kdtree > correlations.json", file=sys.stderr)
        print("  python parse_correlations.py correlations.json > entries.rs", file=sys.stderr)
        sys.exit(1)

    parse_json_to_rust(sys.argv[1])
