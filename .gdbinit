set output-radix 16
set disassembly-flavor intel
add-symbol-file build/userspace/x86_64-unknown-none/debug/helloworld
display/5i $pc
display/20xg $sp
#b *(syscall_handler+16)
#b *(syscall_handler+27) if *($sp) == 0x0
#b *(helloworld::libc::write+75)
# libc::write:
#break *0x0000000000201cd9 
#disable 1
#if *($rsp) == 0x0
#disable 2
#b isr_common_stub
#b irq_common_stub
#b *(irq_common_stub+44)
#b *(irq_common_stub+46)
#b *(irq_common_stub+73)
b *0x0
b *0x2
b *0x3
b *0x4
b *0x5

#set logging on
#set height 0

define log_instructions    
  while 1
    x/i $pc
    stepi
  end
end