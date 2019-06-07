module Editor exposing (..)

import Browser
import Browser.Navigation as Navigation
import Http
import Html exposing (Html)
import Url.Builder
import Json.Decode as Json exposing (Decoder)
import Json.Encode as Encode exposing (Value)
import Element exposing (Element)
import Element.Attributes as Attr
import Element.Events as Events
import Style
import Style.Border as Border
import Style.Color as Color
import Style.Font as Font

type Cell
  = Wall
  | Empty
  | Ghost
  | Berry

type Berry = Taken | NotTaken

type State = A | B | C | D | E | F | G | H

type Move = Up | Down | Left | Right | Wait

type alias Rule =
  { up : Maybe Cell
  , down : Maybe Cell
  , left : Maybe Cell
  , right : Maybe Cell
  , berry : Maybe Berry
  , state : Maybe State
  , nextMove : Move
  , nextState : State
  }

type SubmitResponse = Success | RateLimitExceeded | LevelClosed | Unauthorised | Fail String

type SubmitStatus = Pending | NotStarted | Finished SubmitResponse

type alias Model =
  { rules : List Rule
  , submit : SubmitStatus
  }

type Msg
  = Noop
  | ChangeRule { index : Int, rule : Rule }
  | RemoveRule Int
  | MoveUp Int
  | MoveDown Int
  | AddRule
  | Submit
  | Submitted SubmitResponse

main : Program () Model Msg
main = Browser.document
  { init = always (init, Cmd.none)
  , view = \model -> { title = "Pacman", body = [view model] }
  , update = update
  , subscriptions = always Sub.none
  }

init : Model
init = { rules = [], submit = NotStarted }

responseDecoder : Decoder SubmitResponse
responseDecoder = 
  Json.string
  |> Json.andThen (\str ->
    case str of
      "ok" -> Json.succeed Success
      "rateLimitExceeded" -> Json.succeed RateLimitExceeded
      "levelClosed" -> Json.succeed LevelClosed
      "unauthorized" -> Json.succeed Unauthorised
      _ -> Json.fail "bad response message")

showHttpError : Http.Error -> String
showHttpError err =
  case err of
    Http.BadUrl url ->
      "bad request url: " ++ url
    Http.Timeout ->
      "request timeout"
    Http.NetworkError ->
      "network error"
    Http.BadStatus status ->
      "response status: " ++ String.fromInt status
    Http.BadBody body ->
      "bad response body: " ++ body

encodeCell : Cell -> Value
encodeCell cell =
  case cell of
    Wall -> Encode.string "wall"
    Empty -> Encode.string "empty"
    Ghost -> Encode.string "ghost"
    Berry -> Encode.string "berry"

encodeState : State -> Value
encodeState state =
  case state of
    A -> Encode.string "a"
    B -> Encode.string "b"
    C -> Encode.string "c"
    D -> Encode.string "d"
    E -> Encode.string "e"
    F -> Encode.string "f"
    G -> Encode.string "g"
    H -> Encode.string "h"

encodeMaybe : (a -> Value) -> Maybe a -> Value
encodeMaybe f val = case val of
  Just x -> f x
  Nothing -> Encode.null

encodeMove : Move -> Value
encodeMove move =
  case move of
    Up -> Encode.string "up"
    Down -> Encode.string "down"
    Left -> Encode.string "left"
    Right -> Encode.string "right"
    Wait -> Encode.string "wait"

encodeBerry : Berry -> Value
encodeBerry berry =
  case berry of
    Taken -> Encode.string "taken"
    NotTaken -> Encode.string "notTaken"

encodeRule : Rule -> Value
encodeRule rule =
  Encode.object
    [ ("up", encodeMaybe encodeCell rule.up)
    , ("down", encodeMaybe encodeCell rule.down)
    , ("left", encodeMaybe encodeCell rule.left)
    , ("right", encodeMaybe encodeCell rule.right)
    , ("berry", encodeMaybe encodeBerry rule.berry)
    , ("state", encodeMaybe encodeState rule.state)
    , ("nextMove", encodeMove rule.nextMove)
    , ("nextState", encodeState rule.nextState)
    ]

