; https://wiki.osdev.org/Getting_to_Ring_3#sysret_method
; https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/24593.pdf p. 174
; https://wiki.osdev.org/SYSENTER

; note: this code is for 64-bit long mode only.
;       it is unknown if it works in protected mode.
;       using intel assembly style

global jump_usermode
extern userland
jump_usermode:
	; enable system call extensions that enable sysret and syscall
	mov rcx, 0xc0000080
	rdmsr
	or eax, 1
	wrmsr
	; define SYSCALL CS and SS and 32-bit SYSCALL Target EIP (latter is not needed I think)
	mov rcx, 0xc0000081
	rdmsr
	mov edx, 0x00000000; 0x00180008
	wrmsr
	; define a handler for syscalls and write it to lstar register
	mov rcx, 0xc0000082
	mov rax, syscall_handler
	mov rdx, 0x0
	wrmsr
 
	mov ecx, userland ; to be loaded into RIP
	mov r11, 0x202 ; to be loaded into EFLAGS
	o64 sysret ;use "o64 sysret" if you assemble with NASM

extern system_call
extern userland_loop
syscall_handler:
    swapgs

    call system_call

    swapgs
    sti

	mov ecx, userland_loop ; to be loaded into RIP
	mov r11, 0x202 ; to be loaded into EFLAGS
	o64 sysret