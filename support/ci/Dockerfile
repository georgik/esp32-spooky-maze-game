# Helper Dockerfile to workarond the issue with the GitHub Actions not able to work under esp user

FROM espressif/idf-rust:all_1.68.0.0
USER root
RUN mkdir /__w \
    && chown esp:esp /__w
USER esp
