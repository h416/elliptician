#!/bin/bash

if [ $# -eq 0 ]; then
  echo "usage ./task.sh |check|test|build|" 1>&2
  exit 1
fi

if [ $1 = "check" ] ; then
  cargo fmt && cargo check
elif [ $1 = "test" ]; then
  cargo test
elif [ $1 = "build" ]; then
  export RUSTFLAGS="-C opt-level=3 -C target-cpu=native"
  cargo build --release
else
  echo "unknown command"
fi