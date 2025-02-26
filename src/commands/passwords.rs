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
use std::collections::HashMap;

#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD",
    guild_only,
    ephemeral,
    description_localized("en-US", "Set the passwords for the guild's teams.")
)]
#[hermes::log_cmd]
pub async fn passwords(
    ctx: Context<'_>,
    #[description = "File with the team's passwords."] file: serenity::Attachment,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    // Read the provided file:
    let content = reqwest::get(&file.url)
        .await
        .expect(
            format!(
                "[passwords] Could not download teams file from URL: {}",
                file.url
            )
            .as_str(),
        )
        .text()
        .await
        .expect("[passwords] Could not read the teams file into a string.");

    // Password map to update the gulid's team info:
    let mut pass_map = HashMap::<String, String>::new();

    // Split the file into lines (one team-password pair per line):
    let lines: Vec<&str> = content.lines().collect();
    for line in lines {
        let mut parts = line.split_whitespace();
        let tid = parts
            .next()
            .expect(format!("Could not read team name from teams file. Line: {}", line).as_str());
        let password = parts.next().expect(
            format!(
                "Could not read team password from teams file. Line: {}",
                line
            )
            .as_str(),
        );

        // Set the password for the team, if it exists:
        if let Some(mut team) = team::get_team(&gid, &tid.to_string()) {
            team.set_password(password.to_string());
        }

        // Add the team-password pair to the password map:
        pass_map.insert(tid.to_string(), password.to_string());
    }

    // Update the guild's team info:
    let mut info = match team::get_guild_team_info(&gid) {
        Some(info) => info,
        None => {
            // Create guild team info file, if it does not exist:
            let prefix = utils::load_config(&gid).team_prefix;
            team::GuildTeamInfo::new(gid, prefix)
        }
    };
    info.update_passwords(pass_map);

    Ok(())
}
