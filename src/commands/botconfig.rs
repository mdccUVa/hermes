/*
 *  Hermes - Discord bot for integrating UVa's Tablón into Discord servers.
 *  Copyright (C) 2025  Manuel de Castro
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
extern crate reqwest;

use crate::team;
use crate::utils;
use crate::utils::get_guild_id;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::GuildChannel;
use serde_json;
use std::fs;

async fn check_on_proper_channel(ctx: Context<'_>, channel_name: &String) -> bool {
    let gid = get_guild_id!(ctx);
    let cmd_channel = ctx
        .guild_channel()
        .await
        .expect("[boconfig] The command was not invoked in a guild channel.");
    // TODO: Check if the channel exists in the guild, and send different error messages.
    // Previous attempts caused an "Future is not Send" error on the await for the ctx.reply() calls.
    if cmd_channel.name != *channel_name {
        ctx.reply(
            format!(
                "This command should only be used in the configured bot channel: #{}.",
                channel_name
            )
            .as_str(),
        )
        .await
        .expect(
            format!(
            "[botconfig] Failed to send reply using the command in an invalid channel in guild {}.",
            gid
        )
            .as_str(),
        );

        return false;
    }

    return true;
}

#[poise::command(
    slash_command,
    subcommands(
        "show",
        "tablon_url",
        "team_capacity",
        "team_prefix",
        "bot_channel",
        "lb_channel",
        "notify_leaders",
        "leader_count",
        "public_notify",
        "bot_news_channel",
        "column_separator",
        "update",
    ),
    default_member_permissions = "MANAGE_GUILD",
    guild_only,
    ephemeral
)]
#[hermes::log_cmd]
pub async fn botconfig(ctx: Context<'_>) -> Result<(), Error> {
    // This function will not be executed, as the command has subcommands.
    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Show the current configuration for the bot.")
)]
#[hermes::log_cmd]
pub async fn show(ctx: Context<'_>) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let config = utils::load_config(&gid);

    // Reply with the current configuration:
    ctx.reply(format!(
        "Current configuration:\n\
        ```json\n{}\n```",
        serde_json::to_string_pretty(&config).expect(
            format!(
                "[botconfig] Failed to serialize the config for guild {}.",
                gid
            )
            .as_str()
        )
    ))
    .await
    .expect(
        format!(
            "[botconfig] Failed to send the configuration for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Change the URL for Tablón's endpoint.")
)]
#[hermes::log_cmd]
pub async fn tablon_url(
    ctx: Context<'_>,
    #[description = "The new URL for Tablón's endpoint."] url: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.tablon_url = url.clone();
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The Tablón URL has been changed to <{}>.",
            config.tablon_url
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of Tablón URL change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Change the capacity for teams on this server.")
)]
#[hermes::log_cmd]
pub async fn team_capacity(
    ctx: Context<'_>,
    #[description = "The new capacity for teams."] capacity: u8,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.team_capacity = capacity;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The team capacity has been changed to {}.",
            config.team_capacity
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of team capacity change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Change the prefix for the IDs of the teams of this guild.")
)]
#[hermes::log_cmd]
pub async fn team_prefix(
    ctx: Context<'_>,
    #[description = "The new prefix for team IDs."] prefix: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.team_prefix = prefix.clone();
    utils::update_config_persistence(&config, &gid);

    // Propagate the update to the corresponding team guild configuration:
    if let Some(mut guild_team_config) = team::get_guild_team_info(&gid) {
        guild_team_config.update_prefix(prefix);
    }

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The team prefix has been changed to {}.",
            config.team_prefix
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of team prefix change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Change the guild's channel for usage of special admin-only bot functionalities."
    )
)]
#[hermes::log_cmd]
pub async fn bot_channel(
    ctx: Context<'_>,
    #[description = "The the new channel for admin bot usage."] channel: GuildChannel,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.bot_channel = channel.name;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The bot channel has been changed to #{}.",
            config.bot_channel
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of bot channel change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Change the guild's channel for Tablón's leaderboard.")
)]
#[hermes::log_cmd]
pub async fn lb_channel(
    ctx: Context<'_>,
    #[description = "The new channel for the leaderboard."] channel_name: GuildChannel,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.lb_channel = channel_name.name;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The leaderboard channel has been changed to #{}.",
            config.lb_channel
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of leaderboard channel change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Change whether to notify leaders of leaderboard position changes."
    )
)]
#[hermes::log_cmd]
pub async fn notify_leaders(
    ctx: Context<'_>,
    #[description = "Whether to notify leaders."] notify: bool,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.notify_leaders = notify;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "Leaderboard position notifications have been set to {}.",
            config.notify_leaders
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of leaderboard position notifications change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Change the number of teams to count as \"leaders\" for position update notifications."
    )
)]
#[hermes::log_cmd]
pub async fn leader_count(
    ctx: Context<'_>,
    #[description = "The number leaders."] count: u8,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.leader_count = count;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The number of teams to count as \"leaders\" for position update notifications has been set to {}.",
            config.leader_count
        )
        .as_str(),
    )
        .await.expect(
        format!(
            "[botconfig] Failed to send confirmation of leader count change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Change whether to notify the leaderboard changes publicly.")
)]
#[hermes::log_cmd]
pub async fn public_notify(
    ctx: Context<'_>,
    #[description = "Whether to do public notifications."] public_notify: bool,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.public_notify = public_notify;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "Leaderboard position notifications have been set to {}.",
            config.public_notify
        )
        .as_str(),
    ).await.expect(
        format!(
            "[botconfig] Failed to send confirmation of public leaderboard position notifications change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Change the channel for bot news (e.g. public position update notifications)."
    )
)]
#[hermes::log_cmd]
pub async fn bot_news_channel(
    ctx: Context<'_>,
    #[description = "The new channel for bot news."] channel: GuildChannel,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.bot_news_channel = channel.name;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The bot news channel has been changed to #{}.",
            config.bot_news_channel
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of bot news channel change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Change the separator for multi-field columns in leaderboards."
    )
)]
#[hermes::log_cmd]
pub async fn column_separator(
    ctx: Context<'_>,
    #[description = "The new separator."] separator: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut config = utils::load_config(&gid);

    // Update the configuration:
    config.column_separator = separator;
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(
        format!(
            "The column separator has been changed to {}.",
            config.column_separator
        )
        .as_str(),
    )
    .await
    .expect(
        format!(
            "[botconfig] Failed to send confirmation of column separator change for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Update the configuration for the bot with the provided or a default file."
    )
)]
#[hermes::log_cmd]
pub async fn update(
    ctx: Context<'_>,
    #[description = "JSON configuration file with the new configuration."] file: Option<
        serenity::Attachment,
    >,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    // Read the config JSON - Attachment, or default file:
    let config_json = if let Some(config_file) = file {
        // Handle attachement:
        reqwest::get(&config_file.url)
            .await
            .expect(
                format!(
                    "[botconfig update] Could not download the config file from URL: {}",
                    config_file.url
                )
                .as_str(),
            )
            .text()
            .await
            .expect("[botconfig update] Could not read the teams file into a string.")
    } else {
        // Use the default configuration file (possibly new):
        if fs::metadata("config.json").is_ok() {
            fs::read_to_string("config.json")
                .expect("[botconfig update] Could not read the default configuration file.")
        } else {
            ctx.reply("No configuration file was provided, and the default configuration file was not found.")
                .await
                .expect(
                    format!(
                        "[botconfig update] Failed to send error message for missing configuration file for guild {}.",
                        gid
                    )
                    .as_str(),
                );

            return Ok(());
        }
    };
    let config = serde_json::from_str(config_json.as_str()).expect(
        format!(
            "[botconfig update] Failed to parse the configuration file for guild {}.",
            gid
        )
        .as_str(),
    );

    // Update and save the new configuration:
    utils::update_config_persistence(&config, &gid);

    // Reply to the user, as confirmation:
    ctx.reply(format!(
        "The configuration has been updated! New configuration:\n```json\n{}\n```",
        serde_json::to_string_pretty(&config).expect(
            format!(
                "[botconfig] Failed to serialize the config for guild {}.",
                gid
            )
            .as_str()
        )
    ))
    .await
    .expect(
        format!(
            "[botconfig update] Failed to send confirmation of configuration update for guild {}.",
            gid
        )
        .as_str(),
    );

    Ok(())
}