encodeProgram : List Rule -> Value
encodeProgram rules =
  Encode.object
    [ ("rules", Encode.list encodeRule rules)
    ]

encodeRequest : List Rule -> Value
encodeRequest rules =
  Encode.object
    [ ("program", encodeProgram rules)
    ]

submitProgram : List Rule -> Cmd Msg
submitProgram rules =
  let
    handleResult result =
      case result of
        Ok status -> Submitted status
        Err err -> Submitted (Fail (showHttpError err))
  in
    Http.post
      { url = Url.Builder.absolute [ "api", "submit"] []
      , body = Http.jsonBody (encodeRequest rules)
      , expect = Http.expectJson handleResult responseDecoder
      }

setElement : Int -> Maybe a -> List a -> List a
setElement index newElem list =
  case (index, newElem, list) of
    (0, Just elem, x :: xs) -> elem :: xs
    (0, Just elem, []) -> [elem]
    (0, Nothing, x :: xs) -> xs
    (0, Nothing, []) -> []
    (i, _, []) -> []
    (i, _, x :: xs) -> x :: setElement (i - 1) newElem xs

swapPair : Int -> List a -> List a
swapPair first list =
  if first < 0 then
    list
  else
    case (first, list) of
      (0, x :: y :: xs) -> y :: x :: xs
      (i, []) -> []
      (i, x :: xs) -> x :: swapPair (i - 1) xs

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case (msg, model) of
    (Noop, _) ->
      (model, Cmd.none)
    (ChangeRule { index, rule }, { rules, submit }) ->
      ( { rules = setElement index (Just rule) rules
        , submit = submit
        }
      , Cmd.none
      )
    (RemoveRule index, { rules, submit }) ->
      ( { rules = setElement index Nothing rules
        , submit = submit
        }
      , Cmd.none
      )
    (MoveUp index, { rules, submit }) ->
      ( { rules = swapPair (index - 1) rules
        , submit = submit
        }
      , Cmd.none
      )
    (MoveDown index, { rules, submit }) ->
      ( { rules = swapPair index rules
        , submit = submit
        }
      , Cmd.none
      )
    (AddRule, { rules, submit }) ->
      ( { rules = rules ++ [newRule], submit = submit }
      , Cmd.none
      )
    (Submit, { rules, submit }) ->
      ( { rules = rules, submit = Pending }
      , submitProgram rules
      )
    (Submitted Unauthorised, _) ->
      ( model
      , Navigation.load "/index.html"
      )
    (Submitted result, { rules, submit }) ->
      ( { rules = rules, submit = Finished result }
      , Cmd.none
      )

newRule : Rule
newRule =
  { up = Nothing
  , down = Nothing
  , left = Nothing
  , right = Nothing
  , state = Nothing
  , berry = Nothing
  , nextMove = Wait
  , nextState = A
  }

type Styles
  = NoStyle
  | RuleStyle
  | CellStyle
  | ButtonStyle
  | TextStyle

styleSheet =
  Style.styleSheet
    [ Style.style NoStyle []
    , Style.style RuleStyle
      [ Border.solid
      , Border.all 1
      , Color.border (Style.rgb 0 0 0)
      , Color.background (Style.rgb 0.9 0.9 0.9)
      ]
    , Style.style CellStyle
      [ Border.solid
      , Border.all 1
      , Color.border (Style.rgb 0 0 0)
      ]
    , Style.style ButtonStyle
      [ Border.solid
      , Border.all 1
      , Color.border (Style.rgb 0 0 0)
      , Color.background (Style.rgb 1 0.8 0.05)
      ]
    , Style.style TextStyle
      [ Font.size 30
      , Font.typeface [Font.sansSerif]
      ]
    ]

cellSize : Float
cellSize = 65

