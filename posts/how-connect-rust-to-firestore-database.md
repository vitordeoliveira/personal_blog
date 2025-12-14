---
title: "How connect rust to firestore database"
subtitle: "parsing a markdown file metadata into a struct"
description: "this tutorial walks you through the practice of parsing markdown files for metadata"
tags: ["markdown", "rust", "files", "parsing", "metadata"]
date: "2021-09-13t03:48:00"
ready: false
---

# How connect rust to firestore database

Requirements:

Java, NodeJs, Rust, GCloud

here I start the fake

npm i -g firebase-tools

firebase login

you need to have java installed
asdf plugin add java
asdf install java openjdk-22
asdf global java openjdk-22

asdf plugin add gcloud
asdf install gcloud 470.0.0
asdf global gcloud 470.0.0

MAC:
brew install --cask google-cloud-sdk

gcloud auth application-default login

firebase init

firebase init emulators

firebase emulators:start

cargo add firestore serde tokio-stream futures gcloud-sdk

cargo add tokio -F full

- '--network=cloudbuild'

# builder (lukemathwalker/cargo-chef:latest-rust-1-slim-bullseye)

RUN apt-get update && apt-get install -y ca-certificates

# runner (debian:bullseye-slim)

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
