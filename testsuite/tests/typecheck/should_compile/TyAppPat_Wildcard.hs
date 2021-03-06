{-# LANGUAGE TypeApplications #-}
{-# LANGUAGE GADTs #-}
{-# LANGUAGE ExistentialQuantification #-}
{-# LANGUAGE PartialTypeSignatures #-}

module Main where

f :: Maybe Int -> Int
f (Just @_ x) = x
f Nothing = 0

Just @_ x = Just "hello"

Just @Int y = Just 5

main = do
  print x
