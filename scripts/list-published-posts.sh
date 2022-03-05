#!/bin/sh

set -e

grpcurl -plaintext \
  -import-path ../myblog-proto \
  -proto proto/blog/service.proto \
  -d '{"offset": 0, "limit": 1}' \
  localhost:8082 \
  myblog.proto.blog.BlogService/ListPublishedPosts