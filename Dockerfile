# Use a Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app
RUN apt update && apt install lld clang -y

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml ./
# COPY Cargo.lock ./

# Copy the rest of your source code
COPY . .

# Build your Rust application
RUN cargo build --release

# Create a minimal runtime image
FROM debian:bookworm-slim

# Set the working directory inside the container
WORKDIR /lqrl-back

# Copy the built binary from the builder stage to the final image
COPY --from=builder /app/target/release/web-server .
COPY --from=builder /app/sql .
COPY --from=builder /app/target/release/exercise_cron_job .
COPY --from=builder /app/entrypoint.sh .

RUN chmod +x entrypoint.sh

# Expose the port your Axum-web application will listen on
EXPOSE 8888

RUN mkdir -p public/uploads

CMD ["./entrypoint.sh"]