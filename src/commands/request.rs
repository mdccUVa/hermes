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

use crate::student;
use crate::utils;
use crate::utils::get_guild_id;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use regex::Regex;
use std::io::Write;
use std::process::Command;

#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    description_localized("en-US", "Send a program to Tablón."),
    description_localized("es-ES", "Send a program to Tablón.")
)]
#[hermes::log_cmd]
pub async fn request(
    ctx: Context<'_>,
    #[description = "File to send to Tablón."] file: serenity::Attachment,
    #[description = "Additional arguments to send to Tablón (queue, threads, processes, and program args)."]
    extra_args: Option<String>,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    let mut student = utils::get_triggering_student!(ctx);

    // Retrieve the credentials of the student, and handle the possible error:
    let Some(credentials) = student.get_credentials(&gid) else {
        ctx.reply("**Error:** You cannot send requests to Tablón, as you are not part of a team.")
            .await
            .expect(
                format!(
                    "[request] Failed to send reply to student with no credentials {}.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    };
    // Retrieve the team of the credentials:
    let team = credentials.team();
    // Retrieve the password of the credentials, and handle the possible error:
    let Some(password) = credentials.password() else {
        ctx.reply("**Error:** You cannot send requests to Tablón, as your team has not been registered yet.")
            .await
            .expect(
                format!(
                    "[request] Failed to send reply to student with no password {}.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    };

    // Get the correct args:
    let extra_args = if let Some(given_args) = extra_args {
        match given_args.as_str() {
            "l" => {
                if let Some(last_command) = student.get_last_command(&gid) {
                    last_command.clone()
                } else {
                    ctx.reply(
                        "**Error:** Can't send request without all arguments, as there is no previous command.",
                    )
                    .await
                    .expect(
                        format!(
                            "[request] Failed to send reply to student {}, with no previous command.",
                            student.id()
                        )
                        .as_str(),
                    );

                    return Ok(());
                }
            }
            _ => given_args,
        }
    } else {
        if let Some(preferred_queue) = student.get_preferred_queue(&gid) {
            format!("-q {}", preferred_queue)
        } else {
            ctx.reply(
                "**Error:** Can't send request, as no queue was specified, and no preferred was set.",
            )
            .await
            .expect(
                format!(
                    "[request] Failed to send reply to student with unspecified queue {}.",
                    student.id()
                )
                .as_str(),
            );

            return Ok(());
        }
    };
    let args = format!("-u {} -x {} {}", team, password, extra_args);

    // Save the file to disk:
    let Ok(mut out_program) = std::fs::File::create(format!("guilds/{}/{}", gid, file.filename))
    else {
        ctx.reply(
            "**Error:** Failed to save your program to disk. Try again later, or contact an administrator.",
        )
        .await
        .expect(
            format!(
                "[request] Failed to send reply to student {}, with failed file creation.",
                student.id(),
            )
            .as_str(),
        );

        eprintln!(
            "[request] Failed to save program file to disk, sent by student {}.",
            student.id()
        );

        return Ok(());
    };
    let file_bytes = reqwest::get(&file.url)
        .await
        .expect(format!("Could not download program from URL: {}", file.url).as_str())
        .bytes()
        .await
        .expect("Could not get program bytes from http response.");
    out_program
        .write_all(&file_bytes)
        .expect(format!("Could not save program to disk: {}", file.filename).as_str());

    // TODO: Develop a way to conveniently set the client for a guild using Hermes.
    // TODO: Add Hermes identification to files, for clout 😎
    // TODO: Consider adding a request embed.

    // Equivalent CLI string:
    let req_cmd_str = format!(
        "guilds/{}/client guilds/{}/{} {}",
        gid, gid, file.filename, args
    );

    // Log request:
    let mut req_log = std::fs::OpenOptions::new()
        .append(true)
        .open(format!("guilds/{}/requests.log", gid))
        .expect(
            format!(
                "[requests] Failed to open the guild's log file for guild {}.",
                gid
            )
            .as_str(),
        );
    write!(
        req_log,
        "Request received from {} ({}): {}\n",
        student.name(),
        student.id(),
        req_cmd_str
    )
    .expect(
        format!(
            "[requests] Failed to write to the guild's log file for guild {}.",
            gid
        )
        .as_str(),
    );

    // Construct the command, execute, and handle errors:
    let mut cmd = Command::new(format!("guilds/{}/client", gid));
    cmd.arg(format!("guilds/{}/{}", gid, file.filename));
    for opt in args.split_whitespace() {
        cmd.arg(opt);
    }

    let req_output = cmd.output();

    // Remove the file sent from disk:
    std::fs::remove_file(format!("guilds/{}/{}", gid, file.filename))
        .expect(format!("Could not remove file from disk: {}", file.filename).as_str());

    // Process the client's output:
    let Ok(req_output) = req_output else {
        ctx.reply(
            "**Error:** Failed to send request to Tablón. Try again later, or contact an administrator.",
        )
        .await
        .expect(
            format!(
                "[request] Failed to send reply to student {}, with failed client response for {}.",
                student.id(),
                req_cmd_str,
            )
            .as_str(),
        );

        eprintln!(
            "[request] Failed to send request, triggered by student {} ({}). \
            Request: {}",
            student.id(),
            student.name(),
            req_cmd_str
        );

        return Ok(());
    };

    let stdout_str = String::from_utf8(req_output.stdout).expect(
        format!(
        "[request] Failed to transform the stdout of a request command to a string. Command: {}",
        req_cmd_str,
    )
        .as_str(),
    );
    ctx.reply(format!("Correctly sent the request:\n```{}```", stdout_str))
        .await
        .expect(
            format!(
            "[request] Failed to send reply to student {}, with successful client response for {}",
            student.id(),
            req_cmd_str,
        )
            .as_str(),
        );

    // Save previous command:
    student.set_last_command(gid, extra_args);

    // Save request id in the student's history.
    let req_url = stdout_str.lines().find(|line| line.starts_with("http://"));
    let req_regex = Regex::new(r"(\d+)$").expect("Failed to compile regex for request id.");
    if let Some(req_url) = req_url {
        let rid = req_regex
            .captures(req_url)
            .expect(
                format!(
                    "[request] Failed to find the request ID in the URL {}.",
                    req_url,
                )
                .as_str(),
            )
            .get(0)
            .expect(
                format!(
                    "[request] Failed to find the request ID in the URL {}.",
                    req_url,
                )
                .as_str(),
            )
            .as_str();
        let rid = rid
            .parse::<u16>()
            .expect(format!("[request] Failed to parse the request ID {}.", rid).as_str());

        student.add_request(&gid, rid);
    } else {
        let root_url = utils::load_config(&gid).tablon_url;

        ctx.reply(
            format!(
                "Ooops! I couldn't find the URL generated for your request. That's weird!\n\
                However, it seems that the request itself was sent successfully.\n\
                Please, check manually: <{}>", root_url
            )
        )
        .await
        .expect(
            format!(
                "[request] Failed to send reply to student {}, with failed request ID extraction for {}.",
                student.id(),
                req_cmd_str,
            )
            .as_str(),
        );

        eprintln!(
            "[request] Failed to find the request ID in the output of command {}\nOutput: {}",
            req_cmd_str, stdout_str,
        );

        return Ok(());
    }

    Ok(())
}
