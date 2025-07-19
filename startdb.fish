#!/usr/bin/env fish

set -x DATABASE_URL "postgres://postgres@localhost:5432/openfrontpro"
docker run --rm -p 5432:5432 -e POSTGRES_HOST_AUTH_METHOD=trust postgres:latest
