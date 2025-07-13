import re
import sys
import gzip
from collections import defaultdict
import profile_pb2  # generated from pprof's profile.proto


def parse_log(logfile):
    # Regex for log lines
    r = re.compile(
        r"\[(\w+)\] \[(\d+)\] \[(\d+)ns/\d+cyc\] (ENTER|EXIT) \[(\d+)\] ([\w:]+)"
    )
    events = []
    with open(logfile, "r") as f:
        for line in f:
            m = r.search(line)
            if m:
                ts = int(m.group(3))
                typ = m.group(4)
                func = m.group(6)
                events.append((ts, typ, func))
    return events


def build_samples(events):
    stack = []
    last_ts = None
    samples = defaultdict(int)
    for ts, typ, func in events:
        if typ == "ENTER":
            stack.append((func, ts))
        elif typ == "EXIT":
            # Find matching ENTER
            for i in range(len(stack) - 1, -1, -1):
                if stack[i][0] == func:
                    enter_func, enter_ts = stack.pop(i)
                    duration = ts - enter_ts
                    samples[func] += duration
                    break
    return samples


def build_callgraph_events(events):
    """
    Returns:
        callgraph: dict of caller -> dict of callee -> total_time
        callstacks: list of (stack, duration) for each EXIT event
    """
    stack = []
    callgraph = defaultdict(lambda: defaultdict(int))
    callstacks = []
    for ts, typ, func in events:
        if typ == "ENTER":
            stack.append((func, ts))
        elif typ == "EXIT":
            for i in range(len(stack) - 1, -1, -1):
                if stack[i][0] == func:
                    enter_func, enter_ts = stack.pop(i)
                    duration = ts - enter_ts
                    if stack:
                        caller = stack[-1][0]
                        callgraph[caller][func] += duration
                    callstacks.append(([f for f, _ in stack] + [func], duration))
                    break
    return callgraph, callstacks


def build_pprof(samples):
    profile = profile_pb2.Profile()
    # Add string table
    profile.string_table.append("")  # id=0 is empty
    str_ids = {}

    def str_id(s):
        if s not in str_ids:
            str_ids[s] = len(profile.string_table)
            profile.string_table.append(s)
        return str_ids[s]

    # Add functions and locations
    func_ids = {}
    for i, func in enumerate(samples):
        f = profile.function.add()
        f.id = i + 1
        f.name = str_id(func)
        f.system_name = str_id(func)
        f.filename = str_id("")
        f.start_line = 0
        func_ids[func] = f.id
        loc = profile.location.add()
        loc.id = i + 1
        loc.line.add(function_id=f.id, line=0)
    # Add samples
    for func, dur in samples.items():
        s = profile.sample.add()
        s.location_id.append(func_ids[func])
        s.value.append(dur)
    # Set sample_type (time in nanoseconds)
    st = profile.sample_type.add()
    st.type = str_id("cpu")
    st.unit = str_id("nanoseconds")
    return profile


def build_pprof_with_callgraph(samples, callstacks):
    profile = profile_pb2.Profile()
    profile.string_table.append("")  # id=0 is empty
    str_ids = {}

    def str_id(s):
        if s not in str_ids:
            str_ids[s] = len(profile.string_table)
            profile.string_table.append(s)
        return str_ids[s]

    # Add functions
    func_ids = {}
    for i, func in enumerate(samples):
        f = profile.function.add()
        f.id = i + 1
        f.name = str_id(func)
        f.system_name = str_id(func)
        f.filename = str_id("")
        f.start_line = 0
        func_ids[func] = f.id
    # Add locations (one per function)
    loc_ids = {}
    for i, func in enumerate(samples):
        loc = profile.location.add()
        loc.id = i + 1
        loc.line.add(function_id=func_ids[func], line=0)
        loc_ids[func] = loc.id
    # Add samples with call stacks
    for stack, dur in callstacks:
        s = profile.sample.add()
        # Stack: bottom (root) to top (leaf)
        s.location_id.extend([loc_ids[f] for f in stack if f in loc_ids])
        s.value.append(dur)
    # Set sample_type (time in nanoseconds)
    st = profile.sample_type.add()
    st.type = str_id("cpu")
    st.unit = str_id("nanoseconds")
    return profile


def main():
    if len(sys.argv) != 3:
        print("Usage: trace2pprof.py serial.log output.pb.gz")
        sys.exit(1)
    events = parse_log(sys.argv[1])
    samples = build_samples(events)
    callgraph, callstacks = build_callgraph_events(events)
    profile = build_pprof_with_callgraph(samples, callstacks)
    with gzip.open(sys.argv[2], "wb") as f:
        f.write(profile.SerializeToString())


if __name__ == "__main__":
    main()
