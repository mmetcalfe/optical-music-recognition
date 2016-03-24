module LinearAlgebra.Matrix
  ( Mat, zeroes, identity, matrix
  , fromLists, toLists
  , row, col, getSubmat
  , negate, transpose, inv
  , add, sub, mul, scale
  , qr
  , invUpperTri, newtonInverse
  , householder
  , joinRows, joinCols
  , solve
  ) where

{-| comment

@docs Mat

-}

import Array

unsafe : String -> Maybe a -> a
unsafe msg m =
    case m of
        Just v ->
            v
        _ ->
            Debug.crash msg

{-| Matrix type -}
type alias Mat =
  { shape: (Int, Int)
  , data: Array.Array Float
  }

-- Based on: http://hackage.haskell.org/package/bed-and-breakfast-0.4.3/docs/src/Numeric-Matrix.html#matrix
matrix : (Int, Int) -> ((Int, Int) -> Float) -> Mat
matrix shape f =
  let
    g = f << indexToLocation shape
    (rows, cols) = shape
  in
    { shape = shape, data = Array.initialize (rows*cols) g}

empty : Mat
empty = Mat (0, 0) Array.empty

indexToLocation : (Int, Int) -> Int -> (Int, Int)
indexToLocation (rows, cols) i =
  let
    r = i // cols
    c = i % cols
  in
    (r, c)

locationToIndex : (Int, Int) -> (Int, Int) -> Int
locationToIndex (rows, cols) (r, c) =
  r * cols + c

get : (Int, Int) -> Mat -> Maybe Float
get (r, c) m =
  Array.get (locationToIndex m.shape (r, c)) m.data

set : (Int, Int) -> Float -> Mat -> Maybe Mat
set (r, c) v m =
  let
    -- TODO: Fail if index is invalid.
    newData = Array.set (locationToIndex m.shape (r, c)) v m.data
  in
    Just {data = newData, shape = m.shape}

identity : (Int, Int) -> Mat
identity shape =
  let
    len = (fst shape) * (snd shape)
    isDiag i = (\(r,c) -> r == c) <| indexToLocation shape i
    identFill i = if isDiag i then 1 else 0
  in
    { shape = shape, data = Array.initialize len identFill}

zeroes : (Int, Int) -> Mat
zeroes shape =
  matrix shape (always 0)

fromLists : List (List Float) -> Maybe Mat
fromLists lists =
  let
    rows = List.length lists
    cols = case List.head lists of
      Just row -> List.length row
      Nothing -> 0
    data = Array.fromList (List.concat lists)
  in
    -- Check that each row was the same length:
    if List.all (\l -> List.length l == cols) lists
      then
        Just { shape = (rows, cols), data = data }
      else
        Nothing

toLists : Mat -> List (List Float)
toLists m =
  let
    (rows, cols) = m.shape
    fromMaybe = unsafe "toLists: matrix internals were inconsistent."
  in
    List.map (Array.toList << .data << fromMaybe << flip row m) [0..rows-1]

row : Int -> Mat -> Maybe Mat
row r m =
  let
    (rows, cols) = m.shape
    start = locationToIndex m.shape (r, 0)
    end = start + cols
    data = Array.slice start end m.data
  in
    if r >= 0 && r < rows
      then Just <| {data=data, shape=(1, cols)}
      else Nothing

col : Int -> Mat -> Maybe Mat
col c m =
  let
    fromMaybe = unsafe "toLists: matrix internals were inconsistent."
    (rows, cols) = m.shape
    getVal r = fromMaybe <| get (r, c) m
    data = Array.fromList <| List.map getVal [0..rows-1]
  in
    if c >= 0 && c < cols
      then Just <| {data=data, shape=(rows, 1)}
      else Nothing

