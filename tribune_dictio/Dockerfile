FROM nvidia/cuda:12.1.1-runtime-ubuntu22.04
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# Copy your app
COPY pyproject.toml .
COPY main.py .
COPY init.py .


RUN apt-get update && apt-get install -y --no-install-recommends \
    wget ca-certificates g++ make python3.11-dev python3.11-distutils\
    && rm -rf /var/lib/apt/lists/*

RUN uv sync

EXPOSE 8000


CMD ["uv", "run", "main.py"]
