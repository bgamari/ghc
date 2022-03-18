#!/bin/bash

# remove existing compiled files
if test -f "haskell_tests/$1"; then
    rm haskell_tests/$1 haskell_tests/$1.hi haskell_tests/$1.o
fi

# build
../_build/ghc-stage1 haskell_tests/$1.hs

# run
haskell_tests/$1
