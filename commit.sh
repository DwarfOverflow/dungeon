git add *
git commit
git push origin master

# Web
trunk build --release
rm /home/simon/projet/dwarfoverflow.github.io/index.html
rm /home/simon/projet/dwarfoverflow.github.io/*.wasm
rm /home/simon/projet/dwarfoverflow.github.io/*.js

cp dist/* /home/simon/projet/dwarfoverflow.github.io/
cd /home/simon/projet/dwarfoverflow.github.io/
code .