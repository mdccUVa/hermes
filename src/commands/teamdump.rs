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
use crate::student;
use crate::team;
use crate::utils;
use crate::utils::get_guild_id;
use crate::{Context, Error};
use poise::serenity_prelude::{CreateAttachment, CreateMessage, GuildChannel};
use poise::CreateReply;
use std::fs;

#[poise::command(
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_GUILD",
    ephemeral,
    description_localized(
        "en-US",
        "Export the guild's teams and their member's identifiers as a plain text file."
    ),
    description_localized(
        "es-ES",
        "Export the guild's teams and their member's identifiers as a plain text file."
    )
)]
pub async fn teamdump(
    ctx: Context<'_>,
    #[description = "Channel to send a message with all the teams and their members (as Discord users)."]
    channel: Option<GuildChannel>,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let config = utils::load_config(&gid);

    let prefix = config.team_prefix;
    let team_count = *team::get_existing_guild_team_info!(&gid).count();

    // Construct message and file content:
    let mut out_file = String::new();
    let mut out_msg = "## List of teams:\n\n".to_string();
    let mut out_msg_split = Vec::new(); // Messages sent on Discord.
    for i in 0..team_count {
        let tid = format!("{}{:02}", prefix, i + 1);
        if let Some(team) = team::get_team(&gid, &tid) {
            if team.members().is_empty() {
                continue;
            }

            out_msg += format!("**{}** ", tid).as_str();
            for member in team
                .members()
                .iter()
                .map(|m| student::get_existing_student!(m))
            {
                let uid = member.id();
                out_file += format!("{} {}\n", tid, uid).as_str();
                out_msg += format!("{} ", uid).as_str();
            }
            out_msg += "\n";
        };

        if out_msg.len()
            >= 2000
                - ("**gXXX:**".len()
                    + config.team_capacity as usize * "<@!000000000000000000>".len()
                    + "\n".len())
        {
            // 2000 = maximum message length.
            // 10 = "**gXXX:** " length.
            // 22 = <@!memberID> length.
            // 1 = space or newline length.
            out_msg_split.push(out_msg.clone());
            out_msg = String::new();
        }
    }
    out_msg_split.push(out_msg);

    // Send the list of teams to the channel as a message, if a channel was provided:
    if let Some(channel) = channel {
        for msg in out_msg_split {
            channel
                .send_message(&ctx.http(), CreateMessage::new().content(msg))
                .await
                .expect(
                    "[teamdump] Could not send message with the team list to the provided channel.",
                );
        }
    }

    // Send the list of teams as a plain text file:
    let file_name = format!("{}/teams/team_list.txt", gid);
    fs::write(file_name.clone(), out_file).expect("[teamdump] Could not write team list to file.");

    let msg = CreateReply::default()
        .content("List of teams on the server:")
        .attachment(
            CreateAttachment::path(file_name)
                .await
                .expect("[teamdump] Could not send the team list file as attachment."),
        )
        .ephemeral(true);
    ctx.send(msg)
        .await
        .expect("[teamdump] Could not send the message with the team list file.");

    Ok(())
}
