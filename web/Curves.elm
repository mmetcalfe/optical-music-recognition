module Curves where

{-| Bezier curve functions based on http://pomax.github.io/bezierinfo/
-}

-- Graphics
import Color
import Graphics.Collage as GfxC
-- import Graphics.Element as GfxE
-- import Transform2D
import LinearAlgebra.TransformSE2 as TransformSE2 exposing (TransSE2)
import Math.Vector2 as Vector2 exposing (Vec2)
-- import LinearAlgebra.Matrix2 as Matrix2 exposing (Mat2)
import LinearAlgebra.Angle as Angle
import LinearAlgebra.Matrix as Matrix exposing (Mat)
import LinearAlgebra.Matrix.Unsafe as UnsafeMatrix

import Random
import Random.Distributions

import Array
import Ransac

type alias CubicBezierSpline = (Vec2, Vec2, Vec2, Vec2)

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

cubicBezier : CubicBezierSpline -> Float -> Vec2
cubicBezier spline t =
  let
    (a, b, c, d) = spline
    (fnX, fnY) = cubicBases bezier3 a b c d
  in
    Vector2.vec2 (fnX t) (fnY t)

cubicBezierTangent : CubicBezierSpline -> Float -> Vec2
cubicBezierTangent spline t =
  let
    (a, b, c, d) = spline
    (fnX, fnY) = cubicBases bezier3' a b c d
  in
    Vector2.normalize <| Vector2.vec2 (fnX t) (fnY t)

cubicBezierFrame : CubicBezierSpline -> Float -> TransSE2
cubicBezierFrame spline t =
  let
    point = cubicBezier spline t
    tangent = cubicBezierTangent spline t
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

-- drawPath : List Vec2 -> GfxC.Form
drawPath : List Vec2 -> GfxC.Path
drawPath pts =
  -- let
    -- ls = GfxC.defaultLine
  -- in
    GfxC.path (List.map Vector2.toTuple pts)
      -- |> GfxC.traced ls

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

-- drawParametric : (Float -> Vec2) -> Int -> (Float, Float) -> GfxC.Form
drawParametric : (Float -> Vec2) -> Int -> (Float, Float) -> GfxC.Path
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

-- drawCubicBezier : CubicBezierSpline -> GfxC.Form
drawCubicBezier : CubicBezierSpline -> GfxC.Path
drawCubicBezier spline =
  let
    func = cubicBezier spline
  in
    drawParametric func 100 (0, 1)

drawCubicBezierFrames : CubicBezierSpline -> GfxC.Form
drawCubicBezierFrames spline =
  let
    func = cubicBezierFrame spline
  in
    drawParametricFrames func 10 (0, 1)

debugDrawCubicBezier : Color.Color -> CubicBezierSpline -> GfxC.Form
debugDrawCubicBezier col spline =
  let
    hsl = Color.toHsl col
    hullCol = Color.hsl hsl.hue (hsl.saturation*0.5) (0.66 + hsl.lightness*0.33)
    lw = 3
    circleLineStyle =
      let ls = GfxC.defaultLine
      in { ls | width = lw, color = col }
    hullLineStyle =
      let ls = GfxC.defaultLine
      in { ls | width = lw, color = hullCol } -- Color.complement col }
    curveLineStyle =
      let ls = GfxC.defaultLine
      in { ls | width = 2*lw, color = col }
    drawCircle xy =
      GfxC.circle (1.5*lw)
        |> GfxC.outlined circleLineStyle
        |> GfxC.move (Vector2.toTuple xy)
    (a, b, c, d) = spline
    points = [a, b, c, d]
    lines = drawPath [ a, b, c, d ]
    circles = List.map drawCircle points
  in
    GfxC.group <| [
      lines |> GfxC.traced hullLineStyle,
      (drawCubicBezier spline) |> GfxC.traced curveLineStyle,
      drawCubicBezierFrames spline
    ] ++ circles

cubicBezierMatrix : Mat
cubicBezierMatrix =
  UnsafeMatrix.fromLists
    [ [-1,  3, -3,  1]
    , [ 3, -6,  3,  0]
    , [-3,  3,  0,  0]
    , [ 1,  0,  0,  0]
    ]

estimateTimes : List Vec2 -> List Float
-- TODO: Use a time estimation method similar to:
-- http://www.vision.caltech.edu/malaa/publications/aly08realtime.pdf
estimateTimes points =
  let
    len = List.length points
  in
    List.take len <| linspace len (0, 1)

vec2FromList : List Float -> Vec2
vec2FromList values =
  case values of
    a::b::[] ->
      Vector2.vec2 a b
    _ ->
      Debug.crash "vec2FromList: incorrect number of values."

cubicBezierFromLists : List (List Float) -> CubicBezierSpline
cubicBezierFromLists points =
  let
    vectors = List.map vec2FromList points
  in
    case vectors of
      v1::v2::v3::v4::[] ->
        (v1, v2, v3, v4)
      _ ->
        Debug.crash "cubicBezierFromLists: incorrect number of points."

cubicBezierFromPoints : List Vec2 -> CubicBezierSpline
cubicBezierFromPoints vectors =
    case vectors of
      v1::v2::v3::v4::[] ->
        (v1, v2, v3, v4)
      _ ->
        Debug.crash "cubicBezierFromLists: incorrect number of points."

fitCubicBezierToPoints : List Vec2 -> CubicBezierSpline
fitCubicBezierToPoints points =
  let
    sortedPoints = List.sortBy Vector2.getX points
    times = estimateTimes sortedPoints
    tLists = List.map (\t -> [t^3, t^2, t, 1]) times
    qLists = List.map ((\(a, b) -> [a, b]) << Vector2.toTuple) sortedPoints
    q = UnsafeMatrix.fromLists qLists
    t = UnsafeMatrix.fromLists tLists
    m = cubicBezierMatrix
    tm = UnsafeMatrix.mul t m
    p = UnsafeMatrix.solve tm q
    pLists = UnsafeMatrix.toLists p
  in
    cubicBezierFromLists pLists

pointsNearCubicBezier : Float -> Array.Array Vec2 -> CubicBezierSpline -> List Vec2
-- TODO: Use a faster method for production code.
-- e.g. http://www.vision.caltech.edu/malaa/publications/aly08realtime.pdf
pointsNearCubicBezier maxDist data spline =
  let
    times = linspace 50 (0, 1)
    samples = List.map (cubicBezier spline) times
    sampleDist pt =
      let
        dists = List.map (Vector2.distance pt) samples
      in
        List.minimum dists
    isNearSpline pt =
      case sampleDist pt of
        Just dist ->
          dist < maxDist
        Nothing ->
          False
  in
    Array.toList <| Array.filter isNearSpline data

fitCubicBezierRansac : Random.Seed -> List Vec2 -> (Maybe CubicBezierSpline, Random.Seed)
fitCubicBezierRansac seed points =
  let
    numPoints = List.length points
    inlierRatio = 0.2
    model =
      { fitInliers = fitCubicBezierToPoints
      , numRequired = 5
      , findInliers = pointsNearCubicBezier
      , fitModel = Just << fitCubicBezierToPoints
      }
    params =
      { numIterations = 100
      , maxDist = 7
      , minInliers = round <| inlierRatio * toFloat numPoints
      }
    data = Array.fromList points
    splineGen = Ransac.ransac model params data
  in
    Random.generate splineGen seed
