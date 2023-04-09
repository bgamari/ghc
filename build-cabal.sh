#!/usr/bin/env bash

set -e -x
out=_cabal_out

if [ -n "$CABAL_DIR" ]; then
    dir="$CABAL_DIR"
else
    dir="libraries/Cabal"
fi

if [[ -z "$GHC" ]]; then
  GHC="_build/stage1/bin/ghc"
fi

if [[ -z "$WRAPPER" ]]; then
  WRAPPER="time"
fi

rm -Rf $out
export GHC_ENVIRONMENT="-"
exec $WRAPPER $GHC -package parsec -XHaskell2010 -hidir $out -odir $out -i$dir/Cabal -i$dir/Cabal-syntax/src -i$dir/Cabal/src $dir/Cabal/Setup.hs +RTS -M10G -s -RTS $@
