#!/bin/bash
set -eux

mv /usr/include /usr/include-bak | true

cd usr

gcc -c -g -ffreestanding -fno-exceptions -fno-unwind-tables -fno-asynchronous-unwind-tables -static -nostdlib -fno-pic -fno-builtin --sysroot=/root/env/userland/ -I/root/env/userland/usr/include/ libc.c 
ar rcs libc.a *.o
mv libc.a lib/libc.a

cd ../dash-0.5.13/

export CFLAGS="--sysroot=/root/env/userland/ -I/root/env/userland/usr/include/ -g -fno-unwind-tables -fno-exceptions -fno-asynchronous-unwind-tables"
export CPPFLAGS="--sysroot=/root/env/userland/ -I/root/env/userland/usr/include/ -g -fno-unwind-tables -fno-exceptions -fno-asynchronous-unwind-tables"
export CPPFLAGS_FOR_BUILD="-I/root/env/userland/usr/include/ -fno-unwind-tables -g -fno-exceptions -fno-asynchronous-unwind-tables"

export LDFLAGS="-L/root/env/userland/usr/lib/ -g -static -nostdlib -fno-builtin -Wl,-e,_start"
export LOCAL_LDFLAGS="-L/root/env/userland/usr/lib/ -g -static -nostdlib -fno-builtin  -Wl,-e,_start"
export LDFLAGS_FOR_BUILD="-L/root/env/userland/usr/lib/ -g -static -nostdlib -fno-builtin  -Wl,-e,_start"
export LIBS="-lc"

if [ $# = 1 ] && [ "$1" = "-c" ]; then
	#./configure --host=x86_64-jos --prefix=/usr --without-bash-malloc --enable-static-link --disable-threads
	#make clean 
	./autogen.sh
	./configure --host=x86_64-jos --enable-static
fi

for file in Makefile src/Makefile; do
	if grep -q 'LDFLAGS = -static' "$file"; then
		sed -i 's/LDFLAGS = -static/LDFLAGS = -L\/root\/env\/userland\/usr\/lib\/ -g -static -nostdlib -fno-builtin -Wl,-e,_start/g' "$file"
		echo "Updated LDFLAGS in $file"
	else
		echo "LDFLAGS pattern not found in $file"
	fi
done

make clean
make V=1
