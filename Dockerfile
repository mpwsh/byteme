FROM scratch
ARG TARGETARCH
COPY bin/byteme-${TARGETARCH} /bin/byteme
CMD ["/bin/byteme"]
