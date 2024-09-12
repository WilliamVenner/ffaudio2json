FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y wget xz-utils build-essential git gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross libc-dev-arm64-cross gcc-multilib-arm-linux-gnueabihf software-properties-common

RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN apt-add-repository "deb http://apt.llvm.org/focal/ llvm-toolchain-focal main"
RUN apt-get update
RUN apt-get install -y clang

RUN wget -O /tmp/ffmpeg-n7.0-latest-linuxarm64-lgpl-shared-7.0.tar.xz https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-linuxarm64-lgpl-shared-7.0.tar.xz
RUN tar -xf /tmp/ffmpeg-n7.0-latest-linuxarm64-lgpl-shared-7.0.tar.xz -C /tmp --overwrite
ENV FFMPEG_DIR="/tmp/ffmpeg-n7.0-latest-linuxarm64-lgpl-shared-7.0"