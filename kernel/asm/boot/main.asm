global start
extern long_mode_start

section .boottext exec
bits 32
start:
 	mov esp, stack_top

	call check_multiboot
	call check_cpuid
	call check_long_mode
	call check_sse

	call setup_page_tables
	call enable_paging

	lgdt [gdt64.pointer]
	jmp gdt64.code_segment:long_mode_start

	hlt

; https://wiki.osdev.org/SSE
check_sse:
	;now enable SSE and the like
	mov eax, cr0
	and ax, 0xFFFB		;clear coprocessor emulation CR0.EM
	or ax, 0x2			;set coprocessor monitoring  CR0.MP
	mov cr0, eax
	mov eax, cr4
	or ax, 3 << 9		;set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
	mov cr4, eax
	ret

check_multiboot:
	cmp eax, 0x36d76289
	jne .no_multiboot
	ret
.no_multiboot:
	mov al, "M"
	jmp error

check_cpuid:
	pushfd
	pop eax
	mov ecx, eax
	xor eax, 1 << 21
	push eax
	popfd
	pushfd
	pop eax
	push ecx
	popfd
	cmp eax, ecx
	je .no_cpuid
	ret
.no_cpuid:
	mov al, "C"
	jmp error

check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no_long_mode

	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29
	jz .no_long_mode
	
	ret
.no_long_mode:
	mov al, "L"
	jmp error

setup_page_tables:
	; EQU is used to define constants in assembly
	%define HUGE_PAGE_SIZE    0x200000     ; 2 MiB
	%define PAGE_ENTRY_FLAGS        0b10000011    ; huge page, no access from user, writable, present
	%define PAGE_TABLE_FLAGS        0b011    ; huge page, no access from user, writable, present

	mov eax, page_table_l3
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l4], eax
	; temporarily map the same l3 table also to middle of l4 of virtual memory to enable later switch to higher half kernel
	mov [page_table_l4+256*8], eax
	
	mov eax, page_table_l2
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l3], eax

	mov ecx, 0 ; counter
.loop:
	
	mov eax, HUGE_PAGE_SIZE
	mul ecx
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l2 + ecx * 8], eax

	inc ecx ; increment counter
	cmp ecx, 20 ; checks if the whole table is mapped
	; cmp ecx, 512 ; checks if the whole table is mapped
	jne .loop ; if not, continue

	; TODO map video memory also: probably wrong virtual memory location on the long term; 
	mov eax, 0x000000
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l2 + 510 * 8], eax

	ret

enable_paging:
	; pass page table location to cpu
	mov eax, page_table_l4
	mov cr3, eax

	; enable PAE
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; enable long mode
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; enable paging
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	ret

error:
	; print "ERR: X" where X is the error code
	mov dword [0xb8000], 0x4f524f45
	mov dword [0xb8004], 0x4f3a4f52
	mov dword [0xb8008], 0x4f204f20
	mov byte  [0xb800a], al
	hlt

section .bootbss
align 4096
page_table_l4:
	resb 4096
page_table_l3:
	resb 4096
page_table_l2:
	resb 4096
; TODO Why is stack in bss? Does this make sense? Also Stack is pretty small
stack_bottom:
	resb 4096 * 4000
stack_top:

section .bootrodata
gdt64:
	dq 0 ; zero entry
.code_segment: equ $ - gdt64
	dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ; code segment
.pointer:
	dw $ - gdt64 - 1 ; length
	dq gdt64 ; address
