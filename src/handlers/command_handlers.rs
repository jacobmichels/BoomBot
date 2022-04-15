use serenity::model::interactions::application_command::ApplicationCommandInteraction;

pub fn register_command_handler(command: &ApplicationCommandInteraction) -> String {
    return format!("Check your DMs to get started with BoomBot!");
}
