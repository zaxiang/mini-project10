FROM ghcr.io/cargo-lambda/cargo-lambda:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo lambda build --release --arm64
FROM public.ecr.aws/lambda/provided:al2-arm64
WORKDIR /mini10_work
COPY --from=builder /usr/src/app/target/ ./ 
COPY --from=builder /usr/src/app/src/pythia-70m-q4_0-ggjt.bin ./ 
RUN if [ -d /mini10_work/lambda/mini10/ ]; then echo "Directory exists"; else echo "Directory does not exist"; fi
RUN if [ -f /mini10_work/lambda/mini10/bootstrap ]; then echo "File exists"; else echo "File does not exist"; fi
ENTRYPOINT ["/mini10_work/lambda/mini10/bootstrap"]