getSubmat : Int -> Int -> Int -> Int -> Mat -> Maybe Mat
getSubmat startRow startCol endRow endCol m =
  let
    (rows, cols) = m.shape
    -- TODO: Treat startRow >= rows, etc. as errors.
    sr = startRow % rows
    sc = startCol % cols
    er = endRow % rows
    ec = endCol % cols
    indexIsInSubmat i =
      let
        (r, c) = indexToLocation m.shape i
      in
        sr <= r && r <= er && sc <= c && c <= ec
    indexedData = Array.indexedMap (\i v -> (i, v)) m.data
    filteredData = Array.filter (\(i, v) -> indexIsInSubmat i) indexedData
    data = Array.map (\(i, v) -> v) filteredData
    newShape = (er - sr + 1, ec - sc + 1)
    rangeIsValid (rmin, rmax) (r1, r2) =
      rmin <= r1 && r1 <= r2 && r2 < rmax
  in
    if rangeIsValid (0, rows) (sr, er) && rangeIsValid (0, cols) (sc, ec)
      then
        Just {data=data, shape=newShape}
      else
        Nothing

joinCols : Mat -> Mat -> Maybe Mat
joinCols m1 m2 =
  let
    (rows1, cols1) = m1.shape
    (rows2, cols2) = m2.shape
    data = Array.append m1.data m2.data
  in
    if cols1 == cols2
      then
        Just {data=data, shape=(rows1+rows2, cols1)}
      else
        Nothing

joinRows : Mat -> Mat -> Maybe Mat
joinRows m1 m2 =
  -- TODO: Use a more efficient method.
  let
    m1t = transpose m1
    m2t = transpose m2
    m1m2t = joinCols m1t m2t
  in
    Maybe.map transpose m1m2t

negate : Mat -> Mat
negate a =
  { data = Array.map Basics.negate a.data, shape = a.shape }

add : Mat -> Mat -> Maybe Mat
add a b =
  let
    fromMaybe = unsafe "add: matrix internals were inconsistent."
    addToB i v = v + fromMaybe (Array.get i b.data)
  in
    if a.shape == b.shape
      then Just <| { data = Array.indexedMap addToB a.data, shape=a.shape}
      else Nothing

sub : Mat -> Mat -> Maybe Mat
sub a b = add a (negate b)

transpose : Mat -> Mat
transpose a =
  let
    fromMaybe = unsafe "transpose: matrix internals were inconsistent."
    (r, c) = a.shape
  in
    matrix (c, r) (\(r, c) -> fromMaybe <| get (c, r) a)
  -- let
  --   fromMaybe = unsafe "transpose: matrix internals were inconsistent."
  --   (r, c) = a.shape
  --   newShape = (c, r)
  --   transposedIndex i =
  --     let
  --       (r, c) = indexToLocation a.shape i
  --     in
  --       locationToIndex newShape (c, r)
  --   swap i _ = fromMaybe <| Array.get (transposedIndex i) a.data
  --   data = Array.indexedMap swap a.data
  -- in
  --   { data = data, shape = newShape }

scale : Float -> Mat -> Mat
scale f m =
  { data = Array.map (\v -> f*v) m.data, shape = m.shape }

mul : Mat -> Mat -> Maybe Mat
mul a b =
  let
    dot l1 l2 = List.sum <| List.map2 (\v1 v2 -> v1 * v2) l1 l2

    (ar, ac) = a.shape
    (br, bc) = b.shape
    bt = transpose b
    rowsA = toLists a
    colsB = toLists bt
    listData = List.map (\rowA -> List.map (dot rowA) colsB) rowsA
  in
    if ac == br
      then
        fromLists listData
      else
        Nothing

inv : Mat -> Maybe Mat
inv a =
  case qr a of
    Just (q, r) ->
      Maybe.andThen (invUpperTri r) (\rinv -> rinv `mul` (transpose q))
    _ ->
      Nothing



{-| Improve an estimate x of the inverse of the matrix a.
-}
invNewtonIteration : Mat -> Mat -> Mat
invNewtonIteration a x =
  let
    fromMaybe = unsafe "invNewtonIteration: matrix internals were inconsistent."
    twox = scale 2 x
    xa = fromMaybe <| x `mul` a
    xax = fromMaybe <| xa `mul` x
  in
    fromMaybe <| twox `sub` xax

