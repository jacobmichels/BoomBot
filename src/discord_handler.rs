use std::{collections::HashMap, str::from_utf8};

use anyhow::{Ok, Result};
use reqwest::Response;
use serenity::{
    async_trait,
    builder::CreateApplicationCommands,
    model::{
        channel::ReactionType,
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::ApplicationCommandInteraction, Interaction,
            InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
        },
    },
    prelude::*,
    utils::MessageBuilder,
};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("credentials wrong")]
    AuthFailed,
    #[error("mfa token needed")]
    MFANeeded,
    #[error("http error")]
    HttpError(String),
}

// Unit struct to act as our EventHandler for discord events
pub struct DiscordHandler;

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, _ctx: Context, _data: Ready) {
        log::info!("Bot ready");
    }

    // This runs when the cache object is ready and populated with Guild data
    async fn cache_ready(&self, ctx: Context, guild_ids: Vec<GuildId>) {
        log::info!("Cache ready");

        // Here we iterate through the guilds and create slash commands in them
        for guild_id in guild_ids {
            let name = guild_id.name(ctx.cache.clone()).await.unwrap();

            log::info!("Adding slash commands to guild: {}", name);
            let commands =
                GuildId::set_application_commands(&guild_id, &ctx.http, create_slash_commands)
                    .await;

            log::info!(
                "Successfully added {} commands to {}",
                commands.unwrap().len(),
                name
            );
        }
    }

    // In this method we react to our bot's slash commands
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // If the interaction is an ApplicationCommand, we run the appropriate handler function
            let response_result = run_command_handler(&ctx, &command).await;

            if let Err(why) = response_result {
                log::error!(
                    "An error occured while sending interaction response: {}",
                    why
                )
            }
        }
    }
}

fn create_slash_commands(
    commands: &mut CreateApplicationCommands,
) -> &mut CreateApplicationCommands {
    commands.create_application_command(|command| {
        command.name("register").description(
            "Register yourself with BoomBot to be able to receieve Valorant shop notifications",
        )
    })
}

// This function runs the appropriate handler for each command
async fn run_command_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> anyhow::Result<()> {
    return match command.data.name.as_str() {
        "register" => register_command_handler(ctx, command).await,
        _ => {
            send_ephemeral_reply(ctx, command, "not implemented").await?;
            Ok(())
        }
    };
}

// Helper function to send an ephemeral reply with the supplied message
async fn send_ephemeral_reply(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    reply_message: &str,
) -> Result<(), serenity::Error> {
    command
        .create_interaction_response(ctx.http.clone(), |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .content(reply_message)
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
        })
        .await
}

