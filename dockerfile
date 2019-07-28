# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust or OpenSSL.
ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Fix permissions on source code.
# RUN sudo chown -R rust:rust /home/rust

# See http://whitfin.io/speeding-up-rust-docker-builds/
# Compiles a project which has the same dependencies as our source code
# This fixes caching so that dependencies are rebuilt only when they are changed
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
# Remove unneeded source
RUN rm src/*.rs
# Without removing these files, cargo won't rebuild with our new source code
RUN rm target/x86_64-unknown-linux-musl/release/deps/food_ordering_bot*

# Add our actual source and build
COPY ./src ./src
RUN cargo build --release

# Now, we need to build our real Docker container, copying in `food-ordering-bot`.
FROM alpine:latest
# for cert validation
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/food-ordering-bot \
    /usr/local/bin/
CMD /usr/local/bin/food-ordering-bot