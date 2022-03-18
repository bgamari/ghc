{-# LANGUAGE ForeignFunctionInterface #-}    
{-# LANGUAGE MagicHash #-}    
{-# LANGUAGE UnliftedFFITypes #-}    
{-# LANGUAGE CApiFFI #-}    
 
module Main(main) where    
 
import Foreign.C.Types    
import GHC.Exts    
import GHC.Ptr    
 
foreign import capi unsafe "printf" c_printf :: Ptr a -> CInt -> IO ()    
 
printInt :: CInt -> IO ()    
printInt n = c_printf (Ptr "hello world\n%d\n"#) n    
 
main :: IO ()    
main = do    
    printInt $ fromIntegral $ fib !! 42000    
 
fib :: [Int]    
fib = go 1 1    
  where    
    go n0 n1 = n0 : go n1 (n0+n1)  
