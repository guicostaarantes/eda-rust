FROM rust:1.69 AS build
WORKDIR /app
COPY ./Cargo.lock ./Cargo.lock 
COPY ./Cargo.toml ./Cargo.toml
RUN mkdir -p src/bin
RUN echo 'fn main() {}' > src/bin/dummy.rs
RUN cargo build --release
RUN rm -rf src/bin
COPY ./src ./src
RUN cargo build --release

FROM debian:bullseye-slim AS final
COPY --from=build /app/target/release/auth-msvc .
CMD ["./auth-msvc"]
