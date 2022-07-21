#!/bin/bash
docker build -f docker/gafi-node.dockerfile  -t $1 .
docker push $1
