FROM rust:latest

RUN apt update && apt install libclang-dev -y