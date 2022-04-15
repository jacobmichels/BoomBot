use serenity::{
    builder::CreateApplicationCommands,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
    },
};

use super::command_handlers::register_command_handler;

// Function to create all the slash commands for the application
pub fn create_slash_commands(
    commands: &mut CreateApplicationCommands,
) -> &mut CreateApplicationCommands {
    commands.create_application_command(|command| {
        command.name("register").description(
            "Register yourself with BoomBot to be able to receieve Valorant shop notifications",
        )
    })
}

// This function runs the appropriate handler for each command
pub fn run_command_handler(command: &ApplicationCommandInteraction) -> String {
    return match command.data.name.as_str() {
        "register" => register_command_handler(),
        _ => String::from("not implemented"),
    };
}

// Helper function to send an ephemeral reply with the supplied message
pub async fn send_ephemeral_reply(
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