newtonInverse : Int -> Mat -> Mat -> Mat
newtonInverse n m x =
  let
    f _ a = invNewtonIteration m a
  in
    List.foldl f x [1..n]

{-| Multiply row i by s.
    scaleRow i s m
-}
scaleRow : Int -> Float -> Mat -> Maybe Mat
scaleRow r s m =
  let
    fromMaybe = unsafe "scaleRow: matrix internals were inconsistent."
    (rows, cols) = m.shape
    scaleElem c x =
      let
        sv = fromMaybe <| get (r, c) x
      in
        fromMaybe <| set (r, c) (s * sv) x
  in
    Just <| List.foldl scaleElem m [0..cols-1]

{-| Add row i multiplied by s to row j
    addScaledRowToRow i s j m
-}
addScaledRowToRow : Int -> Float -> Int -> Mat -> Maybe Mat
addScaledRowToRow r1 s r2 m =
  let
    fromMaybe = unsafe "addScaledRowToRow: matrix internals were inconsistent."
    (rows, cols) = m.shape
    addScaledElem c x =
      let
        sv = fromMaybe <| get (r1, c) x
        dv = fromMaybe <| get (r2, c) x
      in
        fromMaybe <| set (r2, c) (dv + s * sv) x
  in
    Just <| List.foldl addScaledElem m [0..cols-1]

{-| Compute the inverse of the given upper triangular matrix.
-}
invUpperTri : Mat -> Maybe Mat
invUpperTri u =
  let
    fromMaybe = unsafe "invUpperTri: matrix internals were inconsistent."
    (rows, cols) = u.shape
    res = identity u.shape
    -- invElem (r, c) =
    --   let
    --     d = fromMaybe <| get (r, r) u
    --     v = fromMaybe <| get (r, c) u
    --   in
    --     if r > c
    --       then 0
    --       else
    --         if r == c
    --           then 1 / d
    --           else -v / d
    cancelElem r c m =
      let
        v = fromMaybe <| get (r, c) u
      in
        if r == c
          then
            fromMaybe <| scaleRow r (1/v) m
          else
            if c > r
              then
                fromMaybe <| addScaledRowToRow c (-v) r m
              else
                m

    reduceRow i m =
      -- e.g. h t1 t2 | 1 0 0
      --      0  1  0 | 0 b c
      --      0  0  1 | 0 0 a
      -- First cancel each t, then divide by h.
      -- e.g. 1  0  0 | 1/h -t1*b/h (-t1*c-t2*a)/h
      --      0  1  0 | 0       b        c
      --      0  0  1 | 0       0        a
        List.foldl (cancelElem i) m (List.reverse [i..rows-1])
  in
    -- TODO: If any diagonals are zero, return Nothing.
    if rows == cols
      then
        Just <| List.foldl reduceRow res (List.reverse [0..rows-1])
        -- Just <| matrix u.shape invElem
      else
        Nothing

length : Mat -> Maybe Float
length x =
  let
    (rows, cols) = x.shape
  in
    if rows > 0 && cols == 1
      then
        Just <| sqrt <| Array.foldl (\v s -> s + v*v) 0 x.data
      else
        Nothing

normalise : Mat -> Maybe Mat
normalise x =
  let
    ml = length x
  in
    Maybe.map (\l -> scale(1/l) x) ml

sign : Float -> Float
sign x =
  if x == 0
    then 0
    else
      if x > 0
        then 1
        else -1

{-| Return the householder matrix that transforms x into [a, 0, ..., 0].
-}
householder : Mat -> Maybe Mat
householder x =
  let
    -- TODO: Handle failure cases correctly.
    fromMaybe = unsafe "householder: x was not a valid column matrix."
    (rows, cols) = x.shape
    asign = -(sign (fromMaybe <| get (0, 0) x)) -- using this sign avoids loss of significance
    a = asign * (fromMaybe <| length x)
    ae1 = matrix (rows, 1) (\(r, _) -> if r == 0 then a else 0)
    u = fromMaybe <| x `sub` ae1
    v = fromMaybe <| normalise u
    vt = transpose v
    q = (identity (rows, rows)) `sub` (2 `scale` (fromMaybe <| v `mul` vt))
  in
    q

