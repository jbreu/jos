// https://github.com/scprogramming/Jazz2.0/blob/main/src/interrupts/idt.s
// https://wiki.osdev.org/ISR
// https://wiki.osdev.org/Interrupts_Tutorial

.code64

.section .data
pushed_registers:
    .quad 0
.globl pushed_registers
stack_frame:
    .quad 0
.globl stack_frame
int_no:
    .quad 0

.section .text

.macro push_all_registers
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12 
    push r13
    push r14
    push r15
    
    sub rsp,0x10
    movdqu [rsp],xmm0
    sub rsp,0x10
    movdqu [rsp],xmm1
    sub rsp,0x10
    movdqu [rsp],xmm2
    sub rsp,0x10
    movdqu [rsp],xmm3
    sub rsp,0x10
    movdqu [rsp],xmm4
    sub rsp,0x10
    movdqu [rsp],xmm5
    sub rsp,0x10
    movdqu [rsp],xmm6
    sub rsp,0x10
    movdqu [rsp],xmm7
    
    mov [rip + pushed_registers], rsp
.endm

.macro pop_all_registers
    movdqu xmm7,[rsp]
    add rsp,0x10
    movdqu xmm6,[rsp]
    add rsp,0x10
    movdqu xmm5,[rsp]
    add rsp,0x10
    movdqu xmm4,[rsp]
    add rsp,0x10
    movdqu xmm3,[rsp]
    add rsp,0x10
    movdqu xmm2,[rsp]
    add rsp,0x10
    movdqu xmm1,[rsp]
    add rsp,0x10
    movdqu xmm0,[rsp]
    add rsp,0x10

    pop r15
    pop r14 
    pop r13 
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
.endm

.macro ISR_NOERRCODE isr
    .globl isr\isr
    isr\isr\():
        cli
        push qword ptr 0
        push qword ptr \isr
        jmp isr_common_stub
.endm

.macro ISR_ERRCODE isr
    .globl isr\isr
    isr\isr\():
        cli
        push qword ptr \isr
        jmp isr_common_stub
.endm

.macro IRQ irq, number
    .globl irq\irq
    irq\irq\():
        mov qword ptr [rip + int_no], \number
        mov [rip + stack_frame], rsp
        push_all_registers
        jmp irq_common_stub
.endm

ISR_NOERRCODE 0
ISR_NOERRCODE 1
ISR_NOERRCODE 2
ISR_NOERRCODE 3
ISR_NOERRCODE 4
ISR_NOERRCODE 5
ISR_NOERRCODE 6
ISR_NOERRCODE 7

ISR_ERRCODE 8
ISR_NOERRCODE 9 
ISR_ERRCODE 10
ISR_ERRCODE 11
ISR_ERRCODE 12
ISR_ERRCODE 13
ISR_ERRCODE 14

ISR_NOERRCODE 15
ISR_NOERRCODE 16
ISR_ERRCODE 17
ISR_NOERRCODE 18
ISR_NOERRCODE 19
ISR_NOERRCODE 20
ISR_ERRCODE 21
ISR_NOERRCODE 22
ISR_NOERRCODE 23
ISR_NOERRCODE 24
ISR_NOERRCODE 25
ISR_NOERRCODE 26
ISR_NOERRCODE 27
ISR_NOERRCODE 28
ISR_ERRCODE 29
ISR_ERRCODE 30
ISR_NOERRCODE 31
ISR_NOERRCODE 128
ISR_NOERRCODE 177

IRQ   0,    32
IRQ   1,    33
IRQ   2,    34
IRQ   3,    35
IRQ   4,    36
IRQ   5,    37
IRQ   6,    38
IRQ   7,    39
IRQ   8,    40
IRQ   9,    41
IRQ  10,    42
IRQ  11,    43
IRQ  12,    44
IRQ  13,    45
IRQ  14,    46
IRQ  15,    47

// https://www.reddit.com/r/osdev/comments/cp40lb/64bit_isr_handler_breaking_my_stack
// https://github.com/rust-osdev/x86_64/issues/392#issuecomment-1257883895

isr_common_stub:
	// https://aaronbloomfield.github.io/pdr/book/x86-64bit-ccc-chapter.pdf
    // https://www.ired.team/miscellaneous-reversing-forensics/windows-kernel-internals/linux-x64-calling-convention-stack-frame 
    push rdi // save previous value to stack as we are gonna using it to pass arguments to isr_handler
	push rsi
    push rax
    // Attention! --> when you push more registers here, you have to update the following stack references

	mov rdi, [rsp+4*8]	// put the the error number into rsi (1nd argument for isr_handler)// it has been previously pushed onto the stack (see macros above)
	mov rsi, [rsp+3*8]	// put the the isr number into rdi (2nd argument for isr_handler)// it has been previously pushed onto the stack (see macros above)

    lea rax, [rip + isr_handler]
	call rax

    pop rax
	pop rsi
	pop rdi

	add rsp, 16 // "pop" the two longs we have pushed originally
	sti
	iretq

irq_common_stub:

    mov rdi, [rip + int_no]

    lea rax, [rip + irq_handler]
	call rax

    pop_all_registers

	iretq