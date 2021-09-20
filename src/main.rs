use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::prelude::Ready;

use std::env;

#[group]
#[commands(ping, test)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and listening.", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
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
    }
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

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    message.edit(&ctx, |m| m.content("Message edited!"));

    Ok(())
}
