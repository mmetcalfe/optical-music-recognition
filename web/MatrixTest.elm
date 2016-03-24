import Html exposing (div, button, text)
import Graphics.Element as GfxE


import LinearAlgebra.Matrix as Matrix

-- inv, qr, gaussJordan
-- TODO: Test non-square matrices.

unsafe : String -> Maybe a -> a
unsafe msg m =
    case m of
        Just v ->
            v
        _ ->
            Debug.crash msg

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
    mat44 = unsafe "mat44" <| Matrix.fromLists [[3, 2, 0, 4], [8, 7, 5, 1], [2, 4, 9, 6], [6, 10, 12, 16]]
    mat35 = unsafe "mat35" <| Matrix.fromLists [[3, 2, 0, 4, 4], [8, 7, 5, 1, 1], [2, 4, 9, 6, 6]]
    mat53 = unsafe "mat53" <| Matrix.fromLists [[3, 2, 0], [8, 7, 5], [2, 4, 9], [6, 10, 12], [3, 2, 1]]
    qr53 = Matrix.qr mat53
    (q53, r53) = Maybe.withDefault (i,i) qr53
    qr35 = Matrix.qr mat35
    (q35, r35) = Maybe.withDefault (i,i) qr35

    -- solve:
    vec4 = unsafe "vec4" <| Matrix.fromLists [[0.1],[0.2],[0.3],[0.4]]
    soln4 = unsafe "soln4" <| Matrix.solve mat44 vec4
    soln4Test = unsafe "soln4Test" <| Matrix.mul mat44 soln4

    vec3 = unsafe "vec3" <| Matrix.fromLists [[0.1],[0.2],[0.3]]
    soln5 = unsafe "soln5" <| Matrix.solve mat35 vec3
    soln5Test = unsafe "soln5Test" <| Matrix.mul mat35 soln5

    vec5 = unsafe "vec5" <| Matrix.fromLists [[0.1],[0.2],[0.3],[0.4],[0.5]]
    soln3 = unsafe "soln3" <| Matrix.solve mat53 vec5
    soln3Test = unsafe "soln3Test" <| Matrix.mul mat53 soln3
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
      , GfxE.show <| Matrix.toLists mat53
      , GfxE.show <| Matrix.toLists q53
      , GfxE.show <| Matrix.toLists r53
      , GfxE.show <| Maybe.map Matrix.toLists (Matrix.mul q53 r53)
      , GfxE.show "  3x5:"
      , GfxE.show <| Matrix.toLists mat35
      , GfxE.show <| Matrix.toLists q35
      , GfxE.show <| Matrix.toLists r35
      , GfxE.show <| Maybe.map Matrix.toLists (Matrix.mul q35 r35)
      , GfxE.show "General matrix and inverse:"
      , GfxE.show <| Maybe.map Matrix.toLists d
      , GfxE.show <| Maybe.map Matrix.toLists dinv
      , GfxE.show <| Maybe.map Matrix.toLists dinvtest
      , GfxE.show "Solve system:"
      , GfxE.show "  4x4:"
      , GfxE.show <| Matrix.toLists mat44
      , GfxE.show <| Matrix.toLists vec4
      , GfxE.show <| Matrix.toLists soln4
      , GfxE.show <| Matrix.toLists soln4Test
      , GfxE.show "3x5 (underdetermined):"
      , GfxE.show <| Matrix.toLists vec3
      , GfxE.show <| Matrix.toLists soln5
      , GfxE.show <| Matrix.toLists soln5Test
      , GfxE.show "5x3 (overdetermined):"
      , GfxE.show <| Matrix.toLists vec5
      , GfxE.show <| Matrix.toLists soln3
      , GfxE.show <| Matrix.toLists soln3Test
      ]
