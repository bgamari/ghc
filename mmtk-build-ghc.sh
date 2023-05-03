set -e 

(cd rts/mmtk/mmtk && cargo build)

rm -rf _build/stage1/rts/
rm _build/stage1/bin/ghc
hadrian/build --flavour=default+debug_info -j8 stage1.ghc-bin.ghc.link.opts+="-mmtk -debug" -VV
GHC=_build/stage1/bin/ghc bash build-cabal.sh
