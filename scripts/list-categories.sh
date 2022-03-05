#!/bin/sh

set -e

grpcurl -plaintext \
  -import-path ../myblog-proto \
  -proto proto/blog/service.proto \
  localhost:8082 \
  myblog.proto.blog.BlogService/ListCategories