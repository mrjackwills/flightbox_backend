#!/bin/bash

# run.sh v0.2.1
# 2024-10-19

APP_NAME='flightbox'

RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
RESET='\033[0m'

PRO=production
DEV=dev

error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n"
	exit 1
}

# $1 string - question to ask
# Ask a yes no question, only accepts `y` or `n` as a valid answer, returns 0 for yes, 1 for no
ask_yn() {
	while true; do
		printf "\n%b%s? [y/N]:%b " "${GREEN}" "$1" "${RESET}"
		read -r answer
		if [[ "$answer" == "y" ]]; then
			return 0
		elif [[ "$answer" == "n" ]]; then
			return 1
		else
			echo -e "${RED}\nPlease enter 'y' or 'n'${RESET}"
		fi
	done
}

if ! [ -x "$(command -v dialog)" ]; then
	error_close "dialog is not installed"
fi

# $1 any variable name
# $2 variable name
check_variable() {
	if [ -z "$1" ]; then
		error_close "Missing variable $2"
	fi
}

check_variable "$APP_NAME" "\$APP_NAME"

# Get the directory of the script
APP_DIR=$(dirname "$(readlink -f "$0")")

DOCKER_DIR="${APP_DIR}/docker"

dev_up() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	echo "starting containers: ${API}"
	docker compose -f dev.docker-compose.yml up --force-recreate --build -d "${APP_NAME}"
}

dev_down() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose -f dev.docker-compose.yml down
}

production_up() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose -f docker-compose.yml up -d
}

production_rebuild() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose -f docker-compose.yml up -d --build
}

production_down() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose -f docker-compose.yml down
}

git_pull_branch() {
	git checkout -- .
	git checkout main
	git pull origin main
	git fetch --tags
	latest_tag=$(git tag | sort -V | tail -n 1)
	git checkout -b "$latest_tag"
	sleep 10
}

pull_branch() {
	GIT_CLEAN=$(git status --porcelain)
	if [ -n "$GIT_CLEAN" ]; then
		echo -e "\n${RED}GIT NOT CLEAN${RESET}\n"
		printf "%s\n" "${GIT_CLEAN}"
	fi
	if [[ -n "$GIT_CLEAN" ]]; then
		if ! ask_yn "Happy to clear git state"; then
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
	for choice in $choices; do
		case $choice in
		0)
			exit
			;;
		1)
			dev_up
			break
			;;
		2)
			dev_down
			break
			;;
		3)
			echo "production up: ${APP_NAME}"
			production_up
			break
			;;
		4)
			production_down
			break
			;;
		5)
			production_rebuild
			break
			;;
		6)
			pull_branch
			break
			;;
		esac
	done
}

main
