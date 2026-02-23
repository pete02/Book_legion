
FROM golang:1.25-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git gcc musl-dev sqlite-dev

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download
COPY . .

# Build with CGO enabled
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /app/app .

FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/app .

# Expose port
EXPOSE 8000

ENV DB_TYPE=sqlite
ENV DB_PATH=/data/storage.db
ENV TTS_BACKEND=mock

# Set entrypoint
ENTRYPOINT ["/app/app"]
