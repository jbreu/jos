import argparse
import os
import shutil
import subprocess
import tempfile
import time
from datetime import datetime

# ---------------------------
# Argument parsing
# ---------------------------
parser = argparse.ArgumentParser(
    description="Sample GDB backtraces from a remote target."
)
parser.add_argument(
    "--interval", type=int, default=10, help="Sampling interval in milliseconds"
parser.add_argument("--count", type=int, default=1000, help="Number of samples")
parser.add_argument("--target", type=str, default=":1234", help="GDB remote target")
parser.add_argument(
    "--executable",
    type=str,
    default="../../dist/x86_64/kernel.bin",
    help="Path to the ELF binary",
)

args = parser.parse_args()

# ---------------------------
# Pre-flight checks
# ---------------------------
if not shutil.which("gdb"):
    print("Error: gdb is not found in PATH.")
    exit(1)

if not os.path.isfile(args.executable):
    print(f"Error: Executable '{args.executable}' not found.")
    exit(1)

# ---------------------------
# Prepare output
# ---------------------------
output_file = "stacks.txt"
print(f"Writing output to: {output_file}")

# ---------------------------
# Sampling loop
# ---------------------------
# Determine the sleep command based on the platform
sleep_cmd = "timeout /nobreak /t" if os.name == "nt" else "sleep"
# Convert milliseconds to seconds and ensure it's a string with decimal point
sleep_duration = f"{args.interval/1000.0:.3f}"

gdb_cmds = f"""
set osabi none
set pagination off
set logging file stacks.txt
set logging enabled off
set logging enabled
file {args.executable}
add-symbol-file ../../build/userspace/x86_64-unknown-none/debug/helloworld

set $i = 0
while ($i < {args.count})
    target remote {args.target}
    interrupt
    thread apply all bt
    echo Sample #
    print $i + 1
    set $i = $i + 1
    detach
    shell {sleep_cmd} {sleep_duration}
end

quit
"""

with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".gdb") as tf:
    tf.write(gdb_cmds)
    temp_path = tf.name

try:
    result = subprocess.run(
        ["gdb", "-nx", "--batch", "-x", temp_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=10 * args.count,  # Adjust timeout for multiple samples
    )

finally:
    os.remove(temp_path)

print(f"Sampling complete. Results stored in '{output_file}'.")
