module Editor exposing (..)

import Browser
import Browser.Navigation as Navigation
import Http
import Html exposing (Html)
import Url.Builder
import Json.Encode as Encode exposing (Value)
import Element exposing (Element)
import Element.Attributes as Attr
import Element.Events as Events
import Element.Input as Input
import Style
import Style.Border as Border
import Style.Color as Color
import Style.Font as Font

type alias Model =
  { name : String
  , password : String
  , message : String
  }

type Msg
  = Noop
  | SetName String
  | SetPassword String
  | Login
  | LoadEditor
  | Error String

main : Program () Model Msg
main = Browser.document
  { init = always (init, Cmd.none)
  , view = \model -> { title = "Login", body = [view model] }
  , update = update
  , subscriptions = always Sub.none
  }

init : Model
init = { name = "", password = "", message = "" }

showHttpError : Http.Error -> String
showHttpError err =
  case err of
    Http.BadUrl url ->
      "bad request url: " ++ url
    Http.Timeout ->
      "request timeout"
    Http.NetworkError ->
      "network error"
    Http.BadStatus 401 ->
      "incorrect credentials"
    Http.BadStatus status ->
      "response status: " ++ String.fromInt status
    Http.BadBody body ->
      "bad response body: " ++ body

encodeRequest : Model -> Value
encodeRequest model =
  Encode.object
    [ ("user", Encode.string model.name)
    , ("password", Encode.string model.password)
    ]

submit : Model -> Cmd Msg
submit model =
  let
    handleResult result =
      case result of
        Ok () -> LoadEditor
        Err err -> Error (showHttpError err)
  in
    Http.post
      { url = Url.Builder.absolute [ "api", "authenticate"] []
      , body = Http.jsonBody (encodeRequest model)
      , expect = Http.expectWhatever handleResult
      }

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    Noop ->
      (model, Cmd.none)
    SetName name ->
      ({ model | name = name }, Cmd.none)
    SetPassword password ->
      ({ model | password = password }, Cmd.none)
    Login ->
      (model, submit model)
    LoadEditor ->
      (model, Navigation.load "/editor.html")
    Error err ->
      ({ model | message = err }, Cmd.none)

type Styles
  = NoStyle
  | TextStyle
  | ErrorTextStyle
  | InputStyle

styleSheet =
  Style.styleSheet
    [ Style.style NoStyle []
    , Style.style TextStyle
      [ Font.size 30
      , Font.typeface [Font.sansSerif]
      ]
    , Style.style ErrorTextStyle
      [ Font.size 30
      , Font.typeface [Font.sansSerif]
      , Color.text (Style.rgb 1 0.3 0.3)
      ]
    , Style.style InputStyle
      [ Border.solid
      , Border.all 1
      , Color.border (Style.rgb 0 0 0)
      , Font.size 30
      , Font.typeface [Font.sansSerif]
      ]
    ]

viewLogin : Model -> Element Styles v Msg
viewLogin model =
  Element.row NoStyle
    [ Attr.padding 10
    , Attr.spacing 10
    ]
    [ Element.column NoStyle
      [ Attr.padding 10
      , Attr.spacing 10
      , Attr.width Attr.content
      ]
      [ Element.el TextStyle [] (Element.text "User")
      , Element.el TextStyle [] (Element.text "Password")
      ]
    , Element.column NoStyle
      [ Attr.padding 10
      , Attr.spacing 10
      , Attr.width Attr.fill
      ]
      [ Input.text InputStyle
        []
        { onChange = SetName
        , value = model.name
        , label = Input.hiddenLabel "user"
        , options = []
        }
      , Input.currentPassword InputStyle
        []
        { onChange = SetPassword
        , value = model.password
        , label = Input.hiddenLabel "password"
        , options = []
        }
      , Element.button TextStyle
        [ Attr.padding 3
        , Attr.width Attr.content
        , Events.onClick Login
        ]
        (Element.text "Login")
      ]
    ]

wrapLogin : Model -> Element Styles v Msg
wrapLogin model =
  Element.el NoStyle
    [ Attr.center
    , Attr.padding 200
    ]
    (Element.column NoStyle []
      [ viewLogin model
      , Element.el ErrorTextStyle
        [ Attr.center ]
        (Element.text model.message)
      ])

view : Model -> Html Msg
view model = Element.layout styleSheet (wrapLogin model)
