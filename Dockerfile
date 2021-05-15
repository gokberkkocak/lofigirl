FROM archlinux AS builder

WORKDIR /app

COPY ./ ./

RUN pacman-key --init

RUN pacman --noconfirm -Syu

RUN pacman --noconfirm -S openssl pkgconf opencv vtk hdf5 qt5-base glew tesseract clang rustup

RUN rustup toolchain install nightly

RUN cargo build --release --features standalone

RUN mv ./target/release/lofigirl ./target/release/lofigirl_standalone

RUN  cargo build --release

FROM archlinux as runner

COPY --from=builder /app/target/release/lofigirl /usr/bin/

COPY --from=builder /app/target/release/lofigirl_server /usr/bin/

COPY --from=builder /app/target/release/lofigirl_standalone /usr/bin/

RUN pacman-key --init

RUN pacman --noconfirm -Syu

RUN pacman --noconfirm -S opencv vtk hdf5 qt5-base glew tesseract tesseract-data-eng 

ENTRYPOINT [ "lofigirl_server" ]