viewRawCell : String -> msg -> Element Styles v msg
viewRawCell image msg =
  Element.image CellStyle
    [ Attr.width (Attr.px cellSize)
    , Attr.height (Attr.px cellSize)
    , Events.onClick msg
    ]
    { caption = ""
    , src = "/images/" ++ image ++ ".png"
    }

viewCell : Maybe Cell -> msg -> Element Styles v msg
viewCell cell msg = case cell of
  Nothing -> viewRawCell "any" msg
  Just Wall -> viewRawCell "wall" msg
  Just Empty -> viewRawCell "empty" msg
  Just Ghost -> viewRawCell "ghost" msg
  Just Berry -> viewRawCell "berry" msg

cycleCell : Maybe Cell -> Maybe Cell
cycleCell cell = case cell of
  Nothing -> Just Wall
  Just Wall -> Just Empty
  Just Empty -> Just Ghost
  Just Ghost -> Just Berry
  Just Berry -> Nothing

viewNeighbours : Int -> Rule -> Element Styles v Msg
viewNeighbours index rule =
  let
    makeMsg r = ChangeRule { index = index, rule = r }
    upMsg = makeMsg { rule | up = cycleCell rule.up }
    downMsg = makeMsg { rule | down = cycleCell rule.down }
    leftMsg = makeMsg { rule | left = cycleCell rule.left }
    rightMsg = makeMsg { rule | right = cycleCell rule.right }
  in
    Element.table NoStyle
      []
      [ [ Element.empty
        , viewCell rule.left leftMsg
        , Element.empty
        ]
      , [ viewCell rule.up upMsg
        , viewRawCell "pacman" Noop
        , viewCell rule.down downMsg
        ]
      , [ Element.empty
        , viewCell rule.right rightMsg
        , Element.empty
        ]
      ]

cycleBerry : Maybe Berry -> Maybe Berry
cycleBerry berry = case berry of
  Nothing -> Just Taken
  Just Taken -> Just NotTaken
  Just NotTaken -> Nothing

viewBerry : Int -> Rule -> Element Styles v Msg
viewBerry index rule =
  let
    makeMsg r = ChangeRule { index = index, rule = r }
    berryMsg = makeMsg { rule | berry = cycleBerry rule.berry }
    img = case rule.berry of
      Nothing -> "any"
      Just Taken -> "tick"
      Just NotTaken -> "cross"
  in
    Element.row NoStyle
      []
      [ viewRawCell "berry" berryMsg
      , viewRawCell img berryMsg
      ]

stateImg : State -> String
stateImg state = case state of
  A -> "stateA"
  B -> "stateB"
  C -> "stateC"
  D -> "stateD"
  E -> "stateE"
  F -> "stateF"
  G -> "stateG"
  H -> "stateH"

cycleState : Maybe State -> Maybe State
cycleState state = case state of
  Nothing -> Just A
  Just A -> Just B
  Just B -> Just C
  Just C -> Just D
  Just D -> Just E
  Just E -> Just F
  Just F -> Just G
  Just G -> Just H
  Just H -> Nothing

viewState : Int -> Rule -> Element Styles v Msg
viewState index rule =
  let
    makeMsg r = ChangeRule { index = index, rule = r }
    stateMsg = makeMsg { rule | state = cycleState rule.state }
    img = case rule.state of
      Nothing -> "any"
      Just s -> stateImg s
  in
    viewRawCell img stateMsg

cycleMove : Move -> Move
cycleMove move = case move of
  Wait -> Up
  Up -> Right
  Right -> Down
  Down -> Left
  Left -> Wait

viewMove : Int -> Rule -> Element Styles v Msg
viewMove index rule =
  let
    makeMsg r = ChangeRule { index = index, rule = r }
    moveMsg = makeMsg { rule | nextMove = cycleMove rule.nextMove }
    img = case rule.nextMove of
      Wait -> "wait"
      Up -> "up"
      Down -> "down"
      Left -> "left"
      Right -> "right"
  in
    viewRawCell img moveMsg

