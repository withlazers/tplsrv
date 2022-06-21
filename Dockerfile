FROM clux/muslrust:1.60.0 AS build

WORKDIR /src

COPY . .

RUN cargo install \
		--path . \
		--root /target

WORKDIR /app

RUN cp /target/bin/tplsrv .


FROM gcr.io/distroless/static:nonroot

COPY --chown=nonroot:nonroot --from=build \
	/app /app

EXPOSE 8080

ENV TPLSRV_DIR=/tmpl

ENTRYPOINT ["/app/tplsrv"]
