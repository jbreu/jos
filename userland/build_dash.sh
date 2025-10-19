#!/bin/bash
set -eux

mv /usr/include /usr/include-bak | true

cd usr

gcc -c -ffreestanding -fno-exceptions -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-rtti libc.c 
ar rcs libc.a *.o
mv libc.a lib/libc.a

cd ../dash-0.5.13/

export CFLAGS="--sysroot=/root/env/userland/ -I/root/env/userland/usr/include/ -fno-unwind-tables -fno-exceptions -fno-asynchronous-unwind-tables"
export CPPFLAGS="--sysroot=/root/env/userland/ -I/root/env/userland/usr/include/ -fno-unwind-tables -fno-exceptions -fno-asynchronous-unwind-tables"
export CPPFLAGS_FOR_BUILD="-I/root/env/userland/usr/include/ -fno-unwind-tables -fno-exceptions -fno-asynchronous-unwind-tables"

export LDFLAGS="-L/root/env/userland/usr/lib/ -static -nostdlib -fno-builtin"
export LOCAL_LDFLAGS="-L/root/env/userland/usr/lib/ -static -nostdlib -fno-builtin"
export LDFLAGS_FOR_BUILD="-L/root/env/userland/usr/lib/ -static -nostdlib -fno-builtin"
export LIBS="-lc"

if [ $# = 1 ] && [ "$1" = "-c" ]; then
	#./configure --host=x86_64-jos --prefix=/usr --without-bash-malloc --enable-static-link --disable-threads
	./configure --host=x86_64-jos --enable-static
	make clean 
fi

# provide a sed script which replaces LDFLAGS = -static -Wl,--fatal-warnings in Makefile with above LDFLAGS
for file in Makefile src/Makefile; do
	if grep -q 'LDFLAGS = -static' "$file"; then
		sed -i 's/LDFLAGS = -static/LDFLAGS = -L\/root\/env\/userland\/usr\/lib\/ -static -nostdlib -fno-builtin/g' "$file"
		echo "Updated LDFLAGS in $file"
	else
		echo "LDFLAGS pattern not found in $file"
	fi
done

#make clean
make V=1