cycleNextState : State -> State
cycleNextState state = case state of
  A -> B
  B -> C
  C -> D
  D -> E
  E -> F
  F -> G
  G -> H
  H -> A

viewNextState : Int -> Rule -> Element Styles v Msg
viewNextState index rule =
  let
    makeMsg r = ChangeRule { index = index, rule = r }
    stateMsg = makeMsg { rule | nextState = cycleNextState rule.nextState }
  in
    viewRawCell (stateImg rule.nextState) stateMsg

viewControls : Int -> Rule -> Element Styles v Msg
viewControls index rule =
  Element.column NoStyle
    []
    [ viewRawCell "up" (MoveUp index)
    , viewRawCell "cross" (RemoveRule index)
    , viewRawCell "down" (MoveDown index)
    ]

verticalCenter : Element Styles v m -> Element Styles v m
verticalCenter e =
  Element.column NoStyle
    [ Attr.verticalCenter
    , Attr.height (Attr.fill)
    ]
    [ e ]

gap : Element Styles v m
gap = Element.el NoStyle [Attr.width (Attr.px (cellSize / 2))] Element.empty

viewRule : Int -> Rule -> Element Styles v Msg
viewRule index rule =
  Element.row RuleStyle
    [ Attr.spacing 30
    , Attr.padding 10
    , Attr.width Attr.content
    ]
    [ verticalCenter (viewState index rule)
    , viewNeighbours index rule
    , verticalCenter (viewBerry index rule)
    , gap
    , verticalCenter (viewMove index rule)
    , verticalCenter (viewNextState index rule)
    , gap
    , viewControls index rule
    ]

viewProgramRules : Int -> List Rule -> List (Element Styles v Msg)
viewProgramRules firstIndex rules =
  case rules of
    [] -> []
    x :: xs -> viewRule firstIndex x :: viewProgramRules (firstIndex + 1) xs

viewAddButton : Element Styles v Msg
viewAddButton =
  Element.image NoStyle
    [ Attr.width (Attr.px 100)
    , Attr.height (Attr.px 100)
    , Attr.center
    , Events.onClick AddRule
    ]
    { caption = ""
    , src = "/images/plus.png"
    }

viewProgram : Model -> Element Styles v Msg
viewProgram model =
  Element.column NoStyle
    [ Attr.width (Attr.content)
    ]
    [ Element.column NoStyle
      [ Attr.spacing 10 ]
      (viewProgramRules 0 model.rules)
    , viewAddButton
    ]

viewSubmitButton : Element Styles v Msg
viewSubmitButton =
  Element.el ButtonStyle
    [ Attr.padding 10
    , Events.onClick Submit
    ]
    (Element.el TextStyle [] (Element.text "Submit"))

viewSubmitStatus : Model -> Element Styles v m
viewSubmitStatus model =
  let
    text = case model.submit of
      Pending -> "Submitting..."
      NotStarted -> ""
      Finished Success -> "OK"
      Finished RateLimitExceeded -> "Blocked: rate limited"
      Finished LevelClosed -> "Level closed"
      Finished Unauthorised -> "Unauthorized"
      Finished (Fail msg) -> msg
  in
    Element.el TextStyle [ Attr.padding 10 ] (Element.text text)

viewSubmit : Model -> Element Styles v Msg
viewSubmit model =
  Element.column NoStyle
    [ Attr.alignTop
    , Attr.width Attr.fill
    ]
    [ Element.row NoStyle
      [ Attr.width Attr.fill
      , Attr.alignRight
      , Attr.spacing 20
      ]
      [ viewSubmitStatus model
      , viewSubmitButton
      ]
    ]

viewEditor : Model -> Element Styles v Msg
viewEditor model =
  Element.row NoStyle
    [ Attr.padding 10 ]
    [ viewProgram model
    , viewSubmit model
    ]

view : Model -> Html Msg
view model = Element.layout styleSheet (viewEditor model)
