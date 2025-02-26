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
    guild_only,
    ephemeral,
    description_localized("en-US", "Get help for using Hermes [CURRENTLY UNUSED]."),
    description_localized("es-ES", "Get help for using Hermes [CURRENTLY UNUSED].")
)]
#[hermes::log_cmd]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("This command is still WIP, sorry! Please check back later.")
        .await
        .expect("[help] Failed to send reply.");

    Ok(())
}
