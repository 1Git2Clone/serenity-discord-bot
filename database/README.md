# serenity-discord-bot/database

This directory is only used for your database migrations from sqlx.
They work with the sqlx cli utility which can be installed using:
```sh
cargo install sqlx-cli
```

After the installation you can run the migration from
`serenity-discord-bot/migrations` by using the following command:

```sh
sqlx migrate run
```

For more info here's their [GitHub repo](https://github.com/launchbadge/sqlx)
