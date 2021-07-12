#!/bin/sh

set -e

DATE=$(date +'%Y-%m-%d')
TIME=$(date +'%H:%m:%S')
export DATETIME="${DATE}T${TIME}.000Z"

TEMPLATE='{"comment": {"text": "Hello, world!", "createdAt": "${DATETIME}", "status": "Published"}}'
BODY=$(echo $TEMPLATE | envsubst)

grpcurl -plaintext \
  -import-path ../myblog-proto \
  -proto proto/discussion/service.proto \
  -d "${BODY}" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  localhost:8080 \
  myblog.proto.discussion.DiscussionService/CreateComment