#!/bin/bash
###############################################################################
# This script is nothing more than a template for running the docker
# instructions from the README.md file. However it only works on Linux using
# docker=cli.
#
# This is the deprecated docker builder. Make sure to check out the
# docker-compose.yml set up.
###############################################################################
docker volume create --name database
docker build -t serenity-discord-bot .
docker run -v ./database:/app/database -d --env-file .env serenity-discord-bot
