module LinearAlgebra.Matrix.Unsafe
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
  , length, normalise
  ) where

{-| Unsafe version of the Matrix API.

Functions do not return `Maybe`s or `Result`s, but will cause a `Debug.crash` if they fail.

These are intended to be easier/faster to work with when safety is not a concern.

Note that these methods can be used in combination with the regular safe methods.
-}

import LinearAlgebra.Matrix as Matrix

unsafe : String -> Maybe a -> a
unsafe msg m =
    case m of
        Just v ->
            v
        _ ->
            Debug.crash ("LinearAlgebra.Matrix.Unsafe." ++ msg)

{-| Re-export of Matrix.Mat.
-}
type alias Mat = Matrix.Mat

-- SAFE METHODS:

{-| Re-export of Matrix.matrix.
-}
matrix : (Int, Int) -> ((Int, Int) -> Float) -> Mat
matrix shape f = Matrix.matrix shape f

{-| Re-export of Matrix.identity.
-}
identity : (Int, Int) -> Mat
identity shape = Matrix.identity shape

{-| Re-export of Matrix.zeroes.
-}
zeroes : (Int, Int) -> Mat
zeroes shape = Matrix.zeroes shape

{-| Re-export of Matrix.toLists.
-}
toLists : Mat -> List (List Float)
toLists m = Matrix.toLists m

{-| Re-export of Matrix.negate.
-}
negate : Mat -> Mat
negate a = Matrix.negate a

{-| Re-export of Matrix.transpose.
-}
transpose : Mat -> Mat
transpose a = Matrix.transpose a

{-| Re-export of Matrix.scale.
-}
scale : Float -> Mat -> Mat
scale f m = Matrix.scale f m

{-| Re-export of Matrix.newtonInverse.
-}
newtonInverse : Int -> Mat -> Mat -> Mat
newtonInverse n m x = Matrix.newtonInverse n m x


-- UNSAFE METHODS:

{-| Unsafe version of Matrix.get. Causes a Debug.crash on error.
-}
get : (Int, Int) -> Mat -> Float
get (r, c) m = unsafe "get" <| Matrix.get (r, c) m

{-| Unsafe version of Matrix.set. Causes a Debug.crash on error.
-}
set : (Int, Int) -> Float -> Mat -> Mat
set (r, c) v m = unsafe "set" <| Matrix.set (r, c) v m

{-| Unsafe version of Matrix.fromLists. Causes a Debug.crash on error.
-}
fromLists : List (List Float) -> Mat
fromLists lists = unsafe "fromLists" <| Matrix.fromLists lists

{-| Unsafe version of Matrix.row. Causes a Debug.crash on error.
-}
row : Int -> Mat -> Mat
row r m = unsafe "row" <| Matrix.row r m

{-| Unsafe version of Matrix.col. Causes a Debug.crash on error.
-}
col : Int -> Mat -> Mat
col c m = unsafe "col" <| Matrix.col c m

{-| Unsafe version of Matrix.getSubmat. Causes a Debug.crash on error.
-}
getSubmat : Int -> Int -> Int -> Int -> Mat -> Mat
getSubmat startRow startCol endRow endCol m = unsafe "getSubmat" <| Matrix.getSubmat startRow startCol endRow endCol m

{-| Unsafe version of Matrix.joinCols. Causes a Debug.crash on error.
-}
joinCols : Mat -> Mat -> Mat
joinCols m1 m2 = unsafe "joinCols" <| Matrix.joinCols m1 m2

{-| Unsafe version of Matrix.joinRows. Causes a Debug.crash on error.
-}
joinRows : Mat -> Mat -> Mat
joinRows m1 m2 = unsafe "joinRows" <| Matrix.joinRows m1 m2

{-| Unsafe version of Matrix.add. Causes a Debug.crash on error.
-}
add : Mat -> Mat -> Mat
add a b = unsafe "add" <| Matrix.add a b

{-| Unsafe version of Matrix.sub. Causes a Debug.crash on error.
-}
sub : Mat -> Mat -> Mat
sub a b = unsafe "sub" <| Matrix.sub a b

{-| Unsafe version of Matrix.mul. Causes a Debug.crash on error.
-}
mul : Mat -> Mat -> Mat
mul a b = unsafe "mul" <| Matrix.mul a b

{-| Unsafe version of Matrix.inv. Causes a Debug.crash on error.
-}
inv : Mat -> Mat
inv a = unsafe "inv" <| Matrix.inv a

{-| Unsafe version of Matrix.invUpperTri. Causes a Debug.crash on error.
-}
invUpperTri : Mat -> Mat
invUpperTri u = unsafe "invUpperTri" <| Matrix.invUpperTri u

{-| Unsafe version of Matrix.length. Causes a Debug.crash on error.
-}
length : Mat -> Float
length x = unsafe "length" <| Matrix.length x

{-| Unsafe version of Matrix.normalise. Causes a Debug.crash on error.
-}
normalise : Mat -> Mat
normalise x = unsafe "normalise" <| Matrix.normalise x

{-| Unsafe version of Matrix.householder. Causes a Debug.crash on error.
-}
householder : Mat -> Mat
householder x = unsafe "householder" <| Matrix.householder x

{-| Unsafe version of Matrix.qr. Causes a Debug.crash on error.
-}
qr : Mat -> (Mat, Mat)
qr m = unsafe "qr" <| Matrix.qr m

{-| Unsafe version of Matrix.solve. Causes a Debug.crash on error.
-}
solve : Mat -> Mat -> Mat
solve a b = unsafe "solve" <| Matrix.solve a b
