git add *
git commit
git push origin master

# Web
trunk build --release
rm /home/simon/projet/naincroyable.github.io/index.html
rm /home/simon/projet/naincroyable.github.io/*.wasm
rm /home/simon/projet/naincroyable.github.io/*.js

cp dist/* /home/simon/projet/naincroyable.github.io/
cd /home/simon/projet/naincroyable.github.io/
code .