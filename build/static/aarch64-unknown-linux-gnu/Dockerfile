FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y yasm wget xz-utils build-essential git gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross libc-dev-arm64-cross gcc-multilib-arm-linux-gnueabihf software-properties-common

RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-add-repository "deb http://apt.llvm.org/focal/ llvm-toolchain-focal main"
RUN apt-get update
RUN apt-get install -y clang