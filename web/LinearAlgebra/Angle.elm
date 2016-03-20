module LinearAlgebra.Angle
  ( normaliseAngle
  , vectorToBearing
  ) where

import Math.Vector2 exposing (Vec2)

fmod : Float -> Float -> Float
fmod value denom =
    value - (toFloat <| truncate (value / denom)) * denom


normaliseAngle : Float -> Float
normaliseAngle value =
  let
    angle = value `fmod` (2*pi)

    angle2 = if angle <= -pi then angle + 2*pi else angle

    angle3 = if angle2 > pi then angle2 - 2*pi else angle2
  in
    angle3

vectorToBearing : Vec2 -> Float
vectorToBearing vec =
  let
    (x, y) = Math.Vector2.toTuple vec
  in
    atan2 y x
