

sed 's/^~"//' stacks.txt > stacks.tmp && mv stacks.tmp stacks.txt
sed -i 's/\\n/\n/g' stacks.txt
#sed -i '/^Thread\|^#/!d' stacks.txt

./FlameGraph/stackcollapse-gdb.pl stacks.txt > stacks.folded

./FlameGraph/flamegraph.pl stacks.folded > stacks.svg