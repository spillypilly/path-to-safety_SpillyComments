#!/bin/sh
docker run --rm -it -v $(pwd):/app -w /app denoland/deno "$@"
