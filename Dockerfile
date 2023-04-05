FROM gcr.io/distroless/static:nonroot

WORKDIR /hello-container

# Copy our build
COPY target/x86_64-unknown-linux-musl/release/hello-container ./
# EXPOSE 8080
CMD ["/hello-container/hello-container"]
