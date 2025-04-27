#!/bin/bash
set -eux

mv /usr/include /usr/include-bak | true

cd usr

gcc -c *.c
ar rcs libc.a *.o
mv libc.a lib

cd ../bash

export CFLAGS="--sysroot=/root/env/userland/ -I/root/env/userland/usr/include/"
export CPPFLAGS="--sysroot=/root/env/userland/ -I/root/env/userland/usr/include/"
export CPPFLAGS_FOR_BUILD="-I/root/env/userland/usr/include/"

# TODO reactivate for jos build
#export LDFLAGS="-L/root/env/userland/usr/lib -static -nostdlib -fno-builtin -Wl,--trace"
#export LOCAL_LDFLAGS="-L/root/env/userland/usr/lib -static -nostdlib -fno-builtin -Wl,--trace"
#export LDFLAGS_FOR_BUILD="-L/root/env/userland/usr/lib -static -nostdlib -fno-builtin -Wl,--trace"

# Build tools on linux
export LDFLAGS="-nostdlib -fno-builtin -Wl,--trace"
export LOCAL_LDFLAGS=" -Wl,--trace"
export LDFLAGS_FOR_BUILD="-Wl,--trace"

sed -i '/^LIBS_FOR_BUILD =/c\LIBS_FOR_BUILD = @LIBS_FOR_BUILD@' Makefile.in
#export LIBS_FOR_BUILD="-lc"


# x86_64-jos: to make it work, add everywhere in configure, config.sub and config.guess where haiku is mentioned a similar entry for jos

if [ $# = 1 ] && [ "$1" = "-c" ]; then
	#./configure --host=x86_64-jos --prefix=/usr --without-bash-malloc --enable-static-link --disable-threads
	./configure --disable-minimal-config  --disable-alias  --disable-alt-array-implementation  --disable-arith-for-command  --disable-array-variables  --disable-bang-history  --disable-brace-expansion  --disable-casemod-attributes  --disable-casemod-expansions  --disable-command-timing  --disable-cond-command  --disable-cond-regexp  --disable-coprocesses  --disable-debugger  --disable-dev-fd-stat-broken  --disable-direxpand-default  --disable-directory-stack  --disable-disabled-builtins  --disable-dparen-arithmetic  --disable-extended-glob  --disable-extended-glob-default  --disable-function-import  --disable-glob-asciiranges-default  --disable-help-builtin  --disable-history  --disable-job-control  --disable-multibyte  --disable-net-redirections  --disable-process-substitution  --disable-progcomp  --disable-prompt-string-decoding  --disable-readline  --disable-restricted  --disable-select  --disable-separate-helpfiles  --disable-single-help-strings  --disable-strict-posix-default  --disable-translatable-strings  --disable-usg-echo-default  --disable-xpg-echo-default  --disable-mem-scramble  --disable-profiling  --disable-static-link  --disable-largefile  --disable-nls  --disable-threads  --disable-rpath  --without-bash-malloc --disable-threads --host=x86_64-pc-linux-gnu #--enable-static-link
fi

echo "Building ~/env/userland/bash/builtins"
(cd ~/env/userland/bash/builtins && make clean && make)
find ~/env/userland/bash/builtins -name '*.a' -exec cp {} ~/env/userland/usr/lib/ \;

#for each folder in /env/userland/bash/lib/ run make and then copy the .a file to /env/userland/usr/lib/
for dir in ~/env/userland/bash/lib/*; do
	echo "Building $dir"
    (cd "$dir" && make clean && make)
    find "$dir" -name '*.a' -exec cp {} ~/env/userland/usr/lib/ \;
done


make clean
make 