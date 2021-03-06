{-# LANGUAGE TypeApplications #-}
{-# LANGUAGE GADTs #-}
{-# LANGUAGE ExistentialQuantification #-}

module Main where

data Foo where
  MkFoo :: forall a. a -> (a -> String) -> Foo

foo :: Foo -> String
foo (MkFoo @a x f) = f (x :: a)

main = do
  print (foo (MkFoo "hello" reverse))
