FROM rust:latest

RUN apt update && apt install libclang-dev git -y

RUN git clone --recursive git@github.com:FuzzingLabs/sui-fuzzer.git

RUN cd sui-fuzzer && make CONFIG_PATH="./config.json" TARGET_FUNCTION="fuzzinglabs"