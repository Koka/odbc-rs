#!/bin/bash
set -e
PATH=~/.cargo/bin/:$PATH
git fetch
git checkout master
git reset --hard origin/master
cargo clean
cargo doc
TMP_DIR=`mktemp -d`
mv ./target/doc/* "$TMP_DIR/"
git checkout gh-pages
rm -rf ./*
mv "$TMP_DIR"/* ./
git add .
git commit -am "Update docs gh-pages"
git checkout master
rm -rf "$TMP_DIR"

