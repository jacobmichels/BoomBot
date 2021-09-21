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
#[commands(ping, test, myshop)]
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
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    let mut message = msg
        .reply(ctx, "This message will be edited in 5 seconds!")
        .await?;

    let author = &msg.author;
    let channel = author.create_dm_channel(ctx).await?;
    channel.say(ctx, "Hey there.").await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    message.edit(&ctx, |m| m.content("Message edited!")).await?;

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

    let user = query_as::<_, ValorantUser>(
        "select username, password from valorant_accounts where discord_id = ?",
    )
    .fetch_one(&mut conn)
    .await;

    match user {
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                //if not, dm them asking for their account info. then add them to the db
                msg.reply(ctx, "User not found").await?;
            }
            _ => {
                msg.reply(ctx, format!("Database error {:?}", err)).await?;
            }
        },
        //if so, then fetch their information from the db and call the my store api.
        Ok(user) => {
            msg.reply(ctx, format!("Found user {}!", user.username))
                .await?;
        }
    }

    Ok(())
}
