#!/bin/sh

set -e

grpcurl -plaintext \
  -import-path ../myblog-proto \
  -proto proto/blog/service.proto \
  -d '{"category":{"id": "5f0d384fbb5a7bb644623cb2"}, "offset": 0, "limit": 1}' \
  localhost:8082 \
  myblog.proto.blog.BlogService/ListCategoryPublishedPosts