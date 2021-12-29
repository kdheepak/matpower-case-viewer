#!/usr/bin/env sh
# abort on errors
set -e
# build
rm -rf dist
NODE_ENV=production npm run wasm && npm run build
# navigate into the build output directory
cd dist
# if you are deploying to a custom domain
# echo 'matpower-case-viewer.kdheepak.com' >CNAME
git init
git add -A
git commit -m 'deploy' -n
git push -f git@github.com:kdheepak/matpower-case-viewer.git main:gh-pages
cd -
