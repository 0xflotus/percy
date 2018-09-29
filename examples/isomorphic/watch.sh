#!/bin/bash

cd $(dirname $0)

cd ./client

./build-wasm.watch.sh &
systemfd --no-pid -s http::7878 -- cargo +nightly watch -w ../server -w ../app -x 'run -p isomorphic-server' &
wait
