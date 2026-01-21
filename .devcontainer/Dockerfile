FROM debian:stable

# Install basic dev tools and setup locale
RUN apt-get update \
    && apt-get install -y sudo curl ca-certificates locales lsb-release redis-tools \
        build-essential git \
    && echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen \
    && locale-gen \
    && rm -rf /var/lib/apt/lists/*

# Install nodejs
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install pnpm
RUN npm install -g pnpm

# Install postgresql-client
RUN install -d /usr/share/postgresql-common/pgdg \
    && curl -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc --fail https://www.postgresql.org/media/keys/ACCC4CF8.asc \
    && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
    && apt-get update \
    && apt-get install -y postgresql-client-17 \
    && rm -rf /var/lib/apt/lists/*

# Create the dev user and have him sudo without password
ENV USER=dev
RUN useradd -mG sudo -s /bin/bash $USER \
    && echo "$USER ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/$USER

# Pre-create a workspace directory with the correct permissions
RUN mkdir /workspace && chown $USER:$USER /workspace

# Switch to default user
USER $USER

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y

# Install rustup components and cargo tools
ENV PATH="/home/$USER/.cargo/bin:${PATH}"
RUN rustup component add llvm-tools \
    && cargo install cargo-binstall \
    && cargo binstall -y cargo-llvm-cov cargo-nextest

# Remove sudo note on login
RUN touch /home/$USER/.sudo_as_admin_successful

# Run build-time user setup script if there's any
COPY --chown=$USER:$USER .devcontainer /tmp/.devcontainer
RUN [ ! -f /tmp/.devcontainer/user/build.sh ] \
    || bash /tmp/.devcontainer/user/build.sh

# Switch back to root, vscode devcontainers will log in as dev
USER root

# Command to stay running indefinitely
CMD sleep infinity
