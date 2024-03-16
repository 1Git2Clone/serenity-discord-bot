#!/bin/bash
###############################################################################
# This script is nothing more than a template for running the docker
# instructions from the README.md file. However it only works on Linux using
# docker=cli.
###############################################################################
docker volume create --name database
docker build -t serenity-discord-bot .
docker run -v database:/database -d --env-file .env serenity-discord-bot
