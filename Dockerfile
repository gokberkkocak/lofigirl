FROM archlinux AS builder

WORKDIR /app

COPY ./ ./

ENV DATABASE_URL=sqlite:token.db

RUN pacman-key --init

RUN pacman --noconfirm -Syu

RUN pacman --noconfirm -S openssl pkgconf opencv vtk hdf5 qt5-base glew tesseract clang rustup sqlite

RUN rustup toolchain install nightly

RUN sqlite3 token.db < migrations/20210525000135_table.sql 

RUN cargo build --release -p lofigirl_client --features standalone

RUN mkdir -p /app/bin

RUN mv ./target/release/lofigirl_client /app/bin/lofigirl_client_standalone

RUN cargo build --release -p lofigirl_client -p lofigirl_server

RUN mv ./target/release/lofigirl_client /app/bin/

RUN mv ./target/release/lofigirl_server /app/bin/

FROM archlinux as runner

COPY --from=builder /app/bin/lofigirl_client /usr/bin/

COPY --from=builder /app/bin/lofigirl_server /usr/bin/

COPY --from=builder /app/bin/lofigirl_client_standalone /usr/bin/

RUN pacman-key --init

RUN pacman --noconfirm -Syu

RUN pacman --noconfirm -S opencv vtk hdf5 qt5-base glew tesseract tesseract-data-eng 

ENTRYPOINT [ "lofigirl_server" ]