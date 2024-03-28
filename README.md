# Serenity Discord Bot

## Features overview

This is a list of some of the Bots available features. For a more comprehensive
list you can always check out the code with all the commands or use the `help`
command.

- A help command containing all the bot commands.
- A bunch of embed interaction commands (like pats, hugs and etc.)
- A levelling system (managed locally). Some people like it, others hate it and
  say its a MEE6 clone (They kinda have a point there). Nevertheless, you can
  always use the SQLite database for something else if you need to. There's also
  a cooldown implementation and periodic cleaning up of the users which aren't
  under cooldown (This was a bit tricky to do because of the Mutex locking and my
  overall skill issue).
- The levelling system has a nice topranks command which gives a cool-looking embed!
- A bot uptime command.

## Setting up

**ALL THE RELATIVE PATHS IN THIS DESCRIPTION BASED ON THE ROOT OF THE REPOSITORY!**

In order to set up your bot token you can do it manually by creating and
modifying the .env file and adding the token in it like this:

```env
BOT_TOKEN=YOUR_DISCORD_BOT_TOKEN_HERE
```

Alternatively, there's also a script that does the exact same thing! To compile
and run it, enter in the following commands:

For Linux;

```sh
rustc extra_utils/generate_dotenv.rs -o bin/generate_dotenv

./bin/generate_dotenv
```

For Windows (Just change `/` to `\`);

```sh
rustc extra_utils\generate_dotenv.rs -o bin\generate_dotenv

.\bin\generate_dotenv
```

After you've configured your dotenv (.env) files, you can just run

```
cargo install sqlx-cli

sqlx database setup

sqlx migrate run

cargo sqlx prepare

cargo run --release
```

The database setting up is a ONE TIME ONLY thing. From your second run after
you just need to do:

```sh
cargo run --release
```

### Advanced setting up (Containerization)

I've also provided a Dockerfile in this repo for anyone who wants to build a
docker image and run the project like that.

Prerequisites:

- Docker engine [docker.com](https://docs.docker.com/engine/install/)

After you have the Docker engine you need to run these commands in your
terminal (Linux):

```sh
docker volume create --name database
docker build -t YOUR_DOCKER_IMAGE_NAME .
docker run -v database:/database -d --env-file .env YOUR_DOCKER_IMAGE_NAME
```

NOTE: The image building could take some time (this is compiling Rust code
after all) so be patient!

Checking your docker images is as simple as:

```sh
docker images
```

There's also a `docker-compose.yml` configuration if you wish to build the image by using Docker Compose.

```sh
docker-compose up -d
```

You need to install Docker Compose from [docker.com/compose/install](https://docs.docker.com/compose/install/) though.

For Windows installs removing the `sudo` prefix and using the PowerShell as an
administrator should work (according to Google Gemini). If you encounter
problems with setting it up on Windows or want to tell me more details on
setting it up on there then feel free to chat to me on Discord - `1kill2steal`

## Additional info

If you want to change your database location, make sure you also change the
.env file.

```env
# This line determines where the database is based on the root directory of the repository.
DATABASE_URL=sqlite:database/botdatabase.sqlite
```

NOTE: The database URL configuration is hard-coded in the
`extra_utils/generate_dotenv.rs` file.
