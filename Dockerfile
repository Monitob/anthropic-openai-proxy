FROM alpine:3.18

RUN apk --no-cache add ca-certificates
WORKDIR /app
RUN addgroup -S app && adduser -S app -G app
USER app

# Copy the pre-built binary
COPY target/release/codex-router .

EXPOSE 8787
CMD ["./codex-router"]