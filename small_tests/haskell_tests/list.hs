list = [1, 2, 3, 4, 5]

main = do
    print list

    print $ head list
    print $ tail list
    print $ last list
    print $ init list

    print $ list !! 3
    print $ elem 3 list

    print $ length list
    print $ null list
    print $ reverse list

    print $ take 2 list
    print $ drop 2 list

    print $ minimum list
    print $ maximum list

    print $ sum list
    print $ product list

    print [1..10]
    print ['A'..'Z']
    print [2,4..20]

    print $ take 10 $ cycle [1..4]
    print $ map (+1) list

    print $ filter (>3) list
    print $ all even list
    print $ any odd list

    print $ foldr (+) 0 list
    print $ foldr1 (+) list

    print $ foldl (+) 0 list
    print $ foldl1 (+) list

    print $ scanr (+) 0 list
    print $ scanr1 (+) list

    print $ scanl (+) 0 list
    print $ scanl1 (+) list

    print $ take 10 $ repeat 0
    print $ replicate 10 0
    print $ drop 3 list

    print $ ['a', 'b'] ++ ['c']
    print $ zip [1, 2, 3] ['a', 'b', 'c']
    print $ unzip [(1, 'a'), (2, 'b'), (3, 'c')]
    print $ zipWith (+) [1, 2, 3] [4, 5, 6]
    print $ [(x, y) | x <- [1..5], y <- ['a'..'e']]

    print $ words "Hello world"
    print $ unwords ["Hello", "world"] 
