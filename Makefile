kernel_source_files := $(shell find src/impl/kernel -name *.rs)
kernel_object_files := $(patsubst src/impl/kernel/%.rs, build/kernel/%.o, $(kernel_source_files))

x86_64_rs_source_files := $(shell find src/impl/x86_64 -name *.rs)
x86_64_rs_object_files := $(patsubst src/impl/x86_64/%.rs, build/x86_64/%.o, $(x86_64_c_source_files))

x86_64_asm_source_files := $(shell find src/impl/x86_64 -name *.asm)
x86_64_asm_object_files := $(patsubst src/impl/x86_64/%.asm, build/x86_64/%.o, $(x86_64_asm_source_files))

x86_64_object_files := $(x86_64_rs_object_files) $(x86_64_asm_object_files)

$(kernel_object_files): build/kernel/%.o : src/impl/kernel/%.rs
	mkdir -p $(dir $@) && \
	rustc -C link-arg=-nostartfiles --target=x86_64-unknown-none --emit=obj $(patsubst build/kernel/%.o, src/impl/kernel/%.rs, $@) -o $@

$(x86_64_rs_object_files): build/x86_64/%.o : src/impl/x86_64/%.rs
	mkdir -p $(dir $@) && \
	rustc -C link-arg=-nostartfiles --target=x86_64-unknown-none --emit=obj $(patsubst build/x86_64/%.o, src/impl/x86_64/%.rs, $@) -o $@

$(x86_64_asm_object_files): build/x86_64/%.o : src/impl/x86_64/%.asm
	mkdir -p $(dir $@) && \
	nasm -f elf64 $(patsubst build/x86_64/%.o, src/impl/x86_64/%.asm, $@) -o $@

.PHONY: build-x86_64
build-x86_64: $(kernel_object_files) $(x86_64_object_files)
	mkdir -p dist/x86_64 && \
	x86_64-elf-ld -n -o dist/x86_64/kernel.bin -T targets/x86_64/linker.ld $(kernel_object_files) $(x86_64_object_files) && \
	cp dist/x86_64/kernel.bin targets/x86_64/iso/boot/kernel.bin && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso
