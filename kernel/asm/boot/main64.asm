global long_mode_start
extern kernel_main

section .boottext
bits 64
long_mode_start:
    ; load null into all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    add rsp, 0xffff800000000000

    mov rax, QWORD kernel_main
	call rax
    hlt
