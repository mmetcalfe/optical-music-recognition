module LinearAlgebra.Matrix
  ( Mat, identity
  , fromLists, toLists
  , row
  , negate, transpose, inv
  , add, sub, mul, scale
  , qr, gaussJordan
  , invUpperTri, newtonInverse
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
    List.map (Array.toList << fromMaybe << flip row m) [0..rows-1]

row : Int -> Mat -> Maybe (Array.Array Float)
row r m =
  let
    (rows, cols) = m.shape
    start = locationToIndex m.shape (r, 0)
    end = start + cols
  in
    if r >= 0 && r < rows
      then Just <| Array.slice start end m.data
      else Nothing

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
    newShape = (c, r)
    transposedIndex i =
      let
        (r, c) = indexToLocation a.shape i
      in
        locationToIndex newShape (c, r)
    swap i _ = fromMaybe <| Array.get (transposedIndex i) a.data
    data = Array.indexedMap swap a.data
  in
    { data = data, shape = newShape }

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

inv : Mat -> Mat
inv a = empty

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



qr : Mat -> (Mat, Mat)
qr a = (empty,empty)

gaussJordan : Mat -> Mat
gaussJordan a = empty
