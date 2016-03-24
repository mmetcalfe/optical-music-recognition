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

    leftm = Maybe.withDefault i <| Matrix.fromLists [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    rightm = Maybe.withDefault i <| Matrix.fromLists [[-1, -2], [-3, -4], [-5, -6]]
    joinedm = Matrix.joinRows leftm rightm

    -- d = Matrix.fromLists [[3, 2], [7, 5]]
    d = Matrix.fromLists [[3, 2, 6], [7, 5, 1], [4, 9, 4]]
    ah = d `Maybe.andThen` Matrix.col 0 `Maybe.andThen` Matrix.householder
    aha = Maybe.withDefault Nothing <| Maybe.map2 Matrix.mul ah d
    uqr = d `Maybe.andThen` Matrix.qr
    (uq, ur) = Maybe.withDefault (i,i) uqr

    -- inverse
    dinv = d `Maybe.andThen` Matrix.inv
    dinvtest = Maybe.withDefault Nothing <| Maybe.map2 Matrix.mul d dinv

    -- non-square:
    mat53 = Matrix.fromLists [[3, 2, 6], [7, 5, 1], [4, 9, 0], [6, 1, 8], [4, 2, 3]]
    qr53 = mat53 `Maybe.andThen` Matrix.qr
    (q53, r53) = Maybe.withDefault (i,i) qr53
    mat35 = Matrix.fromLists [[3, 2, 0, 4, 1], [8, 7, 5, 1, 6], [2, 4, 9, 6, 8]]
    qr35 = mat35 `Maybe.andThen` Matrix.qr
    (q35, r35) = Maybe.withDefault (i,i) qr35
  in
    GfxE.flow GfxE.down
      [ GfxE.show <| i
      , GfxE.show "Matrix:"
      , GfxE.show <| a
      , GfxE.show "Negate:"
      , GfxE.show <| ma
      , GfxE.show "Scale:"
      , GfxE.show <| sa
      , GfxE.show "Transpose:"
      , GfxE.show <| sat
      , GfxE.show "To Lists:"
      , GfxE.show <| Matrix.toLists i
      , GfxE.show <| Maybe.map Matrix.toLists b
      , GfxE.show <| Maybe.map Matrix.toLists c
      , GfxE.show "Add:"
      , GfxE.show <| Maybe.map Matrix.toLists bpc
      , GfxE.show "Subtract:"
      , GfxE.show <| Maybe.map Matrix.toLists bmc
      , GfxE.show "Multiply:"
      , GfxE.show <| Maybe.map Matrix.toLists bc
      , GfxE.show "Upper triangular matrix and inverse:"
      , GfxE.show <| Maybe.map Matrix.toLists u
      , GfxE.show <| Maybe.map Matrix.toLists ui
      , GfxE.show "Identities:"
      , GfxE.show <| Maybe.map Matrix.toLists uui
      , GfxE.show <| Maybe.map Matrix.toLists uiu
      , GfxE.show "Newton approx:"
      , GfxE.show <| Maybe.map Matrix.toLists nui
      , GfxE.show <| Maybe.map Matrix.toLists unui
      , GfxE.show "Householder:"
      , GfxE.show <| Maybe.map Matrix.toLists a
      , GfxE.show <| Maybe.map Matrix.toLists ah
      , GfxE.show <| Maybe.map Matrix.toLists aha
      , GfxE.show "Joins:"
      , GfxE.show "  components:"
      , GfxE.show <| Matrix.toLists leftm
      , GfxE.show <| Matrix.toLists rightm
      , GfxE.show "  transposes:"
      , GfxE.show <| Matrix.toLists <| Matrix.transpose leftm
      , GfxE.show <| Matrix.toLists <| Matrix.transpose rightm
      , GfxE.show "  joined:"
      , GfxE.show <| Maybe.map Matrix.toLists joinedm
      , GfxE.show "QR decomposition:"
      , GfxE.show <| Maybe.map Matrix.toLists d
      , GfxE.show <| Matrix.toLists uq
      , GfxE.show <| Matrix.toLists ur
      , GfxE.show <| Maybe.map Matrix.toLists (Matrix.mul uq ur)
      , GfxE.show "  non-square:"
      , GfxE.show "  5x3:"
      , GfxE.show <| Maybe.map Matrix.toLists mat53
      , GfxE.show <| Matrix.toLists q53
      , GfxE.show <| Matrix.toLists r53
      , GfxE.show <| Maybe.map Matrix.toLists (Matrix.mul q53 r53)
      , GfxE.show "  3x5:"
      , GfxE.show <| Maybe.map Matrix.toLists mat35
      , GfxE.show <| Matrix.toLists q35
      , GfxE.show <| Matrix.toLists r35
      , GfxE.show <| Maybe.map Matrix.toLists (Matrix.mul q35 r35)
      , GfxE.show "General matrix and inverse:"
      , GfxE.show <| Maybe.map Matrix.toLists d
      , GfxE.show <| Maybe.map Matrix.toLists dinv
      , GfxE.show <| Maybe.map Matrix.toLists dinvtest
      ]
