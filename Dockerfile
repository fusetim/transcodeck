# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the Rust executable
RUN cargo build --release --locked

# Create a new stage for the final image
FROM lscr.io/linuxserver/ffmpeg:latest

LABEL org.opencontainers.image.source "https://github.com/fusetim/transcodeck"
LABEL org.opencontainers.image.title "transcodeck"
LABEL org.opencontainers.image.description "A simple utility to transcode faster media files using a cluster of machines."
LABEL org.opencontainers.image.url "https://github.com/fusetim/transcodeck"
LABEL org.opencontainers.image.authors "Fusetim <fusetim.log@gmx.com>"

# Install the required dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
        /var/tmp/*

# Set the working directory inside the container
WORKDIR /app

# Copy the built executable from the builder stage to the final image
COPY --from=builder /app/target/release/transcodeck .

# Set the entry point for the container
ENTRYPOINT ["./transcodeck"]