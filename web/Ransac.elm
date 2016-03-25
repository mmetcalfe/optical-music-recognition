module Ransac where

import Array
import Random exposing (Generator)
import Random.Array

type alias RansacModel pt model =
  -- Find the best model that best fits a minimal set of points
  { fitInliers : List pt -> model

  -- Number of points required for fitInliers
  , numRequired : Int

  -- Find the number of points within a given threshold of the model
  , findInliers : Float -> Array.Array pt -> model -> List pt

  -- Find the model that best fits a large set of points
  , fitModel : List pt -> Maybe model
  }

type alias RansacParams =
  -- Number of attempted model fits
  { numIterations : Int

  -- Minimum distance to model to count as an inlier
  , minDist : Float

  -- Minimum number of inliers required for a model to be accepted
  , minInliers : Int
  }

type alias RansacState pt model =
  { samples : List pt
  , model : Maybe model
  , inliers : List pt
  }

chooseBestState : RansacState p m -> RansacState p m -> RansacState p m
chooseBestState stateA stateB =
  if List.length stateA.inliers > List.length stateB.inliers
    then
      stateA
    else
      stateB

chooseStep : (List a, Array.Array a) -> Generator (List a, Array.Array a)
chooseStep (lst, arr) =
  let
    append (maybeVal, subArr) =
      case maybeVal of
        Just v ->
          ((v :: lst), subArr)
        Nothing ->
          (lst, subArr)
  in
    Random.map append (Random.Array.choose arr)

chooseK : Int -> Array.Array a -> Generator (List a)
chooseK k array =
  let
    foldFunc i g = g `Random.andThen` chooseStep
    initGen = chooseStep ([], array)
    choiceAndOthers = List.foldl foldFunc initGen [2..k]
  in
    Random.map fst choiceAndOthers

ransac
  : Random.Seed
  -> RansacModel pt model
  -> RansacParams
  -> Array.Array pt
  -> Generator (Maybe model)
ransac seed model params data =
  let
    ransacIteration state =
      let
        -- Randomly select points:
        sampleGen = chooseK model.numRequired data

        -- Generate a RansacState from a set of samples:
        processSamples samples =
          let
            -- Fit the model:
            currentFit = model.fitInliers samples

            -- Find the set of inliers:
            currentInliers = model.findInliers params.minDist data currentFit
          in
            { samples = samples
            , model = Just currentFit
            , inliers = currentInliers
            }

        -- Generate candidate models:
        stateGen = Random.map processSamples sampleGen
      in
        Random.map (chooseBestState state) stateGen

    initialGen = ransacIteration (RansacState [] Nothing [])
    foldFunc i g = g `Random.andThen` ransacIteration
    stateGen = List.foldl foldFunc initialGen [1..params.numIterations]
    closeFit = Random.map (\st -> model.fitModel st.inliers) stateGen
  in
    closeFit
