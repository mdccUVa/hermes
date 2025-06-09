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
use crate::{student, utils, utils::get_guild_id, Context, Error};

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    description_localized(
        "en-US",
        "Get your history of previous Tablón request. (Maximum of 30.)"
    ),
    description_localized(
        "es-ES",
        "Get your history of previous Tablón request. (Maximum of 30.)"
    )
)]
#[hermes::log_cmd]
pub async fn history(ctx: Context<'_>) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    let tablon_url = crate::utils::load_config(&gid).tablon_url;

    // Get the request history for the student triggering the commnad:
    let student = utils::get_triggering_student!(ctx);
    let Some(hist) = student.request_history().get(&gid) else {
        ctx.reply("You don't have any request sent through Hermes in this server. Try using the `/request` command first!").await.expect(
            format!(
                "[history] Couldn't send message to user with no history {} ({})",
                student.name(),
                student.id()
            )
            .as_str(),
        );

        return Ok(());
    };

    // Get at most last 30 requests:
    let requests = hist.iter().rev().take(30).collect::<Vec<_>>();
    let mut reply = "**Last requests sent to Tablón:**\n".to_string();

    for req in requests {
        let req_url = format!("<{}/request?rid={}>\n", tablon_url, req);
        reply.push_str(&req_url);
    }

    // Send the reply:
    ctx.reply(reply).await.expect(
        format!(
            "[history] Couldn't send the history message to user {} ({})",
            student.name(),
            student.id()
        )
        .as_str(),
    );

    Ok(())
}
