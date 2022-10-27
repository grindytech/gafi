#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Start Gafi Node ***"

cd $(dirname ${BASH_SOURCE[0]})/..
mkdir -p .local

docker-compose down --remove-orphans
docker-compose pull
docker-compose run --rm --service-ports gafi $@
