FROM nvidia/cuda:12.1.1-runtime-ubuntu22.04
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# Copy your app
COPY pyproject.toml .
COPY main.py .
COPY init.py .


RUN apt-get update && apt-get install -y --no-install-recommends \
    wget ca-certificates g++ make python3.11-dev python3.11-distutils python3.11-venv \
    && rm -rf /var/lib/apt/lists/*

RUN uv sync --python 3.11
RUN uv run python -m ensurepip --upgrade
RUN uv run python -m pip install "setuptools<70"

RUN uv run python -c "import setuptools; print(setuptools.__version__);import pkg_resources; print('pkg_resources OK')"

EXPOSE 8000


CMD ["uv", "run", "main.py"]
