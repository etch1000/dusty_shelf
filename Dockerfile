# syntax=docker/docker:3
FROM archlinux
WORKDIR /app
COPY ./dusty_shelf /app/dusty_shelf
COPY Rocket.toml ./Rocket.toml
RUN pacman -Sy libpqxx --noconfirm
EXPOSE 8080
CMD ["/app/dusty_shelf"]
