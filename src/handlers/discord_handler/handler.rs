use serenity::{
    async_trait,
    builder::CreateApplicationCommands,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{ApplicationCommandInteraction, ApplicationCommandOptionType},
            Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
        },
    },
    prelude::*,
};

use crate::handlers::command_handlers::test_command_handler::test_command_handler;

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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // In this method we react to our bot's slash commands
        if let Interaction::ApplicationCommand(command) = interaction {
            // If the interaction is an ApplicationCommand, we run the appropriate handler function
            let reply_message = get_command_output(&command);

            let reply_send_result = send_ephemeral_reply(&ctx, &command, &reply_message).await;

            if let Err(why) = reply_send_result {
                log::error!(
                    "An error occured while sending interaction response: {}",
                    why
                )
            }
        }
    }
}

// Function to create all the slash commands for the application
fn create_slash_commands(
    commands: &mut CreateApplicationCommands,
) -> &mut CreateApplicationCommands {
    commands.create_application_command(|command| {
        command
            .name("test")
            .description("a test command")
            .create_option(|option| {
                option
                    .name("my_option")
                    .description("a test option")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
                    .add_string_choice("name", "value")
            })
    })
}

// This function runs the appropriate handler for each command
fn get_command_output(command: &ApplicationCommandInteraction) -> String {
    return match command.data.name.as_str() {
        "test" => test_command_handler(),
        _ => String::from("not implemented"),
    };
}

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
