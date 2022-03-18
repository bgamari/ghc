#!/bin/bash

for i in haskell_tests/*.hs; do
    filename=$(basename -- "$i")
    extension="${filename##*.}"
    filename="${filename%.*}"
    echo "--------------$filename--------------"
    ./test_one.sh $filename
done