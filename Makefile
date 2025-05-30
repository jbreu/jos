x86_64_asm_source_files := $(shell find kernel/asm -name *.asm)
x86_64_asm_object_files := $(patsubst kernel/asm/%.asm, build/x86_64/%.o, $(x86_64_asm_source_files))

$(x86_64_asm_object_files): build/x86_64/%.o : kernel/asm/%.asm
	mkdir -p $(dir $@) && \
	nasm -f elf64 -g -F dwarf $(patsubst build/x86_64/%.o, kernel/asm/%.asm, $@) -o $@

#
#cargo rustc --manifest-path userland/Cargo.toml --target-dir build/userspace/ -- -C relocation-model=static -C no-redzone=on -C target-feature=-sse
.PHONY: build-x86_64
build-x86_64: $(x86_64_asm_object_files)
	mkdir -p build/kernel
	mkdir -p build/userspace/x86_64-unknown-none/debug/
	wget https://github.com/Daivuk/PureDOOM/raw/48376ddd6bbdb70085dab91feb1c6ceef80fa9b7/doom1.wad -N
	cd storage && sh generateExt2Img.sh && cd ..
	wget -P userland/src/doom/ https://raw.githubusercontent.com/Daivuk/PureDOOM/48376ddd6bbdb70085dab91feb1c6ceef80fa9b7/PureDOOM.h -N
	cargo rustc --manifest-path kernel/Cargo.toml --target-dir build/kernel/ -- -C no-redzone=on -C target-feature=-sse -C link-arg=-Ttargets/x86_64/linker.ld 
	gcc userland/src/doom/main.c userland/src/doom/libc.c -static -nostdlib -fno-builtin -g -o build/userspace/x86_64-unknown-none/debug/helloworld -Wl,--gc-sections
	mkdir -p dist/x86_64 && \
	objcopy --input binary --output elf64-x86-64 --binary-architecture i386 build/userspace/x86_64-unknown-none/debug/helloworld build/userspace/x86_64-unknown-none/debug/helloworld.o && \
	x86_64-elf-ld -n -o dist/x86_64/kernel.bin --unresolved-symbols=report-all -z noexecstack -T targets/x86_64/linker.ld $(x86_64_asm_object_files) build/kernel/x86_64-unknown-none/debug/libjos.a build/userspace/x86_64-unknown-none/debug/helloworld.o && \
	cp dist/x86_64/kernel.bin targets/x86_64/iso/boot/kernel.bin && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso
