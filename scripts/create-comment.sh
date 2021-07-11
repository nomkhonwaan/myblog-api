#!/bin/sh

set -e

grpcurl -plaintext \
  -import-path ../myblog-proto \
  -proto proto/discussion/service.proto \
  -d '{"comment":{"text":"Hello, world!","createdAt":"2020-07-14T01:47:53.188Z","status":"Published"}}' \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  localhost:8080 \
  myblog.proto.discussion.DiscussionService/CreateComment