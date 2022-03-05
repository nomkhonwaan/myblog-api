#!/bin/sh

set -e

grpcurl -plaintext \
  -import-path ../myblog-proto \
  -proto proto/blog/service.proto \
  -d '{"category":{"id": "5b2863365c31b411b041995e"}, "offset": 0, "limit": 1}' \
  localhost:8082 \
  myblog.proto.blog.BlogService/ListCategoryPublishedPosts