FROM rustlang/rust:nightly

RUN dpkg --add-architecture arm64
RUN apt update && apt upgrade -y
RUN apt install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross pkg-config

RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup toolchain install nightly-aarch64-unknown-linux-gnu

WORKDIR /app

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ 

CMD ["cargo", "build", "-r", "--target", "aarch64-unknown-linux-gnu"]
