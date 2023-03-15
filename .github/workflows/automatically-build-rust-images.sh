#!/bin/bash -e

DOCKER_TOKEN=$(
  curl -s 'https://auth.docker.io/token?service=registry.docker.io&scope=repository:trunnion/cargo-acap:pull' | \
  jq -r .access_token
  )
DOCKER_TAGS=$(
  curl -s -H "Authorization: Bearer $DOCKER_TOKEN" https://registry-1.docker.io/v2/trunnion/cargo-acap/tags/list | \
  jq -r '.tags | .[]'
  )

curl -s https://api.github.com/repos/rust-lang/rust/releases | \
  jq -r 'limit(2; sort_by(.created_at) | reverse | .[] | select(.prerelease==false) | select(.created_at >= "2022-01-01") | .tag_name)' | \
  uniq | \
  while read -r RUST_TAG
  do
    if [[ "$DOCKER_TAGS" == *"$RUST_TAG"* ]]
    then
      echo "image for Rust ${RUST_TAG} already exists"
    else
      echo "starting build for Rust ${RUST_TAG}"
      echo "$RUST_TAG" | jq -R '{ref: "master", inputs: {rustVersion: .}}' | \
        curl \
          -X POST \
          -H "Accept: application/vnd.github.v3+json" \
          -H "Content-Type: application/json" \
          -H "Authorization: Bearer $GITHUB_TOKEN" \
          https://api.github.com/repos/trunnion/cargo-acap/actions/workflows/build-rust-image.yml/dispatches \
          -d @-
    fi
  done
