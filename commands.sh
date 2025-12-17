tribune_test() {
  if [ ! -f Dockerfile ]; then
    echo "No Dockerfile in $(pwd)"
    return 1
  fi

  local name port
  name="$(basename "$PWD")"
  port="$(awk '/^EXPOSE / {print $2; exit}' Dockerfile)"

  if [ -z "$port" ]; then
    echo "No EXPOSE found in Dockerfile"
    return 1
  fi

  docker build -t "lumilukko/${name}" . && \
  docker run \
    --rm \
    -p "${port}:${port}" \
    "$@" \
    "lumilukko/${name}"
}

tribune_build(){
    if [ ! -f Dockerfile ]; then
    echo "No Dockerfile in $(pwd)"
    return 1
  fi

  local name
  name="$(basename "$PWD")"

  docker build -t "lumilukko/${name}" . && \
  docker push "lumilukko/${name}"
}