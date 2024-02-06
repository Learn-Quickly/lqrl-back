# Use a Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml ./
# COPY Cargo.lock ./

# Copy the rest of your source code
COPY .. .

# Build your Rust application
RUN cargo build --release

# Create a minimal runtime image
FROM debian:bookworm-slim

# Set the working directory inside the container
WORKDIR /lqrl-back

# Copy the built binary from the builder stage to the final image
COPY --from=builder /app/target/release/web-server .

# Expose the port your Axum-web application will listen on
EXPOSE 8888

# # Define the command to run your application
CMD ["./lqrl-back"]