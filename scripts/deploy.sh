#!/usr/bin/env sh
# abort on errors
set -e
# build
rm -rf dist
npm run build
# navigate into the build output directory
cd dist
# if you are deploying to a custom domain# echo 'www.example.com' > CNAME
git init
git add -A
git commit -m 'deploy' -n
git push -f git@github.com:kdheepak/matpower-case-viewer.git main:gh-pages
cd -