async fn register_command_handler(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> anyhow::Result<()> {
    // Send reply to channel directing user to DMs
    send_ephemeral_reply(ctx, command, "Check your DMs to get started with BoomBot!").await?;

    // Send DM with credentials warning
    let message = command
        .user
        .direct_message(&ctx, |message| {
            let content = MessageBuilder::new().push("Hello ").mention(&command.user).push("! 👋\n\nTo allow BoomBot access to your Valorant data, you'll need to enter your Riot account credentials to fetch your Valorant data.\n\nReact with 👍 if you're ready to continue, or 🚫 to cancel.").build();
            message.content(content)
        })
        .await?;

    // Get user reaction to continue
    message
        .react(&ctx, ReactionType::Unicode("👍".to_string()))
        .await?;
    message
        .react(&ctx, ReactionType::Unicode("🚫".to_string()))
        .await?;

    let reaction_result = message
        .await_reaction(&ctx)
        .filter(|reaction| {
            if let "👍" = reaction.emoji.as_data().as_str() {
                return true;
            }
            if let "🚫" = reaction.emoji.as_data().as_str() {
                return true;
            }

            false
        })
        .await;

    if reaction_result.is_none() {
        command
            .user
            .direct_message(&ctx, |message| message.content("invalid reaction"))
            .await?;
        return Ok(());
    }

    // If the reaction is negative, cancel the flow
    if reaction_result
        .unwrap()
        .as_inner_ref()
        .emoji
        .as_data()
        .as_str()
        == "🚫"
    {
        command
            .user
            .direct_message(&ctx, |message| {
                message.content(
                    "No problem, feel free to call /register again if you change your mind.",
                )
            })
            .await?;

        return Ok(());
    }

    // Start the positive flow by getting a username from the user
    command
        .user
        .direct_message(&ctx, |message| message.content("Great! Let's get started."))
        .await?;

    let dm_channel_id = command.user.create_dm_channel(&ctx).await?.id;

    loop {
        command
            .user
            .direct_message(&ctx, |message| {
                message.content("Please enter your Riot account username.")
            })
            .await?;

        let username_reply = command
            .user
            .await_reply(&ctx)
            .channel_id(dm_channel_id)
            .filter(|reply_message| {
                if reply_message.content.is_empty() {
                    return false;
                }
                true
            })
            .await;

        if username_reply.is_none() {
            command
                .user
                .direct_message(&ctx, |message| {
                    message.content("Invalid username, please re-run /register.")
                })
                .await?;
            return Ok(());
        }

        let username = &username_reply.unwrap().content;

        command
            .user
            .direct_message(&ctx, |message| {
                message.content("Please enter your Riot account password.")
            })
            .await?;

        let password_reply = command
            .user
            .await_reply(&ctx)
            .channel_id(dm_channel_id)
            .filter(|reply_message| {
                if reply_message.content.is_empty() {
                    return false;
                }
                true
            })
            .await;

        if password_reply.is_none() {
            command
                .user
                .direct_message(&ctx, |message| {
                    message.content("Invalid password, please re-run /register")
                })
                .await?;
            return Ok(());
        }

        let password = &password_reply.unwrap().content;

        command
            .user
            .direct_message(&ctx, |message| {
                message.content("Verifying your credentials...")
            })
            .await?;

        let credentials_result = credentials_valid(username, password, None).await;
        match credentials_result {
            std::result::Result::Ok(()) => {
                command
                    .user
                    .direct_message(&ctx, |message| {
                        message.content("Credentials valid! You're all set up.")
                    })
                    .await?;
                command
                .user
                .direct_message(&ctx, |message| {
                    message.content("To get a list of commands to use, type /help in the channel you registered from.")
                })
                .await?;
                return Ok(());
            }
            Err(err) => match err {
                AuthError::AuthFailed => {
                    command
                        .user
                        .direct_message(&ctx, |message| {
                            message.content("Credentials invalid, try again.")
                        })
                        .await?;
                }
                AuthError::MFANeeded => {
                    command
                        .user
                        .direct_message(&ctx, |message| {
                            message.content("Multi-factor authentication is enabled, please enter your auth code from Riot. Check your email.")
                        })
                        .await?;

                    let mfa_reply = command
                        .user
                        .await_reply(&ctx)
                        .channel_id(dm_channel_id)
                        .filter(|reply_message| {
                            if reply_message.content.is_empty() || reply_message.content.len() != 6
                            {
                                return false;
                            }

                            true
                        })
                        .await;

                    if mfa_reply.is_none() {
                        command
                            .user
                            .direct_message(&ctx, |message| {
                                message.content("Invalid MFA token, please re-run /register")
                            })
                            .await?;
                        return Ok(());
                    }
                    if let std::result::Result::Ok(()) =
                        credentials_valid(username, password, Some(&mfa_reply.unwrap().content))
                            .await
                    {
                        command
                            .user
                            .direct_message(&ctx, |message| {
                                message.content("Credentials valid! You're all set up.")
                            })
                            .await?;
                        command
                            .user
                            .direct_message(&ctx, |message| {
                                message.content("To get a list of commands to use, type /help in the channel you registered from.")
                            })
                            .await?;
                        return Ok(());
                    } else {
                        command
                            .user
                            .direct_message(&ctx, |message| {
                                message.content("Credentials invalid, try again.")
                            })
                            .await?;
                    }
                }
                AuthError::HttpError(message) => return Err(AuthError::HttpError(message).into()),
            },
        }
    }
}

async fn credentials_valid(
    username: &str,
    password: &str,
    mfa_token: Option<&str>,
) -> Result<(), AuthError> {
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();

    let mut data = HashMap::new();
    data.insert("client_id", "play-valorant-web-prod");
    data.insert("nonce", "1");
    data.insert("redirect_uri", "https://playvalorant.com/opt_in");
    data.insert("response_type", "token id_token");

    let response = client
        .post("https://auth.riotgames.com/api/v1/authorization")
        .json(&data)
        .header("User-Agent", "Chrome/99.0.4844.51 Safari/537.36")
        .send()
        .await;

    if let Err(err) = response {
        return Err(AuthError::HttpError(err.to_string()));
    }

    log::info!("request 1 status: {}", response.as_ref().unwrap().status());
    log::info!("text: {}", response.unwrap().text().await.unwrap());

    data.clear();
    data.insert("type", "auth");
    data.insert("username", username);
    data.insert("password", password);
    data.insert("remember", "true");

    let auth_response = client
        .put("https://auth.riotgames.com/api/v1/authorization")
        .header("User-Agent", "Chrome/99.0.4844.51 Safari/537.36")
        .json(&data)
        .send()
        .await;

    if let Err(err) = auth_response {
        return Err(AuthError::HttpError(err.to_string()));
    }

    log::info!("{}", auth_response.unwrap().text().await.unwrap());

    std::result::Result::Ok(())
}
