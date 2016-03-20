module LinearAlgebra.TransformSE2
  ( TransSE2, transSE2, identity, fromComponents
  , getX, getY, getAngle, setAngle, setX, setY, getXY, setXY
  , apply, compose
  , inverse, rotate, translate
  , makeRotate, makeTranslate
  -- , toSquareMatrix
  , localToWorld, worldToLocal
  ) where

{-| Transforms in SE(2): rotations and translations in the 2D plane.

This library provides a type `TransSE2` for representing transforms consisting of rotations and translation in 2D.

A `TransSE2` is equivalent to a 2x3 matrix of the form [R|t] where R is a 2x2 rotation matrix, and t is a 2x1 vector specifying a translation.

This type is useful, as these kinds of transforms are very common in 2D graphics applications.

The set of all such transforms is called the [special Euclidian group in 2 dimensions](https://en.wikipedia.org/wiki/Euclidean_group#Direct_and_indirect_isometries) denoted SE(2).

# Create

@docs TransSE2, transSE2, identity, fromComponents

# Get and Set

The set functions create a new copy of the vector, updating a the necessary fields.
@docs getX, getY, getAngle, setAngle, setX, setY, getXY, setXY

# Operations

@docs apply, compose

# Apply Transformations

@docs inverse, rotate, translate

# Create Transformations

@docs makeRotate, makeTranslate

# Conversions


# Helper functions

Helper functions that can make transforms easier to reason about when they are used to represent coordinate system bases.

@docs localToWorld, worldToLocal

-}

import Math.Vector2 exposing (Vec2)
import Math.Vector3 exposing (Vec3)

import LinearAlgebra.Matrix2 as Matrix2
import LinearAlgebra.Angle as Angle

{-| Type of a transform in SE(2) -}
type alias TransSE2 = Vec3
-- type TransSE2 =
--   TransSE2
--     { x : Float
--     , y : Float
--     , angle : Float
--     }

{-| Creates a new transform that performs the given translation and rotation.

    trans = transSE2 x y angle
-}
transSE2 : Float -> Float -> Float -> TransSE2
transSE2 x y angle = Math.Vector3.vec3 x y angle

{-| Creates a new transform that performs the given translation and rotation.

    trans = fromComponents translation angle
-}
fromComponents : Vec2 -> Float -> TransSE2
fromComponents trans angle =
  let
    (x, y) = Math.Vector2.toTuple trans
  in
    transSE2 x y angle

{-| The identity transform -}
identity : TransSE2
-- identity = TransSE2 {x = 0, y = 0, angle = 0}
identity = transSE2 0 0 0

{-| Extract the x translation of a transform. -}
getX : TransSE2 -> Float
getX = Math.Vector3.getX

{-| Extract the y translation of a transform. -}
getY : TransSE2 -> Float
getY = Math.Vector3.getY

{-| Extract the rotation angle of a transform. -}
getAngle : TransSE2 -> Float
getAngle = Math.Vector3.getZ

{-| Extract the translation vector of a transform. -}
getXY : TransSE2 -> Vec2
getXY t = Math.Vector2.vec2 (getX t) (getY t)

{-| Update the x translation of a transform, returning a new transform. -}
setX : Float -> TransSE2 -> TransSE2
setX = Math.Vector3.setX

{-| Update the y translation of a transform, returning a new transform. -}
setY : Float -> TransSE2 -> TransSE2
setY = Math.Vector3.setY

{-| Update the rotation angle of the transform, returning a new transform. -}
setAngle : Float -> TransSE2 -> TransSE2
setAngle = Math.Vector3.setZ

{-| Set the translation component of the transform to the given vector. -}
setXY : Vec2 -> TransSE2 -> TransSE2
setXY xy vec =
    fromComponents xy (getAngle vec)

{-| Applies the given transform to the given vector.

    result = apply trans vec
-}
apply : TransSE2 -> Vec2 -> Vec2
apply t v =
  let
    b = fromComponents v 0
    r = compose t b
  in
    getXY r

{-| Returns the transform that applies both transforms in sequence:
`compose a b` == a * b.
-}
compose : TransSE2 -> TransSE2 -> TransSE2
compose a b =
  let
    at = getAngle a
    bt = getAngle b
    rotA = Matrix2.makeRotate at
    rv = (getXY a) `Math.Vector2.add` (Matrix2.transform rotA (getXY b))
    rt = Angle.normaliseAngle (at+bt)
  in
    fromComponents rv rt


{-| Transform `localBasis` represented in local coordinates of `basis` into world coordinates.

    worldSpaceBasis = localToWorld basis localBasis

(Note that `localToWorld = compose`)
-}
localToWorld : TransSE2 -> TransSE2 -> TransSE2
localToWorld basis localBasis =
  basis `compose` localBasis

{-| Transform `worldBasis` represented in world coordinates into local coordinates of `basis`.

    localBasis = worldToLocal basis worldBasis

Note that this is simply composition with the inverse of `basis`:

    worldToLocal basis worldBasis = compose (inverse basis) worldBasis
-}
worldToLocal : TransSE2 -> TransSE2 -> TransSE2
worldToLocal basis worldBasis =
  (inverse basis) `compose` worldBasis

{-| Returns the inverse of the given transformation.

    identity = compose t (inverse t) = compose (inverse t) t
-}
inverse : TransSE2 -> TransSE2
inverse a =
  let
    at = getAngle a
    invRotA = Matrix2.makeRotate -at
    rv = Math.Vector2.negate (Matrix2.transform invRotA (getXY a))
    rt = -at
  in
    fromComponents rv rt

{-| Returns a transform that applies a rotation after applying the given transform.
-}
rotate : TransSE2 -> Float -> TransSE2
rotate t angle =
  (makeRotate angle) `compose` t

{-| Returns a transform that applies a translation after applying the given transform.
-}
translate : TransSE2 -> Vec2 -> TransSE2
translate t vec =
  (makeTranslate vec) `compose` t

{-| Creates a transform that performs a rotation in radians about the origin.
-}
makeRotate : Float -> TransSE2
makeRotate angle =
  transSE2 0 0 angle
-- makeRotate angle = TransSE2 {x=0, y=0, angle=angle}

{-| Creates a transform that performs a translation by the given vector.
-}
makeTranslate : Vec2 -> TransSE2
makeTranslate vec =
  let
    (x, y) = Math.Vector2.toTuple vec
  in
    transSE2 x y 0
    -- TransSE2 {x=x, y=y, angle=0}

lookAt : Vec2 -> Vec2 -> TransSE2
lookAt from to =
  let
    vecHeading = to `Math.Vector2.sub` from
    (x, y) = Math.Vector2.toTuple vecHeading
    angle = Angle.vectorToBearing(vecHeading)
  in
   transSE2 x y angle

-- {-| Returns the 3x3 matrix that represents this transform in homogeneous coordinates.
-- -}
-- toSquareMatrix : TransSE2 -> Mat3
-- toSquareMatrix = Mat3.identity
