#!/bin/bash
set -o errexit -o nounset

GITHUB_REPO_SLUG=${1:?'Must provide GitHub repository slug ("owner/repo")'}
# See https://help.github.com/articles/creating-an-access-token-for-command-line-use/
GITHUB_TOKEN=${2:?'Must provide GitHub access token'}


# Build docs with default options for current directory
cargo doc --verbose

# Reuse Cargo's build directory instead of creating a temporary one
pushd target/doc/

# We do not have an index page (yet), so create a redirect to main module documentation
echo '<meta http-equiv=refresh content=0;url=chip8_vm/index.html>' > index.html

# Overwrite gh-pages branch with rustdoc
git init
git add --all .
git commit --message='Update documentation'
git push --quiet --force "https://${GITHUB_TOKEN}@github.com/${GITHUB_REPO_SLUG}.git" master:gh-pages

# Cleanup, restore current directory
popd
