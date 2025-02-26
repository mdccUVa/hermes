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
use crate::{Context, Error};

#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD",
    guild_only,
    ephemeral,
    description_localized("en-US", "Create a new leaderboard [CURRENTLY UNUSED].")
)]
#[hermes::log_cmd]
async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Experimental leaderboard integration is still WIP, and may be discarded altogether in a future update.")
        .await
        .expect("[leaderboard] Failed to send reply.");

    Ok(())
}
