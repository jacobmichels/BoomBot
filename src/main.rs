use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::prelude::Ready;
use std::str::FromStr;
use std::sync::Arc;

use serenity::prelude::{RwLock, TypeMapKey};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{query_as, ConnectOptions, Pool, Sqlite, SqliteConnection};

use std::env;

#[group]
#[commands(myshop)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and listening.", ready.user.name);
    }
}

#[derive(sqlx::FromRow)]
struct ValorantUser {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), impl std::error::Error> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("/")) // set the bot's prefix to "/"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut client =
        Client::builder(env::var("DISCORD_TOKEN").expect("Discord token not available."))
            .event_handler(Handler)
            .framework(framework)
            .await
            .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
        return Err(why);
    }

    Ok(())
}

#[command]
async fn myshop(ctx: &Context, msg: &Message) -> CommandResult {
    let author_id = msg.author.id.to_string();

    println!("{}", author_id);

    let mut conn = SqliteConnectOptions::from_str("sqlite://app.db")
        .unwrap()
        .create_if_missing(true)
        .connect()
        .await
        .unwrap();

    //check if author_id is in the database.

    let user: Option<ValorantUser> = query_as::<_, ValorantUser>(
        "select username, password from valorant_accounts where discord_id = ?",
    )
    .bind(author_id)
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    match user {
        Some(user) => {
            msg.reply(
                ctx,
                format!("Fetching shop for valorant user {}...", user.username),
            )
            .await?;
        }
        None => {
            msg.reply(
                ctx,
                "To view your shop, we need to get set up first. Check your DMs for instructions!",
            )
            .await?;

            msg.author.create_dm_channel(&ctx).await?
            .say(&ctx, format!("Hello {}! To access your Valorant shop, I need access to your Riot account. Please respond with the following: one message containing only your riot username, one message containing only your riot password.", msg.author.name))
            .await?;
        }
    }

    Ok(())
}
