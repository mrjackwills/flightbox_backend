#########
# SETUP #
#########
FROM alpine:3.19 AS setup

ARG DOCKER_APP_USER=app_user \
DOCKER_APP_GROUP=app_group \
DOCKER_GUID=1000 \
DOCKER_UID=1000

RUN addgroup -g "${DOCKER_GUID}" -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add xset ca-certificates \
	&& update-ca-certificates

WORKDIR /app

# This gets automatically updated via create_release.sh
ARG FLIGHTBOX_VERSION=v0.1.17

RUN wget "https://github.com/mrjackwills/flightbox_backend/releases/download/${FLIGHTBOX_VERSION}/flightbox_linux_aarch64.tar.gz" \
	&& tar xzvf flightbox_linux_aarch64.tar.gz flightbox \
	&& rm flightbox_linux_aarch64.tar.gz

##########
# RUNNER #
##########

FROM scratch

ARG DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

COPY --from=setup /app/ /app
COPY --from=setup /etc/group /etc/passwd /etc/
COPY --from=setup /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER ${DOCKER_APP_USER}

ENTRYPOINT ["/app/flightbox"]