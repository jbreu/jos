# Performance Analysis Tools

## Generate Flamegraphs

While JOS is running in qemu with gdb server turned on, run psample.py to collect stack samples:

```bash
python psample.py --interval 100 --count 100 --target :1234 --executable ../../dist/x86_64/kernel.bin
```

Parameters:
- `--interval`: Sampling interval in milliseconds (default: 100)
- `--count`: Number of samples to collect (default: 100)
- `--target`: GDB remote target (default: :1234)
- `--executable`: Path to the kernel ELF binary (default: ../../dist/x86_64/kernel.bin)

This will create stacks.txt with the collected stack samples.

Then run generateFlameGraph.sh to create the flamegraph

Make sure to delete stacks.txt in between sessions

# Generate tracefiles

Run in qemu with serial console connected; in the QEMU console window, press the "l" key to write out the trace buffer. This step is automated in the test_retrieve_profiling.

Then call serial2chrome.py with the serial as input to generate Chrome trace format files. You can open the generated file in Chrome trace view, or for a better experience, upload it to Perfetto at https://ui.perfetto.dev/#!/viz by clicking "Open trace file" and selecting your trace file.

# Generate Google pprof files

Call trace2pprof.py ../../serial.log output.pb.gz to generate pprof files (https://github.com/google/pprof)

Run pprof -http=localhost:1234 .\output.pb.gz to examine in your browser some graphical details