{-| Perform a qr decomposition of the matrix
-}
qrStep : Int -> (Mat, Mat) -> (Mat, Mat)
qrStep i (cumQ, cumR) =
  let
    (rows, cols) = cumR.shape
    ai = unsafe "qrStep: submat" <| getSubmat i i -1 -1 cumR
    x = unsafe "qrStep: x" <| col 0 ai
    tl = identity (i, i)
    tr = zeroes (i, rows-i)
    bl = zeroes (rows-i, i)
    br = unsafe "qrStep: householder" <| householder x
    testbr = br `mul` ai
    testbr2 = br `mul` (transpose br)
    leftQ = unsafe "qrStep: ql" <| tl `joinCols` bl
    rightQ = unsafe "qrStep: qr" <| tr `joinCols` br
    qi = unsafe "qrStep: qi" <| joinRows leftQ rightQ
    testQi = qi `mul` (transpose qi)
    resultQ = unsafe "qrStep: qc" <| cumQ `mul` (transpose qi)
    testQ = resultQ `mul` (transpose resultQ)
    resultR = unsafe "qrStep: rc" <| qi `mul` cumR
    testQR = resultQ `mul` resultR
  in
    (resultQ, resultR)

qr : Mat -> Maybe (Mat, Mat)
qr m =
  let
    (rows, cols) = m.shape
    initQ = identity (rows, rows)
    -- t = min (rows-2) (cols-1)
    t = min (rows-1) (cols)
  in
    Just <| List.foldl qrStep (initQ, m) [0..t]

{-| Solves the system `ax = b` for matrix `a` and column vector `b`.
Returns a least squares solution if `a` is non-square

    let
      a = fromLists [[1,2,3], [4,5,6], [7,8,9], [10,11,12]]
      b = fromLists [0.1, -0.2, 0.3]
    in
      x = solve a b
-}
solve : Mat -> Mat -> Maybe Mat
-- See: https://en.wikipedia.org/wiki/QR_decomposition#Using_for_solution_to_linear_inverse_problems
-- TODO: Use backward/forward substitution rather than calculating the inverse.
solve a b =
  let
    (rows, cols) = a.shape
  in
    if rows < cols
      then -- The problem is underdetermined
        let
          at = transpose a
          (atq, atr) = unsafe "solve (m<n): atqr" <| qr at
          r1 = unsafe "solve (m<n): r1" <| getSubmat 0 0 (rows-1) (rows-1) atr
          r1Inv = unsafe "solve (m<n): r1Inv" <| invUpperTri r1
          rZeroes = zeroes ((cols - rows), rows)
          rInv = unsafe "solve (m<n): rInv" <| r1Inv `joinCols` rZeroes
          pseudoInv = unsafe "solve (m<n): pseudoInv" <| atq `mul` rInv
        in
          mul pseudoInv b
      else -- The problem is fully determined or overdetermined
        let
          (aq, ar) = Debug.log "aq, ar" <| unsafe "solve (m>=n): aqr" <| qr a
          q1 = Debug.log "q1" <| unsafe "solve (m>=n): q1" <| getSubmat 0 0 (rows-1) (cols-1) aq
          r1 = Debug.log "r1" <| unsafe "solve (m>=n): r1" <| getSubmat 0 0 (cols-1) (cols-1) ar
          r1Inv = Debug.log "r1Inv" <| unsafe "solve (m>=n): r1Inv" <| invUpperTri r1
          q1t = Debug.log "q1t" <| transpose q1
          pseudoInv = Debug.log "pseudoInv" <| unsafe "solve (m>=n): pseudoInv" <| r1Inv `mul` q1t
        in
          mul pseudoInv b
