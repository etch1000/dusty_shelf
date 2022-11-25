# syntax=docker/docker:3
FROM archlinux
WORKDIR /home/rootkill/rust_all/dusty_shelf/
COPY target/debug/dusty_shelf /bin/dusty_shelf
COPY Rocket.toml ./Rocket.toml
RUN pacman -Sy libpqxx --noconfirm
EXPOSE 8000
CMD ["/bin/dusty_shelf"]
