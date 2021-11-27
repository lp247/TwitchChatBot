FROM rust:latest as builder
WORKDIR /usr/src/chatbot
COPY ./chatbot .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && update-ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/chatbot /usr/local/bin/chatbot
CMD ["chatbot"]
