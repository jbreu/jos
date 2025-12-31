x86_64_asm_source_files := $(shell find kernel/asm -name *.asm)
x86_64_asm_object_files := $(patsubst kernel/asm/%.asm, build/x86_64/%.o, $(x86_64_asm_source_files))

$(x86_64_asm_object_files): build/x86_64/%.o : kernel/asm/%.asm
	mkdir -p $(dir $@) && \
	nasm -f elf64 -g -F dwarf $(patsubst build/x86_64/%.o, kernel/asm/%.asm, $@) -o $@

.PHONY: setup
setup: $(x86_64_asm_object_files)
	mkdir -p build/kernel && \
	mkdir -p build/userspace/x86_64-unknown-none/debug/ && \
	mkdir -p dist/x86_64 && \
	test -f userland/src/doom/doom1.wad || wget https://github.com/Daivuk/PureDOOM/raw/48376ddd6bbdb70085dab91feb1c6ceef80fa9b7/doom1.wad -O userland/src/doom/doom1.wad && \
	test -f userland/src/doom/PureDOOM.h || wget -P userland/src/doom/ https://raw.githubusercontent.com/Daivuk/PureDOOM/48376ddd6bbdb70085dab91feb1c6ceef80fa9b7/PureDOOM.h -N

.PHONY: userland
userland:
	gcc userland/src/doom/main.c userland/src/doom/libc.c -static -nostdlib -fno-builtin -g -o build/userspace/x86_64-unknown-none/debug/doom -Wl,--gc-sections --sysroot=/root/env/userland/src/doom && \
	test -f dash-0.5.13.tar.gz || wget https://git.kernel.org/pub/scm/utils/dash/dash.git/snapshot/dash-0.5.13.tar.gz -N && \
	tar -xzf dash-0.5.13.tar.gz -C userland/ && \
	cd userland && ./build_dash.sh -c && cd .. && \
	cd storage && sh generateExt2Img.sh && cd ..

.PHONY: kernel
kernel:
	cargo rustc --manifest-path kernel/Cargo.toml --target-dir build/kernel/ -- -C no-redzone=on -C target-feature=-sse -C link-arg=-Ttargets/x86_64/linker.ld 
	$(MAKE) iso

.PHONY: iso
iso: 
	x86_64-elf-ld -n -o dist/x86_64/kernel.bin --unresolved-symbols=report-all -z noexecstack -T targets/x86_64/linker.ld $(x86_64_asm_object_files) build/kernel/x86_64-unknown-none/debug/libjos.a && \
	cp dist/x86_64/kernel.bin targets/x86_64/iso/boot/kernel.bin && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso
	
.PHONY: all
all: setup userland kernel