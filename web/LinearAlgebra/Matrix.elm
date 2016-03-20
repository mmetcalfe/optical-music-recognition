module LinearAlgebra.Matrix2
  ( Matrix
  ) where

{-| comment

@docs Matrix

-}

import Array

{-| Matrix type -}
type alias Matrix =
  { shape: (Int, Int)
  , data: Array.Array Float
  }

empty : Matrix
empty = Matrix (0, 0) Array.empty

indexToLocation : (Int, Int) -> Int -> (Int, Int)
indexToLocation (rows, cols) i =
  let
    r = i // cols
    c = i % cols
  in
    (r, c)

identity : (Int, Int) -> Matrix
identity shape =
  let
    len = (fst shape) * (snd shape)
    isDiag i = (\(r,c) -> r == c) <| indexToLocation shape i
    identFill i = if isDiag i then 1 else 0
  in
    { shape = shape, data = Array.initialize len identFill}

fromLists : List (List Float) -> Maybe Matrix
fromLists a = Nothing

toLists : Matrix -> List (List Float)
toLists a = []


add : Matrix -> Matrix -> Matrix
add a b = empty

sub : Matrix -> Matrix -> Matrix
sub a b = empty

mul : Matrix -> Matrix -> Matrix
mul a b = empty

transpose : Matrix -> Matrix
transpose a = empty

neg : Matrix -> Matrix
neg a = empty

inv : Matrix -> Matrix
inv a = empty

qr : Matrix -> (Matrix, Matrix)
qr a = (empty,empty)

gaussJordan : Matrix -> Matrix
gaussJordan a = empty
