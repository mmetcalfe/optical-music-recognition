module Curves where

-- Graphics
import Color
import Graphics.Collage as GfxC
import Graphics.Element as GfxE
import Transform2D
import Text
import LinearAlgebra.TransformSE2 as TransformSE2 exposing (TransSE2)
import Math.Vector2 as Vector2 exposing (Vec2)
import LinearAlgebra.Matrix2 as Matrix2 exposing (Mat2)
import LinearAlgebra.Angle as Angle

import Random
import Random.Distributions

bezier2 : (Float, Float, Float) -> Float -> Float
bezier2 (w0, w1, w2) t =
  let
    t2 = t*t
    mt = 1 - t
    mt2 = mt*mt
  in
    w0*mt2 + w1*2*mt*t + w2*t2

bezier3 : (Float, Float, Float, Float) -> Float -> Float
bezier3 (w0, w1, w2, w3) t =
  let
    t2 = t*t
    t3 = t*t2
    mt = 1 - t
    mt2 = mt*mt
    mt3 = mt*mt2
  in
    w0*mt3 + w1*3*mt2*t + w2*3*mt*t2 + w3*t3

bezier3' : (Float, Float, Float, Float) -> Float -> Float
bezier3' (w0, w1, w2, w3) t =
  let
    dw0 = 3*(w1-w0)
    dw1 = 3*(w2-w1)
    dw2 = 3*(w3-w2)
  in
    bezier2 (dw0, dw1, dw2) t

cubicBases
  : ((Float, Float, Float, Float) -> Float -> Float)
  -> Vec2 -> Vec2 -> Vec2 -> Vec2
  -> ((Float -> Float), (Float -> Float))
cubicBases func a b c d =
  let
    (wx0, wy0) = Vector2.toTuple a
    (wx1, wy1) = Vector2.toTuple b
    (wx2, wy2) = Vector2.toTuple c
    (wx3, wy3) = Vector2.toTuple d
    fnX = func (wx0, wx1, wx2, wx3)
    fnY = func (wy0, wy1, wy2, wy3)
  in
    (fnX, fnY)

cubicBezier : Vec2 -> Vec2 -> Vec2 -> Vec2 -> Float -> Vec2
cubicBezier a b c d t =
  let
    (fnX, fnY) = cubicBases bezier3 a b c d
  in
    Vector2.vec2 (fnX t) (fnY t)

cubicBezierTangent : Vec2 -> Vec2 -> Vec2 -> Vec2 -> Float -> Vec2
cubicBezierTangent a b c d t =
  let
    (fnX, fnY) = cubicBases bezier3' a b c d
  in
    Vector2.normalize <| Vector2.vec2 (fnX t) (fnY t)

cubicBezierFrame : Vec2 -> Vec2 -> Vec2 -> Vec2 -> Float -> TransSE2
cubicBezierFrame a b c d t =
  let
    point = cubicBezier a b c d t
    tangent = cubicBezierTangent a b c d t
    (tx, ty) = Vector2.toTuple tangent
    -- normal = Vector2.vec2 ty -tx
    angle = Angle.vectorToBearing tangent
  in
    TransformSE2.fromComponents point angle

randomOffsetGen : (Float, Float) -> Float -> (Float -> TransSE2) -> Random.Generator Vec2
randomOffsetGen range stdDev func =
  let
    t = Random.float 0 1
    frame = Random.map func t
    offset = Random.map (\v -> v * stdDev) Random.Distributions.normal
    point f o = TransformSE2.apply f (Vector2.vec2 0 o)
  in
    Random.map2 point frame offset

randomOffsets : (Float, Float) -> Float -> (Float -> TransSE2) -> Int -> Random.Seed -> (List Vec2, Random.Seed)
randomOffsets range stdDev func num seed =
  let
    gen = randomOffsetGen range stdDev func
    list = Random.list num gen
  in
    Random.generate list seed

drawLine : Vec2 -> Vec2 -> GfxC.Form
drawLine a b =
  let
    ls = GfxC.defaultLine
  in
    GfxC.segment (Vector2.toTuple a) (Vector2.toTuple b)
      |> GfxC.traced ls

drawPath : List Vec2 -> GfxC.Form
drawPath pts =
  let
    ls = GfxC.defaultLine
  in
    GfxC.path (List.map Vector2.toTuple pts)
      |> GfxC.traced ls

linspace : Int -> (Float, Float) -> List Float
linspace n (minv, maxv) =
  let
    intoRange v = minv + (maxv - minv) * (toFloat v / toFloat n)
  in
    List.map intoRange [0..n]

drawFrame : Float -> TransSE2 -> GfxC.Form
drawFrame r frame =
  let
    pt0 = TransformSE2.apply frame (Vector2.vec2 0 0)
    ptx = TransformSE2.apply frame (Vector2.vec2 r 0)
    pty = TransformSE2.apply frame (Vector2.vec2 0 r)
  in
    GfxC.group [drawLine pt0 ptx, drawLine pt0 pty]

drawParametric : (Float -> Vec2) -> Int -> (Float, Float) -> GfxC.Form
drawParametric func samples range =
  let
    times = linspace samples range
  in
    drawPath (List.map func times)

drawParametricFrames : (Float -> TransSE2) -> Int -> (Float, Float) -> GfxC.Form
drawParametricFrames func samples range =
  let
    times = linspace samples range
  in
    GfxC.group (List.map (drawFrame 10 << func) times)

drawCubicBezier : Vec2 -> Vec2 -> Vec2 -> Vec2 -> GfxC.Form
drawCubicBezier a b c d =
  let
    lines = drawPath [ a, b, c, d ]
    func = cubicBezier a b c d
    curve = drawParametric func 100 (0, 1)
  in
    GfxC.group [ lines, curve ]

drawCubicBezierFrames : Vec2 -> Vec2 -> Vec2 -> Vec2 -> GfxC.Form
drawCubicBezierFrames a b c d =
  let
    lines = drawPath [ a, b, c, d ]
    func = cubicBezierFrame a b c d
    curve = drawParametricFrames func 10 (0, 1)
  in
    GfxC.group [ lines, curve ]
