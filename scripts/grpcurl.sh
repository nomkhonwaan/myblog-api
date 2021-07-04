#!/bin/sh

set -e

grpcurl -plaintext \
  -import-path ~/Workspaces/myblog-proto \
  -proto proto/blog/service.proto \
  -d '{"offset": 0, "limit": 1}' \
  localhost:8080 \
  myblog.proto.blog.BlogService/ListPublishedPosts