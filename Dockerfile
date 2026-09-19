FROM scratch
ARG TARGETARCH
COPY bin/byteme-${TARGETARCH} /bin/byteme
ENTRYPOINT ["/bin/byteme"]
