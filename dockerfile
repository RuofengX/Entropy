####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/*
RUN sed -i 's/security.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list.d/*
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /entropy

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

WORKDIR /entropy

# Copy our build
COPY --from=builder /entropy/target/x86_64-unknown-linux-musl/release/entropy-game ./

EXPOSE 80
CMD ["/entropy/entropy-game"]
