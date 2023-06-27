# re-build MMTk
(cd rts/mmtk/mmtk && cargo build)
# re-build ghc
rm -rf _build/stage1/rts/
hadrian/build --flavour=default+debug_info -j8 -VV

# run fibo
_build/stage1/bin/ghc -fforce-recomp -mmtk -rtsopts -threaded -debug -g3 -Lrts/mmtk/mmtk/target/debug -optl-lmmtk_ghc fibo.hs
RUST_LOG=warn ./fibo 5000 +RTS -M5M

# run circsim
_build/stage1/bin/ghc -fforce-recomp -mmtk -rtsopts -threaded -debug -g3 -Lrts/mmtk/mmtk/target/debug -optl-lmmtk_ghc -inofib/gc/circsim Main
(cd nofib/gc/circsim/ &&  RUST_LOG=warn ./Main 8 20 +RTS -M50M)

# run fibheaps
_build/stage1/bin/ghc -fforce-recomp -mmtk -rtsopts -threaded -debug -g3 -Lrts/mmtk/mmtk/target/debug -optl-lmmtk_ghc -inofib/gc/fibheaps Main
(cd nofib/gc/fibheaps/ && RUST_LOG=warn ./Main 1000 +RTS -M20M)

# run nbody
_build/stage1/bin/ghc -fforce-recomp -mmtk -rtsopts -threaded -debug -g3 -Lrts/mmtk/mmtk/target/debug -optl-lmmtk_ghc -inofib/shootout/n-body Main
(cd nofib/shootout/n-body/ && RUST_LOG=warn ./Main 500000 +RTS -M50M)

# run ghc
# ./mmtk-build-ghc.sh

