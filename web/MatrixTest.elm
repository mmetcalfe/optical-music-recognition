import Html exposing (div, button, text)
import Graphics.Element as GfxE


import LinearAlgebra.Matrix as Matrix

-- inv, qr, gaussJordan
-- TODO: Test non-square matrices.

main : GfxE.Element
main =
  let
    i = Matrix.identity (3, 3)
    a = Matrix.fromLists [[1, 2, 3], [4, -5, 6], [-7, 8, 9]]
    ma = Maybe.map Matrix.negate a
    sa = Maybe.map (Matrix.scale 2) a
    sat = Maybe.map Matrix.transpose sa
    b = Matrix.fromLists [[0, 2, 0], [4, 0, 6], [0, 8, 0]]
    c = Matrix.fromLists [[1, 0, 3], [0, 5, 0], [7, 0, 9]]
    bpc = Maybe.withDefault Nothing <| Maybe.map2 Matrix.add b c
    bmc = Maybe.withDefault Nothing <| Maybe.map2 Matrix.sub b c
    bc = Maybe.withDefault Nothing <| Maybe.map2 Matrix.mul b c
    u = Matrix.fromLists [[1, 2, 3], [0, 5, 6], [0, 0, 9]]
    -- u = Matrix.fromLists [[3, 2], [0, 5]]
    ui = Maybe.withDefault Nothing <| Maybe.map Matrix.invUpperTri u
    uui = Maybe.withDefault Nothing <| Maybe.map2 Matrix.mul u ui
    uiu = Maybe.withDefault Nothing <| Maybe.map2 Matrix.mul ui u
    nui = Maybe.map2 (\m x -> Matrix.newtonInverse 5 m x) u ui
    unui = Maybe.withDefault Nothing <| Maybe.map2 Matrix.mul u nui
  in
    GfxE.flow GfxE.down
      [ GfxE.show <| i
      -- , GfxE.show <| a
      -- , GfxE.show <| ma
      -- , GfxE.show <| sa
      -- , GfxE.show <| sat
      -- , GfxE.show <| Matrix.toLists i
      -- , GfxE.show <| Maybe.map Matrix.toLists b
      -- , GfxE.show <| Maybe.map Matrix.toLists c
      -- , GfxE.show <| Maybe.map Matrix.toLists bpc
      -- , GfxE.show <| Maybe.map Matrix.toLists bmc
      -- , GfxE.show <| Maybe.map Matrix.toLists bc
      , GfxE.show "Matrix and inverse:"
      , GfxE.show <| Maybe.map Matrix.toLists u
      , GfxE.show <| Maybe.map Matrix.toLists ui
      , GfxE.show "Identities:"
      , GfxE.show <| Maybe.map Matrix.toLists uui
      , GfxE.show <| Maybe.map Matrix.toLists uiu
      , GfxE.show "Newton approx:"
      , GfxE.show <| Maybe.map Matrix.toLists nui
      , GfxE.show <| Maybe.map Matrix.toLists unui
      ]
