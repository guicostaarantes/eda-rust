FROM rust:1.67 AS build
WORKDIR /app
COPY ./Cargo.lock ./Cargo.lock 
COPY ./Cargo.toml ./Cargo.toml
RUN mkdir -p src/bin
RUN echo 'fn main() {}' > src/bin/dummy.rs
RUN cargo build --release
COPY ./src ./src
RUN cargo build --release

FROM debian:bullseye-slim AS final
RUN apt update
RUN apt install -y curl
COPY --from=build /app/target/release/auth-msvc .
HEALTHCHECK CMD curl http://localhost:8000 --data-raw '{"query":"{__typename}"}'
CMD ["./auth-msvc"]
