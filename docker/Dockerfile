# ############
# # App only #
# ############

FROM alpine:3.17

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add tzdata xset \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone 

WORKDIR /app

# This gets automatically updated via create_release.sh
RUN wget https://github.com/mrjackwills/flightbox_backend/releases/download/v0.0.1/flightbox_backend_aarch64.tar.gz \
	&& tar xzvf flightbox_backend_aarch64.tar.gz flightbox \
	&& rm flightbox_backend_aarch64.tar.gz \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app/

USER ${DOCKER_APP_USER}

CMD [ "./flightbox"]