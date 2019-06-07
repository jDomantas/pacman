mkdir static

cd pacman-web-editor
elm make src/Main.elm --optimize --output ../static/editor.html

cd pacman-web-login
elm make src/Main.elm --optimize --output ../static/login.html

cd ..
rm -r static/images
cp -r pacman-web-editor/images static/images
