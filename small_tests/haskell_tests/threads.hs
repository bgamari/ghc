import Control.Concurrent

main = do
    getNumCapabilities >>= print
    print rtsSupportsBoundThreads

    forkIO $ sequence_ $ replicate 3 $ do { print "Thread 1"; threadDelay 1 }
    forkIO $ sequence_ $ replicate 3 $ do { print "Thread 2"; threadDelay 1 }
    forkIO $ sequence_ $ replicate 3 $ do { print "Thread 3"; threadDelay 1 }
    threadDelay 10000
