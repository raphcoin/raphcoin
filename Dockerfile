FROM phusion/baseimage:0.10.2 as builder

ENV TERM=xterm
ARG PROJECT=raphcoin
ARG PROFILE=release
WORKDIR /substrate

RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	cargo install --git https://github.com/alexcrichton/wasm-gc

COPY . .

RUN PATH="$PATH:$HOME/.cargo/bin" && \
    rustup default nightly && \
	./scripts/build.sh && \
	rustup default stable
RUN PATH="${PATH}:${HOME}/.cargo/bin" && \
    cargo build --$PROFILE && \
	mv ./target/$PROFILE/$PROJECT /app

FROM phusion/baseimage:0.10.2
COPY --from=builder /app .
ENTRYPOINT ["/app"]
