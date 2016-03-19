import Html exposing (div, button, text)
-- import Html.Events exposing (onClick)
-- import StartApp.Simple as StartApp
import StartApp
import Task
import Effects
import Mouse
import Time

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
import LinearAlgebra.Matrix2 as Matrix2 exposing (Mat2)


type Action =
  Tick Float
  | MousePosition (Int, Int)
  | MouseDown Bool

-- { values : List (Float)
type alias Model =
    { points : List Vec2
    , mousePos : Vec2
    , dragging : Bool
    , canvasWidth : Int
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

onTick : Signal Action
-- onTick = Signal.map (\time -> Tick time) (Time.every Time.millisecond)
onTick = Signal.map (\time -> Tick time) (Time.fps 100)

onMouseMove = Signal.map (\position -> MousePosition position) Mouse.position
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
    values = (List.map toFloat [])
    pairs = []
    seed = (Random.initialSeed 1)
    canvasWidth = 600
  in
    Model points mousePos dragging canvasWidth values pairs seed
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
    -- , inputs = [ onTick, onMouseMove, onMouseDown ]
    , inputs = [ onMouseMove, onMouseDown ]
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
      Tick time -> tickUpdate time model
      MousePosition pos -> mouseMoveUpdate pos model
      MouseDown isDown -> mouseButtonUpdate isDown model
  in
    (model', Effects.none)
  --   Decrement -> model - 1

tickUpdate : Float -> Model -> Model
tickUpdate time model =
  let
    (value, seed') = generateValue model.seed
  in { model |
      values = List.take 500 <| value :: model.values,
      seed = seed'
    }

mouseMoveUpdate : (Int, Int) -> Model -> Model
mouseMoveUpdate (ix, iy) model =
  let
    hs = 0.5 * toFloat model.canvasWidth
    x = toFloat ix - hs
    y = hs - toFloat iy
    mousePos = Vector2.vec2 x y
    mouseDist pos = Vector2.distance pos mousePos
    -- (close, far) = List.partition (\pos -> mouseDist pos < 5) model.points
    sorted = List.sortBy mouseDist model.points
    closest = List.head sorted
    points = case closest of
      Just pos ->
        if model.dragging && mouseDist pos < 50
          then
            mousePos :: (List.drop 1 sorted)
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

view : Signal.Address Action -> Model -> Html.Html
view address model =
  let
    drawPoint xy =
      let
        (x, y) = Vector2.toTuple xy
        r = 5
        col = Color.hsl 0 0 0
      in
        GfxC.circle r
          |> GfxC.filled col
          |> GfxC.move (x, y)

  -- let (rand, seed') =
  --   generateValue model.seed
  -- in
    -- div [] [Html.text <| toString model.values]
  -- Html.fromElement <| Plot.timeSeries Plot.defaultPlot model.values
    points = List.map drawPoint model.points
    drawing = GfxC.collage model.canvasWidth model.canvasWidth points
  in
    Html.fromElement <| drawing
