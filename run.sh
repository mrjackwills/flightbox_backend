#!/bin/bash

# v0.1.0

# CHANGE
MONO_NAME='flightbox'

RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
RESET='\033[0m'

DOCKER_GUID=$(id -g)
DOCKER_UID=$(id -u)
DOCKER_TIME_CONT="America"
DOCKER_TIME_CITY="New_York"

PRO=production
DEV=dev

error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n";
	exit 1
}

# $1 any variable name
# $2 variable name
check_variable() {
	if [ -z "$1" ]
	then
		error_close "Missing variable $2"
	fi
}

check_variable "$MONO_NAME" "\$MONO_NAME"

if ! [ -x "$(command -v dialog)" ]; then
	error_close "dialog is not installed"
fi

set_base_dir() {
	local workspace="/workspaces/pi_client"
	local server="$HOME/${MONO_NAME}"
	if [[ -d "$workspace" ]]
	then
		BASE_DIR="${workspace}"
	else 
		BASE_DIR="${server}"
	fi
}

set_base_dir


# $1 string - question to ask
ask_yn () {
	printf "%b%s? [y/N]:%b " "${GREEN}" "$1" "${RESET}"
}

# return user input
user_input() {
	read -r data
	echo "$data"
}

# Containers
API="${MONO_NAME}"

dev_up () {
	cd "${BASE_DIR}/docker" || error_close "${BASE_DIR} doesn't exist"
	echo "starting containers: ${API}"
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	docker compose -f dev.docker-compose.yml up --force-recreate --build -d "${API}"
}

dev_down () {
	cd "${BASE_DIR}/docker" || error_close "${BASE_DIR} doesn't exist"
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	docker compose -f dev.docker-compose.yml down
}

production_up () {
	cd "${BASE_DIR}/docker" || error_close "${BASE_DIR} doesn't exist"
DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	DOCKER_BUILDKIT=0 \
	docker compose -f docker-compose.yml up -d
}

production_rebuild () {
	cd "${BASE_DIR}/docker" || error_close "${BASE_DIR} doesn't exist"
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	DOCKER_BUILDKIT=0 \
	docker compose -f docker-compose.yml up -d --build
}

production_down () {
	cd "${BASE_DIR}/docker" || error_close "${BASE_DIR} doesn't exist"
	DOCKER_GUID=${DOCKER_GUID} \
	DOCKER_UID=${DOCKER_UID} \
	DOCKER_TIME_CONT=${DOCKER_TIME_CONT} \
	DOCKER_TIME_CITY=${DOCKER_TIME_CITY} \
	docker compose -f docker-compose.yml down
}

git_pull_branch() {
	git checkout -- .
	git checkout main
	git pull origin main
	git fetch --tags
	latest_tag=$(git tag | sort -V | tail -n 1)
	git checkout -b "$latest_tag"
}

pull_branch() {
	GIT_CLEAN=$(git status --porcelain)
	if [ -n "$GIT_CLEAN" ]; then
		echo -e "\n${RED}GIT NOT CLEAN${RESET}\n"
		printf "%s\n" "${GIT_CLEAN}"
	fi
	if [[ -n "$GIT_CLEAN" ]]; then
		ask_yn "Happy to clear git state"
		if [[ "$(user_input)" =~ ^n$ ]]; then
			exit
		fi
	fi
	git_pull_branch
	main
}

main() {
	echo "in main"
	cmd=(dialog --backtitle "Start ${MONO_NAME} containers" --radiolist "choose environment" 14 80 16)
	options=(
		1 "${DEV} up" off
		2 "${DEV} down" off
		3 "${PRO} up" off
		4 "${PRO} down" off
		5 "${PRO} rebuild" off
		6 "pull & branch" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices
	do
		case $choice in
			0)
				exit;;
			1)
				dev_up
				break;;
			2)
				dev_down
				break;;
			3)
				echo "production up: ${API}"
				production_up
				break;;
			4)
				production_down
				break;;
			5)
				production_rebuild
				break;;
			6)
				pull_branch
				break
			;;
		esac
	done
}

main