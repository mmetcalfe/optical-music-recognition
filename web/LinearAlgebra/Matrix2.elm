module LinearAlgebra.Matrix2
  ( Mat2, mat2, identity
  , transform
  , inverse, unsafeInverse, transpose
  , mul
  , makeRotate, makeScale
  , rotate, scale
  ) where

import Math.Vector2 exposing (Vec2)
import Math.Vector4 exposing (Vec4)

{-| 2x2 matrix type -}
type alias Mat2 = Vec4
{- Note: We represent the (row-major) matrix [[a, b], [c, d]] as the vector [a,
b, c, d]. -}

mat2 : ((Float, Float), (Float, Float)) -> Mat2
mat2 ((a, b), (c, d)) = Math.Vector4.fromTuple (a, b, c, d)

{-| Multiply a vector by a 2x2 matrix: m * v
-}
transform : Mat2 -> Vec2 -> Vec2
transform mat vec =
  let
    (a11, a12, a21, a22) = Math.Vector4.toTuple mat
    (x1, x2) = Math.Vector2.toTuple vec
  in
    Math.Vector2.fromTuple (a11*x1 + a12*x2, a21*x1 + a22*x2)


{-| A matrix with all 0s, except 1s on the diagonal.
-}
identity : Mat2
identity = mat2 ((1, 0), (0, 1))

{-| Computes the inverse of the given matrix m.
Returns `Nothing` if the matrix is not invertible.
-}
inverse : Mat2 -> Maybe Mat2
inverse mat =
  let
    (a, b, c, d) = Math.Vector4.toTuple mat
    det = a*d - b*c
  in
    if abs det < 1e-16
      then
        Nothing
      else
        let
          idet = 1 / (a*d - b*c)
        in
          Just <| Math.Vector4.fromTuple (d*idet, -b*idet, -c*idet, a*idet)

{-| Computes the inverse of the given matrix m.
Return value is undefined if the given matrix is not invertible.
-}
unsafeInverse : Mat2 -> Mat2
unsafeInverse mat =
  let
    (a, b, c, d) = Math.Vector4.toTuple mat
    idet = 1 / (a*d - b*c)
  in
    Math.Vector4.fromTuple (d*idet, -b*idet, -c*idet, a*idet)

{-| Matrix multiplcation: a * b
-}
mul : Mat2 -> Mat2 -> Mat2
mul matA matB =
  let
    (a11, a12, a21, a22) = Math.Vector4.toTuple matA
    (b11, b12, b21, b22) = Math.Vector4.toTuple matB
    r11 = a11*b11 + a12*b21
    r12 = a11*b12 + a12*b22
    r21 = a21*b11 + a22*b21
    r22 = a21*b12 + a22*b22
  in
    Math.Vector4.fromTuple (r11, r12, r21, r22)


{-| Creates a transformation matrix for rotation in radians about the
3-element vector axis.
-}
makeRotate : Float -> Mat2
makeRotate angle =
  let
    c = cos(angle)
    s = sin(angle)
  in
    mat2 ((c, -s), (s, c))

{-| Concatenates a rotation in radians about an axis to the given matrix.
-}
rotate : Float -> Mat2 -> Mat2
rotate angle mat = (makeRotate angle) `mul` mat

{-| Creates a transformation matrix for scaling each of the x, y, and z axes by
the amount given in the corresponding element of the 3-element vector.
-}
makeScale : Vec2 -> Mat2
makeScale vec =
  let
    (sx, sy) = Math.Vector2.toTuple vec
  in
    mat2 ((sx, 0), (0, sy))

{-| Concatenates a scaling to the given matrix.
-}
scale : Vec2 -> Mat2 -> Mat2
scale vec mat =
  (makeScale vec) `mul` mat

{-| "Flip" the matrix across the diagonal by swapping row index and column
index.
-}
transpose : Mat2 -> Mat2
transpose mat =
  let
    (a11, a12, a21, a22) = Math.Vector4.toTuple mat
  in
    Math.Vector4.fromTuple (a11, a21, a12, a22)
