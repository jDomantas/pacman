mkdir static

cd pacman-web-editor
elm make src/Editor.elm --optimize --output ../static/editor.html

cd pacman-web-login
elm make src/Login.elm --optimize --output ../static/login.html

cd ..
rm -r static/images
cp -r pacman-web-editor/images static/images
