# re-build MMTk
(cd rts/mmtk/mmtk && cargo build)
# re-build ghc
# rm -rf _build/stage1/rts/
hadrian/build --flavour=default+debug_info -j8 -VV

# run fibo
_build/stage1/bin/ghc -fforce-recomp -mmtk -rtsopts -threaded -debug -g3 -Lrts/mmtk/mmtk/target/debug -optl-lmmtk_ghc fibo.hs

./fibo 5000 +RTS -M5M