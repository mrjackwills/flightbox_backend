#########
# SETUP #
#########
FROM alpine:3.22 AS setup

ARG DOCKER_APP_USER=app_user
ARG DOCKER_APP_GROUP=app_group
ARG DOCKER_GUID=1000
ARG DOCKER_UID=1000

# This gets automatically updated via create_release.sh
ARG CURRENT_VERSION=v0.2.4

RUN addgroup -g "${DOCKER_GUID}" -S "${DOCKER_APP_GROUP}" \
	&& adduser -u "${DOCKER_UID}" -S -G "${DOCKER_APP_GROUP}" "${DOCKER_APP_USER}" \
	&& apk add --no-cache xset ca-certificates wget tar\
	&& update-ca-certificates

WORKDIR /app

# Somewhat convoluted way to automatically select & download the correct package, absed on system arch
RUN ARCH=$(uname -m) && \
	case "$ARCH" in \
		x86_64) SUFFIX=x86_64 ;; \
		aarch64) SUFFIX=aarch64 ;; \
		*) exit 1 ;; \
	esac \ 
	&& wget "https://github.com/mrjackwills/flightbox_backend/releases/download/v${CURRENT_VERSION}/flightbox_linux_${SUFFIX}.tar.gz" \
	&& tar xzvf flightbox_linux_${SUFFIX}.tar.gz flightbox \
	&& rm "flightbox_linux_${SUFFIX}.tar.gz" \
	&& chown "${DOCKER_APP_USER}:${DOCKER_APP_GROUP}" /app/flightbox

##########
# RUNNER #
##########

FROM scratch

ARG DOCKER_APP_USER=app_user
ARG DOCKER_APP_GROUP=app_group

COPY --from=setup /app/ /app
COPY --from=setup /etc/passwd /etc/group /etc/
COPY --from=setup /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER "${DOCKER_APP_USER}""

ENTRYPOINT ["/app/flightbox"]