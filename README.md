# Serenity Discord Bot

### Setting up

In order to set up your bot token you can do it manually by creating
and modifying the .env file and adding the token in it like this:

```env
BOT_TOKEN=YOUR_DISCORD_BOT_TOKEN_HERE
```

Alternatively, there's also a script that does the exact same thing!
To compile and run it, enter in these commands (in the **ROOT** directory of this repository!).

For Linux:

```sh
mkdir bin/

mkdir database/

rustc extra_utils/generate_dotenv.rs -o bin/generate_dotenv

./bin/generate_dotenv
```

For Windows:

```sh
mkdir bin\

mkdir database\

rustc extra_utils\generate_dotenv.rs -o bin\generate_dotenv

.\bin\generate_dotenv
```

After you've configured your dotenv (.env) files, you can just run

```
cargo install sqlx-cli

sqlx database setup

cargo sqlx prepare

cargo run --release
```

and you have your bot ready to go!

### Additional info

The database setting up is a ONE TIME ONLY thing. From your second run after you just need to do:

```
cargo run --release
```

Also, If you want to change your database location, make sure you also change the .env file.

```env
# This line determines where the database is based on the root directory of the repository.
DATABASE_URL=sqlite:database/botdatabase.sqlite
```
