#!/bin/bash

set -e

cargo doc
cd target/doc

echo '<meta http-equiv=refresh content=0;url=geo/index.html>' > index.html
touch .nojekyll

git init
git config user.name "Travis"
git config user.email "noreply@travis-ci.org"
git add .
git commit -m "Deploy to GitHub Pages"
git push --force --quiet https://${TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git master:gh-pages > /dev/null 2>&1
