/*
 *  Hermes - Discord bot for integrating UVa's Tablón into Discord servers.
 *  Copyright (C) 2025  Manuel de Castro <manuel@infor.uva.es>
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
use crate::{student, utils, Context, Error};
use std::fmt::Write;

#[poise::command(slash_command, subcommands("get", "set_queue"), guild_only, ephemeral)]
#[hermes::log_cmd]
pub async fn settings(ctx: Context<'_>) -> Result<(), Error> {
    // This function will not be executed, as the command has subcommands.
    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Print your current settings."),
    description_localized("es-ES", "Print your current settings.")
)]
#[hermes::log_cmd]
pub async fn get(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = utils::get_guild_id!(ctx);
    let student = utils::get_triggering_student!(ctx);

    let credentials_or_none = student.get_credentials(&guild_id);
    let queue_or_none = student.get_preferred_queue(&guild_id);
    let request_or_none = student.get_last_command(&guild_id);

    // Construct reply message in function of what settings exist:
    // Team and password:
    let mut reply_msg = "Your current settings for this server are:\n".to_string();
    if let Some(credentials) = credentials_or_none {
        let team = credentials.team();
        let password_or_none = credentials.password();
        write!(&mut reply_msg, "- Team: `{}`\n", team).unwrap();
        if let Some(password) = password_or_none {
            write!(&mut reply_msg, "- Password: ||`{}`||\n", password).unwrap();
        } else {
            write!(&mut reply_msg, "- Password: [Not set]\n").unwrap();
        }
    } else {
        write!(&mut reply_msg, "- You are not in a team in this server\n").unwrap();
    }
    // Queue:
    if let Some(queue) = queue_or_none {
        write!(
            &mut reply_msg,
            "- Default queue for requests: `{}`\n",
            queue
        )
        .unwrap();
    } else {
        write!(&mut reply_msg, "- Default queue for requests: [Not set]\n").unwrap();
    }
    // Last request command:
    if let Some(request) = request_or_none {
        write!(&mut reply_msg, "- Last request command: `{}`\n", request).unwrap();
    }

    // Reply, as confirmation:
    ctx.reply(reply_msg).await.expect(
        format!(
            "[settings] Failed to send reply after user {} accessed their settings.",
            student.id()
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Change your default queue for requests."),
    description_localized("es-ES", "Change your default queue for requests.")
)]
#[hermes::log_cmd]
pub async fn set_queue(
    ctx: Context<'_>,
    #[description = "The queue to set as default for your requests."] queue: String,
) -> Result<(), Error> {
    let guild_id = utils::get_guild_id!(ctx);
    let mut student = utils::get_triggering_student!(ctx);

    student.set_preferred_queue(guild_id, queue.clone());

    // Reply, as confirmation:
    ctx.reply(format!(
        "Your default queue for requests has been set to `{}`",
        queue
    ))
    .await
    .expect(
        format!(
            "[settings] Failed to send reply after user {} set their default queue to {}.",
            student.id(),
            queue
        )
        .as_str(),
    );

    Ok(())
}
