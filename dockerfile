FROM rustlang/rust:nightly as builder

EXPOSE 8000

WORKDIR /contactive
COPY . .
RUN cargo install --path .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

CMD [ "contactive" ]