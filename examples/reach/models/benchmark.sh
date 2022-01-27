#!/bin/bash

# Perform the benchmarks for reach.
BENCHMARKS=()
for file in *.ldd
do
  BENCHMARKS+=("reach $file")
done

hyperfine --warmup 3 "${BENCHMARKS[@]/#}"

# Perform the benchmarks for lddmc
BENCHMARKS=()
for file in *.ldd
do
  BENCHMARKS+=("lddmc -w1 -sbfs $file")
done

hyperfine --warmup 3 "${BENCHMARKS[@]/#}"