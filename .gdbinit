set output-radix 16
set disassembly-flavor intel
set osabi none
set substitute-path /root/env/userland/ c:/Users/Jakob/Documents/workspace/os-series/userland/
add-symbol-file c:/Users/Jakob/Documents/workspace/os-series/userland/dash-0.5.13/src/dash
display/5i $pc
display/20xg $sp

#b *0x100018
#commands
#  add-symbol-file dist/x86_64/kernel.bin
#  b isr_common_stub
#  #b irq_common_stub
#  #b *(irq_common_stub+44)
#  b *0x0
#  b *0x2
#  b *0x3
#  b *0x4
#  b *0x5
#end

#set logging on
#set height 0

define log_instructions    
  while $pc >= 0x1000
    stepi
  end
end

define print_palette
  set $i = 0
  while $i < 768
    printf "%02x %02x %02x\n", *(palette + $i), *(palette + $i + 1), *(palette + $i + 2)
    set $i = $i + 3
  end
end


define flamesample
  set logging off
  set pagination 0
  set $i = 0
  set logging file tools/perf/stacks.txt
  while $i < 200
    set logging on
    thread apply all bt
    set logging off
    stepi 10000
    interrupt
    set $i = $i + 1
  end  
end