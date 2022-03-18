import Control.Concurrent
import System.Mem

main :: IO ()
main = do
    forkIO $ putStrLn "hello world"
    threadDelay $ 1000*1000
    performMajorGC
    threadDelay $ 1000*1000

