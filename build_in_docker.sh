#!/bin/bash

sudo docker run --rm -it -v "$PWD:/src:ro" -v "$PWD/target-docker:/target-docker" rust:1.52-buster bash -c '
    set -ex
    cd /src
    cargo build --release --target-dir /target-docker || true; bash
'

