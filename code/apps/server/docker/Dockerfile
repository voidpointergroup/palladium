FROM rust:1.61.0 as build-env
ARG toolchain
ARG version
RUN rustup default $toolchain

WORKDIR /code
COPY ./.rust ./.rust
COPY ./libs ./libs
COPY ./apps/server ./apps/server
RUN ls -alghR /code
WORKDIR /code/apps/server
RUN sed 's/version = "0.0.0"/version = "'$version'"/g' Cargo.toml > Cargo.toml.tmp
RUN mv Cargo.toml.tmp Cargo.toml
RUN cargo build --release

FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=build-env /code/apps/server/target/release/palladium-server .
USER 1000
CMD ["./palladium-server", "serve", "-c", "./config/config.yaml"]
