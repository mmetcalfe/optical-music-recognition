import Html exposing (div, button, text)
-- import Html.Events exposing (onClick)
-- import StartApp.Simple as StartApp
import StartApp
import Task
import Effects
import Mouse
import Time
import Window

import Random
import Random.Distributions
-- import Plot

-- Graphics
import Color
import Graphics.Collage as GfxC
import Graphics.Element as GfxE
import Transform2D
import Text
import LinearAlgebra.TransformSE2 as TransformSE2 exposing (TransSE2)
import Math.Vector2 as Vector2 exposing (Vec2)
-- import LinearAlgebra.Matrix2 as Matrix2 exposing (Mat2)

import Curves exposing (BezierSpline)

type Action =
  -- WindowDims (Int, Int)
  Tick Float (Int, Int) -- time, windowDims
  | MousePosition (Int, Int)
  | MouseDown Bool

-- { values : List (Float)
type alias Model =
    { points : List Vec2
    , mousePos : Vec2
    , dragging : Bool
    , canvasWidth : Int
    , windowDims : (Int, Int)
    -- Old stuff:
    , values : List Float
    , pairs : List (Float, Float)
    , seed : Random.Seed
    }

floatGen : Random.Generator Float
-- floatGen = (Random.float -1 1)
-- floatGen = Random.Float.unitRange
-- floatGen = (Random.Float.normal -1 1 1)
-- floatGen = standardNormal
floatGen = Random.Distributions.normal
generateValue : Random.Seed -> (Float, Random.Seed)
generateValue seed =
  let (f, s) = Random.generate floatGen seed
  in (f*0.2, s)

-- onWindowDims : Signal Action
-- onWindowDims = Signal.map (\dims -> WindowDims dims) Window.dimensions

onTick : Signal Action
onTick = Signal.map2 (\time dims -> Tick time dims) (Time.fps 1) Window.dimensions
-- onTick = Signal.map (\time -> Tick time) (Time.fps 100)
-- onTick = Signal.map (\time -> Tick time) (Time.every Time.millisecond)

onMouseMove : Signal Action
onMouseMove = Signal.map (\position -> MousePosition position) Mouse.position

onMouseDown : Signal Action
onMouseDown = Signal.map (\isDown -> MouseDown isDown) Mouse.isDown


-- type Action = Increment | Decrement

-- model : number
  -- Model [0, 1, 2]
initialModel : Model
initialModel =
  let
    points = [Vector2.vec2 10 30, Vector2.vec2 50 50, Vector2.vec2 100 100, Vector2.vec2 100 200]
    mousePos = Vector2.vec2 0 0
    dragging = False
    canvasWidth = 600
    windowDims = (500, 500)
    values = (List.map toFloat [])
    pairs = []
    seed = (Random.initialSeed 1)
  in
    Model points mousePos dragging canvasWidth windowDims values pairs seed
  -- Model () (List.map toFloat []) [] (Random.initialSeed 1)
    -- { values = List.map toFloat [0, 1, 2]
    -- , seed = Random.initialSeed 0
    -- }

app : StartApp.App Model
app =
  StartApp.start
    { init = init
    , update = update
    -- , update = ZigguratTest.update
    -- , model = model
    , view = view
    -- , view = ZigguratTest.view
    , inputs = [ onTick, onMouseMove, onMouseDown ]
    -- , inputs = [ onMouseMove, onMouseDown, onWindowDims ]
    }

main : Signal Html.Html
main = app.html

-- Tasks?
port tasks : Signal (Task.Task Effects.Never ())
port tasks =
  app.tasks

-- view address model =
--   div []
--     [ button [ onClick address Decrement ] [ Html.text "-" ]
--     , div [] [ Html.text (toString model) ]
--     , button [ onClick address Increment ] [ Html.text "+" ]
--     ]

init : (Model, Effects.Effects Action)
init = (initialModel, Effects.none)
  -- ( Model topic "assets/waiting.gif"
  -- , getRandomGif topic
  -- )

update : Action -> Model -> (Model, Effects.Effects Action)
update action model =
  let
    model' = case action of
      Tick time dims -> tickUpdate time dims model
      MousePosition pos -> mouseMoveUpdate pos model
      MouseDown isDown -> mouseButtonUpdate isDown model
      -- WindowDims dims -> windowDimsUpdate dims model
  in
    (model', Effects.none)
  --   Decrement -> model - 1

-- windowDimsUpdate : (Int, Int) -> Model -> Model
-- windowDimsUpdate dims model =
--   { model |
--     windowDims = dims
--   }

tickUpdate : Float -> (Int, Int) -> Model -> Model
tickUpdate time dims model =
  let
    (value, seed') = generateValue model.seed
  in { model |
      values = List.take 500 <| value :: model.values,
      seed = seed',
      windowDims = dims
    }

-- moveSpline : Vec2 -> BezierSpline -> BezierSpline
-- moveSpline mousePos
--   let
--     mouseDist pos = Vector2.distance pos mousePos
--     sorted = List.sortBy mouseDist model.points
--     closest = List.head sorted
--     replace closest pt =
--       if closest == pt
--         then mousePos
--         else pt
--     points = case closest of
--       Just pos ->
--         if model.dragging && mouseDist pos < 100
--           then
--             List.map (replace pos) model.points
--           else
--         model.points
--       _ -> model.points
--   in { model |
--       mousePos = mousePos,
--       points = points
--     }

mouseMoveUpdate : (Int, Int) -> Model -> Model
mouseMoveUpdate (ix, iy) model =
  let
    -- hs = 0.5 * toFloat model.canvasWidth
    x = toFloat ix - 0.5 * toFloat (fst model.windowDims)
    y = 0.5 * toFloat (snd model.windowDims) - toFloat iy
    mousePos = Vector2.vec2 x y
    mouseDist pos = Vector2.distance pos mousePos
    -- (close, far) = List.partition (\pos -> mouseDist pos < 5) model.points
    sorted = List.sortBy mouseDist model.points
    closest = List.head sorted
    replace closest pt =
      if closest == pt
        then mousePos
        else pt
    points = case closest of
      Just pos ->
        if model.dragging && mouseDist pos < 100
          then
            List.map (replace pos) model.points
          else
        model.points
      _ -> model.points
  in { model |
      mousePos = mousePos,
      points = points
    }

mouseButtonUpdate : Bool -> Model -> Model
mouseButtonUpdate isDown model =
  let
    mouseDist pos = Vector2.distance pos model.mousePos
    isClose = List.any (\pos -> mouseDist pos < 20) model.points
  in { model |
      dragging = (isDown && isClose)
    }

label : String -> Float -> Float -> GfxC.Form
label str x y =
      let
        text = Text.fromString str
          |> Text.height 0.1
          |> Text.color Color.red
          -- |> Text.color col
      in
        GfxC.text text
          |> GfxC.move (x, y)

colPoint : Color.Color -> Vec2 -> GfxC.Form
colPoint col xy =
      let
        r = 5
      in
        GfxC.circle r
          |> GfxC.filled col
          |> GfxC.move (Vector2.toTuple xy)

view : Signal.Address Action -> Model -> Html.Html
view address model =
  let
    drawPoint n xy =
      let
        (x, y) = Vector2.toTuple xy
        r = 5
        col = Color.hsl (toFloat ((n * 71) % 256)) 1 0.5
      in
        GfxC.circle r
          |> GfxC.filled col
          |> GfxC.move (x, y)

  -- let (rand, seed') =
  --   generateValue model.seed
  -- in
    -- div [] [Html.text <| toString model.values]
  -- Html.fromElement <| Plot.timeSeries Plot.defaultPlot model.values
    points = List.indexedMap drawPoint model.points
    curve = case model.points of
      p1::p2::p3::p4::[] ->
        let
          frameFunc = Curves.cubicBezierFrame (p1, p2, p3, p4)
          (randomOffsets, seed) = Curves.randomOffsets (0, 1) 10 frameFunc 100 model.seed
          randomOffsetPoints = List.map (colPoint (Color.hsl 0 0 0.5)) randomOffsets
        in
          GfxC.group <| [
            Curves.drawCubicBezier (p1, p2, p3, p4),
            Curves.drawCubicBezierFrames (p1, p2, p3, p4)
          ] ++ randomOffsetPoints
      _ -> label "Failed" 0 0
    forms = curve :: points
    (cw, ch) = model.windowDims
    drawing = GfxC.collage cw ch forms
  in
    Html.fromElement <| drawing
