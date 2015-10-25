#!/bin/bash

set -e

eval "$(ssh-agent -s)"

openssl aes-256-cbc -K $encrypted_c3811c6aa805_key -iv $encrypted_c3811c6aa805_iv -in .travis/deploy_key.pem.enc -out .travis/deploy_key.pem -d
chmod 600 .travis/deploy_key.pem
ssh-add .travis/deploy_key.pem

cargo doc
cd target/doc

echo '<meta http-equiv=refresh content=0;url=geo/index.html>' > index.html
touch .nojekyll

git init
git config user.name "Travis"
git config user.email "noreply@travis-ci.org"
git add .
git commit -m "Deploy to GitHub Pages"
git push --force --quiet git@github.com:${TRAVIS_REPO_SLUG}.git master:gh-pages > /dev/null 2>&1
