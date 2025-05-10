# Remove leading '~"' from each line
sed 's/^~"//' stacks.txt > stacks.tmp && mv stacks.tmp stacks.txt

# Replace '\n' literal text with actual newlines
sed -i 's/\\n/\n/g' stacks.txt

./FlameGraph/stackcollapse-gdb.pl stacks.txt > stacks.folded

# Remove stack traces that only contain unknown symbols ('??;??;')
sed -i '/^??;??;/d' stacks.folded

./FlameGraph/flamegraph.pl stacks.folded > stacks.svg