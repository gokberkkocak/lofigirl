FROM debian:bookworm AS builder

WORKDIR /app

COPY ./ ./

RUN apt update
RUN apt upgrade -y
RUN apt install -y build-essential libopencv-dev curl libssl-dev libleptonica-dev clang libclang-dev libtesseract-dev pkg-config sqlite3

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV DATABASE_URL=sqlite:token.db
RUN sqlite3 token.db < migrations/20210525000135_table.sql 

RUN mkdir -p /app/bin
RUN cargo build --release -p lofigirl_client --features standalone
RUN mv ./target/release/lofigirl_client /app/bin/lofigirl_client_standalone
RUN cargo build --release -p lofigirl_client -p lofigirl_server
RUN mv ./target/release/lofigirl_client /app/bin/
RUN mv ./target/release/lofigirl_server /app/bin/

FROM debian:bookworm as runner

COPY --from=builder /app/bin/lofigirl_client /usr/bin/
COPY --from=builder /app/bin/lofigirl_server /usr/bin/
COPY --from=builder /app/bin/lofigirl_client_standalone /usr/bin/

RUN apt update
RUN apt upgrade -y
RUN apt install -y libopencv-dev libleptonica-dev libtesseract-dev tesseract-ocr-eng python3 python3-pip

RUN pip3 install --break-system-packages yt-dlp

ENTRYPOINT [ "lofigirl_server" ]