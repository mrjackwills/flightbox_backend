#########
# SETUP #
#########

FROM alpine:3.18 AS SETUP

ARG DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group \
	DOCKER_GUID \
	DOCKER_UID \
	DOCKER_TIME_CONT \
	DOCKER_TIME_CITY

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN addgroup -g "${DOCKER_GUID}" -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
 	&& apk --no-cache add xset ca-certificates \
	&& update-ca-certificates

WORKDIR /app

# This gets automatically updated via create_release.sh
RUN wget https://github.com/mrjackwills/flightbox_backend/releases/download/v0.1.6/flightbox_backend_aarch64.tar.gz \
	&& tar xzvf flightbox_backend_aarch64.tar.gz flightbox \
	&& rm flightbox_backend_aarch64.tar.gz

##########
# RUNNER #
##########

FROM scratch AS RUNNER

ARG DOCKER_TIME_CONT \
		DOCKER_TIME_CITY \
		DOCKER_APP_USER=app_user \
		DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

COPY --from=SETUP /app/ /app
COPY --from=SETUP /etc/group /etc/passwd /etc/
COPY --from=SETUP /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER ${DOCKER_APP_USER}

ENTRYPOINT ["/app/flightbox"]