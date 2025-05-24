#!/usr/bin/env python3

import json
import re
import sys
from typing import List, Dict, Any, Optional, Tuple


def parse_trace_line(line: str) -> Optional[Dict[str, Any]]:
    """Parse a single trace line into components."""
    # Match lines like:
    # [DEBUG] [7245] [7093900ns/14537706611cyc] ENTER [1] init_gdt
    # [DEBUG] [11715] [10826500ns/14554226763cyc] EXIT [1]
    pattern = (
        r".*\[DEBUG\] \[(\d+)\] \[(\d+)ns/(\d+)cyc\] " r"(ENTER|EXIT) \[(\d+)\](.*)"
    )
    match = re.match(pattern, line)

    if not match:
        return None

    try:
        debug_time, ns_time, cycles, event_type, call_id, func_name = match.groups()
        return {
            "debug_time": int(debug_time),
            "timestamp_ns": int(ns_time),
            "cycles": int(cycles),
            "event": event_type,
            "call_id": int(call_id),
            "name": (func_name.strip() if func_name.strip() else f"<id_{call_id}>"),
        }
    except (IndexError, ValueError):
        return None


def convert_to_chrome_trace(input_lines: List[str]) -> Dict[str, Any]:
    """Convert trace data to Chrome tracing format."""
    events = []
    process_id = 1
    call_stack = []  # Stack of call info dicts

    for line in input_lines:
        parsed = parse_trace_line(line)
        if not parsed:
            continue

        if parsed["event"] == "ENTER":
            if parsed["name"].startswith("<id_"):
                continue
            # Push to stack, record nesting
            nesting_level = len(call_stack)

            call_stack.append(
                {
                    "call_id": parsed["call_id"],
                    "name": parsed["name"],
                    "timestamp_ns": parsed["timestamp_ns"],
                    "cycles": parsed["cycles"],
                    "debug_time": parsed["debug_time"],
                    "nesting_level": nesting_level,
                }
            )

            events.append(
                {
                    "name": parsed["name"],
                    "pid": process_id,
                    "tid": 1,  # single-threaded trace
                    "ts": parsed["timestamp_ns"] / 1000.0,  # ns to us
                    "cat": "kernel_function",
                    "ph": "B",
                    "args": {
                        "cycles": parsed["cycles"],
                        "debug_time": parsed["debug_time"],
                        "call_id": parsed["call_id"],
                        "nesting_level": nesting_level,
                    },
                }
            )
        elif parsed["event"] == "EXIT":
            # Find the matching ENTER by call_id, searching from the top

            if not call_stack:
                print(f"Warning: No matching ENTER for EXIT {parsed['call_id']}")
                continue

            enter = call_stack.pop()
            if enter["call_id"] == parsed["call_id"]:
                nesting_level = enter["nesting_level"]
                events.append(
                    {
                        "name": enter["name"],
                        "pid": process_id,
                        "tid": 1,
                        "ts": parsed["timestamp_ns"] / 1000.0,
                        "cat": "kernel_function",
                        "ph": "E",
                        "args": {
                            "cycles": parsed["cycles"],
                            "debug_time": parsed["debug_time"],
                            "call_id": parsed["call_id"],
                            "nesting_level": nesting_level,
                        },
                    }
                )
                # Remove only the matched entry

    return {
        "traceEvents": events,
        "displayTimeUnit": "ns",
        "metadata": {"process_name": "JOS Kernel Functions", "timestamp_unit": "ns"},
    }


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <input_file>")
        sys.exit(1)

    input_file = sys.argv[1]

    try:
        with open(input_file, "r") as f:
            lines = f.readlines()

        trace_data = convert_to_chrome_trace(lines)

        output_file = "chrome_trace.json"
        with open(output_file, "w") as f:
            json.dump(trace_data, f, indent=2)

        print(
            f"""Trace data has been written to {output_file}

To view the trace:
1. Open Chrome browser
2. Go to chrome://tracing
3. Click 'Load' and select {output_file}

The visualization will show:
- Function calls organized by nesting level
- Accurate timing information in both nanoseconds and CPU cycles
- Call stack depth and relationships"""
        )

    except FileNotFoundError:
        print(f"Error: Could not find input file {input_file}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {str(e)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
