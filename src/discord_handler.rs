use anyhow::Result;
use serenity::{
    async_trait,
    builder::CreateApplicationCommands,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::ApplicationCommandInteraction, Interaction,
            InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
        },
    },
    prelude::*,
};

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
    send_ephemeral_reply(ctx, command, "Check your DMs to get started with BoomBot!").await?;
    Ok(())
}
