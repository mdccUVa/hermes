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
use crate::{Context, Error};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    description_localized("en-US", "Show this software's license information.."),
    description_localized("es-ES", "Recibir información de la licencia del software.")
)]
#[hermes::log_cmd]
pub async fn license(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(
        "Hermes  Copyright (C) 2025  Manuel de Castro <manuel@infor.uva.es>\n\
        This program comes with ABSOLUTELY NO WARRANTY.\n\
        This is free software, and you are welcome to redistribute it under certain conditions.\n\
        This program is licensed under the GNU General Public License v3.0.\
        For more information, see <https://www.gnu.org/licenses/>.",
    )
    .await
    .expect("[license] Failed to send reply.");

    Ok(())
}
