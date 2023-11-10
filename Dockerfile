FROM rust:latest

RUN apt update && apt install libclang-dev git -y

RUN git clone --recursive https://github.com/FuzzingLabs/sui-fuzzer.git

RUN cd sui-fuzzer && cargo build --release