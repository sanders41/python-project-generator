use anyhow::{bail, Result};

use crate::{
    file_manager::save_file_with_content,
    project_info::{ProjectInfo, ProjectManager, Pyo3PythonManager},
};

fn create_dockercompose_file(project_info: &ProjectInfo) -> String {
    let base_name = &project_info.project_slug;

    format!(
        r#"services:
  backend:
    image: {base_name}-backend:latest
    restart: unless-stopped
    networks:
      - traefik-public-{base_name}
      - default
    build:
      context: .
    container_name: {base_name}-backend
    healthcheck:
      test: ["CMD", "curl", "-f", "http://127.0.0.1:8000/api/v1/health"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    depends_on:
      db:
        condition: service_healthy
        restart: true
      valkey:
        condition: service_healthy
        restart: true
      migrations:
        condition: service_completed_successfully
    env_file:
      - .env
    environment:
      - POSTGRES_HOST=db
      - VALKEY_HOST=valkey
    labels:
      - traefik.enable=true
      - traefik.docker.network=traefik-public-{base_name}
      - traefik.constraint-label=traefik-public

      - traefik.http.services.${{STACK_NAME?Variable not set}}-backend.loadbalancer.server.port=8000

      # Rate limiting middleware
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-api-rate-limit.ratelimit.burst=50
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-api-rate-limit.ratelimit.average=25

      # Security headers middleware (backend-specific)
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.contenttypenosniff=true
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.referrerpolicy=strict-origin-when-cross-origin
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.forcestsheader=true
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.stsincludesubdomains=true
      - traefik.http.middlewares.${{STACK_NAME?Variable not set}}-security-headers.headers.stsseconds=31536000

      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-http.rule=Host(`api.${{DOMAIN?Variable not set}}`)
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-http.entrypoints=http

      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.rule=Host(`api.${{DOMAIN?Variable not set}}`) - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.entrypoints=https - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.tls=true
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.tls.certresolver=le

      # Enable redirection for HTTP and HTTPS
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-http.middlewares=https-redirect

      # Enable rate limiting and security headers
      - traefik.http.routers.${{STACK_NAME?Variable not set}}-backend-https.middlewares=${{STACK_NAME?Variable not set}}-api-rate-limit,${{STACK_NAME?Variable not set}}-security-headers

  db:
    image: postgres:18-alpine
    restart: unless-stopped
    container_name: {base_name}-db
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $POSTGRES_USER -d $POSTGRES_DB"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    expose:
      - 5432
    env_file:
      - .env
    environment:
      - POSTGRES_PASSWORD=${{POSTGRES_PASSWORD?Variable not set}}
      - POSTGRES_USER=${{POSTGRES_USER?Variable not set}}
      - POSTGRES_DB=${{POSTGRES_DB?Variable not set}}
    volumes:
      - {base_name}-db-data:/var/lib/postgresql/data

  valkey:
    image: valkey/valkey:8-alpine
    restart: unless-stopped
    container_name: {base_name}-valkey
    healthcheck:
      test:
        [
          "CMD",
          "valkey-cli",
          "--no-auth-warning",
          "-a",
          "${{VALKEY_PASSWORD?Variable not set}}",
          "ping",
        ]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    expose:
      - 6379
    env_file:
      - .env
    command: valkey-server --requirepass ${{VALKEY_PASSWORD?Variable not set}}
    volumes:
      - {base_name}-valkey-data:/var/lib/valkey/data

  migrations:
    image: ghcr.io/sanders41/sqlx-migration-runner:1
    container_name: {base_name}-migrations
    env_file:
      - .env
    environment:
      - POSTGRES_HOST=db
      - DATABASE_URL=postgresql://${{POSTGRES_USER}}:${{POSTGRES_PASSWORD}}@db:5432/${{POSTGRES_DB}}
    depends_on:
      db:
        condition: service_healthy
        restart: true
    volumes:
      - ./migrations:/migrations

volumes:
  {base_name}-db-data:
  {base_name}-valkey-data:

networks:
  traefik-public-{base_name}:
    name: traefik-public-{base_name}
    # Allow setting it to false for testing
    external: true
"#
    )
}

pub fn save_dockercompose_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("docker-compose.yml");
    let file_content = create_dockercompose_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockercompose_override_file(project_info: &ProjectInfo) -> String {
    let base_name = &project_info.project_slug;

    format!(
        r#"services:
  proxy:
    image: traefik:3
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    ports:
      - "80:80"
      - "8090:8080"
    # Duplicate the command from docker-compose.yml to add --api.insecure=true
    command:
      # Enable Docker in Traefik, so that it reads labels from Docker services
      - --providers.docker
      # Add a constraint to only use services with the label for this stack
      - --providers.docker.constraints=Label(`traefik.constraint-label`, `traefik-public`)
      # Do not expose all Docker services, only the ones explicitly exposed
      - --providers.docker.exposedbydefault=false
      # Create an entrypoint "http" listening on port 80
      - --entrypoints.http.address=:80
      # Enable the access log, with HTTP requests
      - --accesslog
      # Enable the Traefik log, for configurations and errors
      - --log
      # Enable debug logging for local development
      - --log.level=DEBUG
      # Enable the Dashboard and API
      - --api
      # Enable the Dashboard and API in insecure mode for local development
      - --api.insecure=true
    labels:
      # Enable Traefik for this service, to make it available in the public network
      - traefik.enable=true
      - traefik.constraint-label=traefik-public
    networks:
      - traefik-public-{base_name}
      - default

  backend:
    image: {base_name}-backend
    restart: no
    healthcheck:
      test: ["CMD", "curl", "-f", "http://127.0.0.1:8000/api/v1/health"]
      interval: 10s
      retries: 5
      start_period: 10s
      timeout: 5s
    ports:
      - "8000:8000"
    networks:
      - traefik-public-{base_name}
      - default
    build:
      context: .
    container_name: {base_name}-backend
    depends_on:
      db:
        condition: service_healthy
        restart: true
      valkey:
        condition: service_healthy
        restart: true
    env_file:
      - .env
    environment:
      - SECRET_KEY=someKey
      - POSTGRES_HOST=db
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=test_password
      - VALKEY_HOST=valkey
      - VALKEY_PASSWORD=test_password
      - ENVIRONMENT=local
    labels:
      - traefik.enable=true
      - traefik.docker.network=traefik-public-{base_name}
      - traefik.constraint-label=traefik-public
      - traefik.http.services.${{STACK_NAME:-{base_name}}}-backend.loadbalancer.server.port=8000
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-http.rule=Host(`api.127.0.0.1`)
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-http.entrypoints=http
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.rule=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.entrypoints=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.tls=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-https.tls.certresolver=
      - traefik.http.routers.${{STACK_NAME:-{base_name}}}-backend-http.middlewares=

  db:
    restart: "no"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $POSTGRES_USER -d $POSTGRES_DB"]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    ports:
      - "5432:5432"

  valkey:
    restart: "no"
    # By default only 16 databases are allowed. Bumping this just for testing so that tests can
    # run in parallel without impacting each other
    command: valkey-server --requirepass test_password --databases 100
    healthcheck:
      test:
        [
          "CMD",
          "valkey-cli",
          "--no-auth-warning",
          "-a",
          "${{VALKEY_PASSWORD?Variable not set}}",
          "ping",
        ]
      interval: 10s
      retries: 5
      start_period: 30s
      timeout: 10s
    ports:
      - 6379:6379

networks:
  traefik-public-{base_name}:
    # For local dev, don't expect an external Traefik network
    external: false
"#
    )
}

pub fn save_dockercompose_override_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("docker-compose.override.yml");
    let file_content = create_dockercompose_override_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockercompose_traefik_file(project_info: &ProjectInfo) -> String {
    let base_name = &project_info.project_slug;

    format!(
        r#"services:
  traefik:
    image: traefik:3
    container_name: {base_name}-traefik
    ports:
      # Listen on port 80, default for HTTP, necessary to redirect to HTTPS
      - 80:80
      # Listen on port 443, default for HTTPS
      - 443:443
    restart: unless-stopped
    env_file:
      - .env
    labels:
      # Enable Traefik for this service, to make it available in the public network
      - traefik.enable=true
      # Use the traefik-public network (declared below)
      - traefik.docker.network=traefik-public
      # Define the port inside of the Docker service to use
      - traefik.http.services.traefik-dashboard.loadbalancer.server.port=8080
      # Make Traefik use this domain (from an environment variable) in HTTP
      - traefik.http.routers.traefik-dashboard-http.entrypoints=http
      - traefik.http.routers.traefik-dashboard-http.rule=Host(`traefik.${{DOMAIN?Variable not set}}`)
      # traefik-https the actual router using HTTPS
      - traefik.http.routers.traefik-dashboard-https.entrypoints=https
      - traefik.http.routers.traefik-dashboard-https.rule=Host(`traefik.${{DOMAIN?Variable not set}}`)
      - traefik.http.routers.traefik-dashboard-https.tls=true
      # Use the "le" (Let's Encrypt) resolver created below
      - traefik.http.routers.traefik-dashboard-https.tls.certresolver=le
      # Use the special Traefik service api@internal with the web UI/Dashboard
      - traefik.http.routers.traefik-dashboard-https.service=api@internal
      # https-redirect middleware to redirect HTTP to HTTPS
      - traefik.http.middlewares.https-redirect.redirectscheme.scheme=https
      - traefik.http.middlewares.https-redirect.redirectscheme.permanent=true
      # traefik-http set up only to use the middleware to redirect to https
      - traefik.http.routers.traefik-dashboard-http.middlewares=https-redirect
      # admin-auth middleware with HTTP Basic auth
      # Using the environment variables USERNAME and HASHED_PASSWORD
      - traefik.http.middlewares.admin-auth.basicauth.users=${{USERNAME?Variable not set}}:${{HASHED_PASSWORD?Variable not set}}
      # Enable HTTP Basic auth, using the middleware created above
      - traefik.http.routers.traefik-dashboard-https.middlewares=admin-auth
    volumes:
      # Add Docker as a mounted volume, so that Traefik can read the labels of other services
      - /var/run/docker.sock:/var/run/docker.sock:ro
      # Mount the volume to store the certificates
      - {base_name}-traefik-public-certificates:/certificates
    command:
      # Enable Docker in Traefik, so that it reads labels from Docker services
      - --providers.docker
      # Do not expose all Docker services, only the ones explicitly exposed
      - --providers.docker.exposedbydefault=false
      # Create an entrypoint "http" listening on port 80
      - --entrypoints.http.address=:80
      # Create an entrypoint "https" listening on port 443
      - --entrypoints.https.address=:443
      # Create the certificate resolver "le" for Let's Encrypt, uses the environment variable EMAIL
      - --certificatesresolvers.le.acme.email=${{EMAIL?Variable not set}}
      # Store the Let's Encrypt certificates in the mounted volume
      - --certificatesresolvers.le.acme.storage=/certificates/acme.json
      # Use the TLS Challenge for Let's Encrypt
      - --certificatesresolvers.le.acme.tlschallenge=true
      # Enable the access log, with HTTP requests
      - --accesslog
      # Enable the Traefik log, for configurations and errors
      - --log
      # Enable the Dashboard and API
      - --api
    networks:
      # Use the public network created to be shared between Traefik and
      # any other service that needs to be publicly available with HTTPS
      - traefik-public

volumes:
  # Create a volume to store the certificates, even if the container is recreated
  {base_name}-traefik-public-certificates:

networks:
  # Use the previously created public network "traefik-public", shared with other
  # services that need to be publicly available via this Traefik
  traefik-public:
    name: traefik-public
    external: true
"#
    )
}

pub fn save_dockercompose_traefik_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("docker-compose.treafik.yml");
    let file_content = create_dockercompose_traefik_file(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockerfile(project_info: &ProjectInfo) -> Result<String> {
    let python_version = &project_info.python_version;
    let source_dir = &project_info.source_dir;
    match project_info.project_manager {
        ProjectManager::Uv => Ok(format!(
            r#"# syntax=docker/dockerfile:1

FROM ubuntu:24.04 AS builder

WORKDIR /app

ENV \
  PYTHONUNBUFFERED=true \
  UV_PYTHON_INSTALL_DIR=/opt/uv/python \
  UV_LINK_MODE=copy

RUN : \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  build-essential \
  curl \
  ca-certificates \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install uv
ADD https://astral.sh/uv/install.sh /uv-installer.sh

RUN sh /uv-installer.sh && rm /uv-installer.sh

ENV PATH="/root/.local/bin:$PATH"

# Create virtual environment and download Python
RUN uv venv -p {python_version}

COPY . ./

RUN --mount=type=cache,target=/root/.cache/uv \
  uv sync --locked --no-dev --no-editable


# Build production stage
FROM ubuntu:24.04 AS prod

RUN useradd appuser

WORKDIR /app

RUN chown appuser:appuser /app

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/app/.venv/bin:$PATH" \
  PORT="8000"

COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/{source_dir} /app/{source_dir}
COPY --from=builder /opt/uv/python /opt/uv/python
COPY ./scripts/entrypoint.sh /app

RUN chmod +x /app/entrypoint.sh

EXPOSE 8000

USER appuser

ENTRYPOINT ["./entrypoint.sh"]
"#,
        )),
        ProjectManager::Poetry => Ok(format!(
            r#"# syntax=docker/dockerfile:1

FROM ubuntu:24.04 AS builder

WORKDIR /app

ENV \
  PYTHONUNBUFFERED=true \
  POETRY_NO_INTERACTION=true \
  POETRY_VIRTUALENVS_IN_PROJECT=true \
  POETRY_CACHE_DIR=/tmp/poetry_cache

RUN : \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  build-essential \
  curl \
  ca-certificates \
  software-properties-common \
  && add-apt-repository ppa:deadsnakes/ppa \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  python{python_version} \
  python{python_version}-dev \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install Poetry
RUN curl -sSL https://install.python-poetry.org | python{python_version} -

ENV PATH="/root/.local/bin:$PATH"

COPY pyproject.toml poetry.lock ./

COPY . ./

RUN --mount=type=cache,target=$POETRY_CACHE_DIR \
  poetry config virtualenvs.in-project true \
  && poetry install --only=main


# Build production stage
FROM ubuntu:24.04 AS prod

RUN useradd appuser

WORKDIR /app

RUN chown appuser:appuser /app

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/app/.venv/bin:$PATH" \
  PORT="8000"

RUN : \
  && apt-get update \
  && apt-get install -y --no-install-recommends\
  software-properties-common \
  && add-apt-repository ppa:deadsnakes/ppa \
  && apt-get update \
  && apt-get install -y --no-install-recommends python{python_version} \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/{source_dir} /app/{source_dir}
COPY ./scripts/entrypoint.sh /app

RUN chmod +x /app/entrypoint.sh

EXPOSE 8000

USER appuser

ENTRYPOINT ["./entrypoint.sh"]
"#
        )),
        ProjectManager::Setuptools => Ok(format!(
            r#"# syntax=docker/dockerfile:1

FROM ubuntu:24.04 AS builder

WORKDIR /app

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/root/.local/bin:$PATH"

RUN : \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  software-properties-common \
  && add-apt-repository ppa:deadsnakes/ppa \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  python{python_version} \
  python{python_version}-venv \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python{python_version} -m venv .venv

COPY . ./

RUN .venv/bin/python -m pip install -r requirements.txt


# Build production stage
FROM ubuntu:24.04 AS prod

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/app/.venv/bin:$PATH" \
  PORT="8000"

RUN : \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  software-properties-common \
  && add-apt-repository ppa:deadsnakes/ppa \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  python{python_version} \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN useradd appuser

WORKDIR /app

RUN chown appuser:appuser /app

COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/{source_dir} /app/{source_dir}
COPY ./scripts/entrypoint.sh /app

RUN chmod +x /app/entrypoint.sh

EXPOSE 8000

USER appuser

ENTRYPOINT ["./entrypoint.sh"]
"#
        )),
        ProjectManager::Maturin => {
            if let Some(project_manager) = &project_info.pyo3_python_manager {
                match project_manager {
                    Pyo3PythonManager::Uv => Ok(format!(
                        r#"# syntax=docker/dockerfile:1

FROM ubuntu:24.04 AS builder

WORKDIR /app

ENV \
  PYTHONUNBUFFERED=true \
  UV_PYTHON_INSTALL_DIR=/opt/uv/python \
  UV_LINK_MODE=copy

RUN : \
 && apt-get update \
  && apt-get install -y --no-install-recommends \
  build-essential \
  curl \
  ca-certificates \
  libssl-dev \
  pkg-config \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install uv
ADD https://astral.sh/uv/install.sh /uv-installer.sh

RUN sh /uv-installer.sh && rm /uv-installer.sh

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.local/bin:/root/.cargo/bin:$PATH"

# Create virtual environment and download Python
RUN uv venv -p {python_version}

COPY pyproject.toml Cargo.toml Cargo.lock README.md LICENSE ./
COPY src/ ./src
RUN mkdir {source_dir}

RUN --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  uv tool run maturin develop -r

COPY uv.lock ./

RUN --mount=type=cache,target=/root/.cache/uv \
  uv sync --locked --no-dev --no-install-project

COPY . /app


# Build production stage
FROM ubuntu:24.04 AS prod

RUN useradd appuser

WORKDIR /app

RUN chown appuser:appuser /app

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/app/.venv/bin:$PATH" \
  PORT="8000"

COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/{source_dir} /app/{source_dir}
COPY --from=builder /opt/uv/python /opt/uv/python
COPY ./scripts/entrypoint.sh /app

RUN chmod +x /app/entrypoint.sh

EXPOSE 8000

USER appuser

ENTRYPOINT ["./entrypoint.sh"]
"#,
                    )),
                    Pyo3PythonManager::Setuptools => Ok(format!(
                        r#"# syntax=docker/dockerfile:1

FROM ubuntu:24.04 AS builder

WORKDIR /app

ENV \
  PYTHONUNBUFFERED=true \
  UV_PYTHON_INSTALL_DIR=/opt/uv/python \
  UV_LINK_MODE=copy

RUN : \
 && apt-get update \
  && apt-get install -y --no-install-recommends \
  build-essential \
  curl \
  ca-certificates \
  libssl-dev \
  pkg-config \
  software-properties-common \
  && add-apt-repository ppa:deadsnakes/ppa \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  python{python_version} \
  python{python_version}-dev \
  python{python_version}-venv \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install uv
ADD https://astral.sh/uv/install.sh /uv-installer.sh

RUN sh /uv-installer.sh && rm /uv-installer.sh

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.local/bin:/root/.cargo/bin:$PATH"

# Create virtual environment
RUN python{python_version} -m venv .venv

COPY pyproject.toml Cargo.toml Cargo.lock README.md LICENSE ./
COPY src/ ./src
RUN mkdir {source_dir}

RUN --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  uv tool run maturin develop -r

COPY requirements.txt ./

RUN --mount=type=cache,target=/root/.cache/uv \
  .venv/bin/python -m pip install -r requirements.txt

COPY . /app


RUN --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  .venv/bin/python -m pip install -r requirements.txt \
  && uv tool run maturin develop -r


# Build production stage
FROM ubuntu:24.04 AS prod

RUN useradd appuser

WORKDIR /app

RUN chown appuser:appuser /app

ENV \
  PYTHONUNBUFFERED=true \
  PATH="/app/.venv/bin:$PATH" \
  PORT="8000"

RUN : \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  software-properties-common \
  && add-apt-repository ppa:deadsnakes/ppa \
  && apt-get update \
  && apt-get install -y --no-install-recommends \
  python{python_version} \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/.venv /app/.venv
COPY --from=builder /app/{source_dir} /app/{source_dir}
COPY ./scripts/entrypoint.sh /app

RUN chmod +x /app/entrypoint.sh

EXPOSE 8000

USER appuser

ENTRYPOINT ["./entrypoint.sh"]
"#
                    )),
                }
            } else {
                bail!("A PyO3 python manager is required for Maturin projects")
            }
        }
        ProjectManager::Pixi => bail!("Pixi is not currently supported for FastAPI projects"),
    }
}

pub fn save_dockerfile(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("Dockerfile");
    let file_content = create_dockerfile(project_info)?;

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_dockerignore(project_info: &ProjectInfo) -> String {
    let mut info = r#"__pycache__
app.egg-info
*.pyc
.mypy_cache
.pytest_cache
.ruff_cache
.coverage
htmlcov
.cache
.venv
.env*
*.log
Dockerfile
.dockerignore
.git
tests
tests-results
"#
    .to_string();

    if project_info.project_manager == ProjectManager::Maturin {
        info.push_str("target\n");
    }

    info
}

pub fn save_dockerfileignore(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join(".dockerignore");
    let file_content = create_dockerignore(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_entrypoint_script(project_info: &ProjectInfo) -> String {
    let module = &project_info.module_name();

    format!(
        r#"#!/bin/bash

CORES=$(nproc --all)
WORKERS=$((($CORES * 2 + 1) > 8 ? 8 : ($CORES * 2 + 1)))

echo Starting Granian with $WORKERS workers

.venv/bin/granian ./{module}/main:app --host 0.0.0.0 --port 8000 --interface asgi --no-ws --workers ${{WORKERS}} --runtime-mode st --loop uvloop --log-level info --log --workers-lifetime 10800 --respawn-interval 30 --process-name granian-at-reporter
"#
    )
}

pub fn save_entrypoint_script(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.base_dir();
    let file_path = base.join("scripts/entrypoint.sh");
    let file_content = create_entrypoint_script(project_info);

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_info::{DatabaseManager, LicenseType, ProjectInfo, Pyo3PythonManager};
    use insta::assert_yaml_snapshot;
    use std::fs::create_dir_all;
    use tmp_path::tmp_path;

    #[tmp_path]
    fn project_info_dummy() -> ProjectInfo {
        ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: "my-project".to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.11".to_string(),
            min_python_version: "3.10".to_string(),
            project_manager: ProjectManager::Poetry,
            pyo3_python_manager: Some(Pyo3PythonManager::Uv),
            is_application: true,
            is_async_project: false,
            github_actions_python_test_versions: vec![
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
                "3.13".to_string(),
                "3.14".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            include_docs: false,
            docs_info: None,
            download_latest_packages: false,
            project_root_dir: Some(tmp_path),
            is_fastapi_project: true,
            database_manager: Some(DatabaseManager::AsyncPg),
        }
    }

    #[test]
    fn test_save_dockerfile_uv() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Uv;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("Dockerfile");
        save_dockerfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dockerfile_poetry() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Poetry;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("Dockerfile");
        save_dockerfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dockerfile_setuptools() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Setuptools;
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("Dockerfile");
        save_dockerfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dockerfile_maturin_uv() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.pyo3_python_manager = Some(Pyo3PythonManager::Uv);
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("Dockerfile");
        save_dockerfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }

    #[test]
    fn test_save_dockerfile_maturin_setuptools() {
        let mut project_info = project_info_dummy();
        project_info.project_manager = ProjectManager::Maturin;
        project_info.pyo3_python_manager = Some(Pyo3PythonManager::Setuptools);
        let base = project_info.base_dir();
        create_dir_all(&base).unwrap();
        let expected_file = base.join("Dockerfile");
        save_dockerfile(&project_info).unwrap();

        assert!(expected_file.is_file());

        let content = std::fs::read_to_string(expected_file).unwrap();

        assert_yaml_snapshot!(content);
    }
}
