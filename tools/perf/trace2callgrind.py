#!/usr/bin/env python3

import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Set, Tuple


class CallNode:
    def __init__(self, name: str):
        self.name = name
        self.inclusive_time = 0
        self.self_time = 0
        self.inclusive_cycles = 0
        self.self_cycles = 0
        self.call_count = 0
        self.callers: Dict[str, int] = defaultdict(int)
        self.callees: Dict[str, int] = defaultdict(int)
        self.entry_time = 0
        self.entry_cycles = 0


class TraceProcessor:
    def __init__(self):
        self.nodes: Dict[str, CallNode] = {}
        self.stack: List[Tuple[str, int, int]] = (
            []
        )  # (function_name, entry_time, entry_cycles)
        self.spans: Dict[int, str] = {}  # span_id to function_name mapping

    def process_line(self, line: str):
        # Match timestamp and cycles
        time_match = re.match(r"\[(\d+)ns/(\d+)cyc\]", line)
        if not time_match:
            return

        timestamp = int(time_match.group(1))
        cycles = int(time_match.group(2))

        # Handle span creation (new_span)
        span_enter = re.search(r"ENTER \[(\d+)\] (\S+)", line)
        if span_enter:
            span_id, func_name = span_enter.groups()
            self.spans[int(span_id)] = func_name
            return

        # Handle function entry
        enter_match = re.search(r"ENTER \[(\d+)\]", line)
        if enter_match:
            span_id = int(enter_match.group(1))
            if span_id in self.spans:
                func_name = self.spans[span_id]
                if func_name not in self.nodes:
                    self.nodes[func_name] = CallNode(func_name)
                self.nodes[func_name].call_count += 1
                if self.stack:
                    caller = self.stack[-1][0]
                    self.nodes[caller].callees[func_name] += 1
                    self.nodes[func_name].callers[caller] += 1
                self.stack.append((func_name, timestamp, cycles))
            return

        # Handle function exit
        exit_match = re.search(r"EXIT \[(\d+)\]", line)
        if exit_match and self.stack:
            span_id = int(exit_match.group(1))
            if span_id in self.spans:
                func_name = self.spans[span_id]
                if self.stack and self.stack[-1][0] == func_name:
                    _, entry_time, entry_cycles = self.stack.pop()
                    duration = timestamp - entry_time
                    cycle_count = cycles - entry_cycles
                    node = self.nodes[func_name]
                    node.inclusive_time += duration
                    node.inclusive_cycles += cycle_count
                    if self.stack:
                        # Subtract this time from parent's self time
                        parent = self.nodes[self.stack[-1][0]]
                        parent.self_time -= duration
                        parent.self_cycles -= cycle_count
                    node.self_time += duration
                    node.self_cycles += cycle_count

    def write_callgrind(self, output_path: Path):
        with open(output_path, "w") as f:
            # Write header
            f.write("version: 1\n")
            f.write("creator: jos-kernel-tracer\n")
            f.write("cmd: jos-kernel\n")
            f.write("positions: line\n")
            f.write("events: Cycles Nanoseconds\n\n")

            # Write function data
            for name, node in self.nodes.items():
                # Default to kernel module and source file if we can't determine them
                mod = "kernel"
                source = "kernel/src/kernel.rs"

                # Try to determine the source file from the function name
                if "." in name:
                    mod, func = name.split(".", 1)
                    source = f"kernel/src/{mod}.rs"
                else:
                    func = name

                # Write position spec
                f.write(f"ob={mod}\n")  # Object/module
                f.write(f"fl={source}\n")  # Source file
                f.write(f"fn={func}\n")  # Function name
                f.write(f"1 {node.inclusive_cycles} {node.inclusive_time}\n")

                # Write caller information
                for caller, count in node.callers.items():
                    if "." in caller:
                        caller_mod, caller_func = caller.split(".", 1)
                        caller_source = f"kernel/src/{caller_mod}.rs"
                    else:
                        caller_mod = "kernel"
                        caller_func = caller
                        caller_source = "kernel/src/kernel.rs"

                    f.write(f"cob={caller_mod}\n")
                    f.write(f"cfl={caller_source}\n")
                    f.write(f"cfn={caller_func}\n")
                    f.write(f"calls={count} 1\n")
                    f.write(f"1 {node.inclusive_cycles} {node.inclusive_time}\n")

                # Write callee information
                for callee, count in node.callees.items():
                    if "." in callee:
                        callee_mod, callee_func = callee.split(".", 1)
                        callee_source = f"kernel/src/{callee_mod}.rs"
                    else:
                        callee_mod = "kernel"
                        callee_func = callee
                        callee_source = "kernel/src/kernel.rs"

                    f.write(f"cob={callee_mod}\n")
                    f.write(f"cfl={callee_source}\n")
                    f.write(f"cfn={callee_func}\n")
                    f.write(f"calls={count} 1\n")
                    callee_cycles = self.nodes[callee].inclusive_cycles * count
                    callee_time = self.nodes[callee].inclusive_time * count
                    f.write(f"1 {callee_cycles} {callee_time}\n")

                f.write("\n")


def main():
    if len(sys.argv) != 3:
        print("Usage: trace2callgrind.py <trace_log> <output_callgrind>")
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])

    processor = TraceProcessor()

    with open(input_path) as f:
        for line in f:
            if not line.startswith("[DEBUG]"):
                continue

            line = line.split(" ", 2)[2]  # Remove [DEBUG] prefix
            processor.process_line(line.strip())

    processor.write_callgrind(output_path)
    print(f"Generated callgrind output at {output_path}")


if __name__ == "__main__":
    main()
