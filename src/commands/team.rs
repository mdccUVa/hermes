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
use crate::{
    student, team,
    team::Team,
    utils::{self, get_guild_id, get_triggering_student},
    Context, Error,
};
use poise::serenity_prelude::User;

// TODO: Do something with default member permissions for student commands.

#[poise::command(
    slash_command,
    subcommands("create", "invite", "invitations", "join", "leave", "rename"),
    subcommand_required,
    guild_only
)]
pub async fn team(_: Context<'_>) -> Result<(), Error> {
    // This function will not be executed, as the command has subcommands.
    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Create and join a new team, and invite other students to join it."
    ),
    description_localized(
        "es-ES",
        "Create and join a new team, and invite other students to join it."
    )
)]
#[hermes::log_cmd]
pub async fn create(
    ctx: Context<'_>,
    #[description = "The other students to invite to the team."] others: Vec<User>,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut student = get_triggering_student!(ctx);

    // Get if the user is already in a team:
    if student.get_team_id(&gid).is_some() {
        ctx.reply("You are already in a team in this server.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to create another team.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    }

    // Check the amount of invited students do not exceed the allowed time size:
    let config = utils::load_config(&gid);
    if others.len() > (config.team_capacity - 1) as usize {
        ctx.reply(format!(
            "You can only invite up to {} other student(s) to the team.",
            config.team_capacity - 1
        ))
        .await
        .expect(
            format!(
                "[team] Failed to send reply after user {} tried to invite more students than \
                allowed to a new team.",
                student.id()
            )
            .as_str(),
        );

        return Ok(());
    }

    // Collect the students that can be invited:
    let mut students_to_invite = Vec::with_capacity(others.len());
    for other in others {
        if other.id == student.id() {
            ctx.reply("You cannot invite yourself to your own team.")
                .await
                .expect(
                    format!(
                        "[team] Failed to send reply after user {} tried to invite themself to \
                        their own team.",
                        student.id()
                    )
                    .as_str(),
                );

            continue;
        }

        let other_student = student::get_existing_student!(&other.id);

        // Check if the student is already in a team:
        if other_student.get_team_id(&gid).is_some() {
            ctx.reply(format!(
                "<@{}> is already in a team in this server.",
                other.id
            ))
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to invite to their team \
                    a student already in another team.",
                    student.id()
                )
                .as_str(),
            );

            continue;
        }

        // Add the students to the list of students to invite when the team is created:
        students_to_invite.push(other_student);
    }

    // Create guild's team info, if it does not exist:
    if team::get_guild_team_info(&gid).is_none() {
        team::GuildTeamInfo::new(gid, config.team_prefix);
    }

    // Create team:
    let mut team = Team::new(gid, team::register_team(&gid));
    team.add_member(&mut student);

    // Send the invitations:
    for mut other_student in students_to_invite {
        other_student.add_team_request(gid, team.id().clone(), student.id().clone());
    }

    // Reply to confirm the creation of the team:
    let tid = team.id();
    ctx.reply(format!(
        "Team {} has been created successfully.\n\
        Tell your partner(s) to use `/team join {}` to join the team, \
        or `/team invitations` to check their invitations.",
        tid, tid
    ))
    .await
    .expect(
        format!(
            "[team] Failed to send reply after user {} created a team.",
            student.id()
        )
        .as_str(),
    );

    Ok(())
}

// MAYBE LATER: pretty-print with some custom embed or something?
#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Check your pending team invitations."),
    description_localized("es-ES", "Check your pending team invitations.")
)]
#[hermes::log_cmd]
pub async fn invitations(ctx: Context<'_>) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let student = get_triggering_student!(ctx);

    // Get the team invitations:
    let Some(team_requests) = student.get_team_requests(&gid) else {
        ctx.reply("You do not have any team invitations.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} checked their non-existant \
                    team invitations.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    };
    if team_requests.is_empty() {
        ctx.reply("You do not have any team invitations.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} checked their empty \
                team invitations.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    }
    // Reply with the team requests:
    let mut reply = "You have the following team invitations:\n".to_string();
    // I could use a map here, but I think casting inside the loop is prettier.
    for req in team_requests {
        let (team_id, sender_id) = req.into();
        reply.push_str(format!("- Team {} by <@{}>\n", team_id, sender_id).as_str());
    }

    ctx.reply(reply).await.expect(
        format!(
            "[team] Failed to send reply after user {} checked their team invitations.",
            student.id()
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Join an existing team."),
    description_localized("es-ES", "Join an existing team.")
)]
#[hermes::log_cmd]
pub async fn join(
    ctx: Context<'_>,
    // TODO: Autocomplete with the teams the student was invited to.
    #[description = "The team to join. You should have been invited to join it."]
    #[rename = "team"]
    team_id: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut student = get_triggering_student!(ctx);

    // Check if the student is already in a team:
    if student.get_team_id(&gid).is_some() {
        ctx.reply("You are already in a team in this server.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to join another team.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    }

    // Check the student was invited to the team:
    let Some(team_requests) = student.get_team_requests(&gid) else {
        ctx.reply("You were not invited to that team.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to join a team without \
                being invited on that server.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    };
    if team_requests
        .iter()
        .filter(|req| req.team_id() == &team_id)
        .collect::<Vec<_>>()
        .is_empty()
    {
        ctx.reply("You were not invited to that team.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to join a team without \
            being invited.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    }

    // Get the team:
    let mut team = team::get_existing_team!(&gid, &team_id);

    // Join the team:
    team.add_member(&mut student);

    // Reply, as confirmation:
    ctx.reply(format!("You have joined team {} successfully.", team_id))
        .await
        .expect(
            format!(
                "[team] Failed to send reply after user {} joined team {}.",
                student.id(),
                team_id
            )
            .as_str(),
        );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Leave your current team."),
    description_localized("es-ES", "Leave your current team.")
)]
#[hermes::log_cmd]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut student = get_triggering_student!(ctx);

    // Check if the student is in a team:
    let Some(team_id) = student.get_team_id(&gid) else {
        ctx.reply("You are not in a team in this server.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to leave a team without \
                    being in one.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    };

    // Get the team:
    let mut team = team::get_existing_team!(&gid, &team_id);

    // Check the team is not confirmed:
    if team.confirmed() {
        ctx.reply("You can no longer leave your team, as it is definitive.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to leave a confirmed team.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    }

    // Leave the team:
    team.remove_member(&mut student);

    // Reply, as confirmation:
    ctx.reply(format!("You have left team {} successfully.", team.id()))
        .await
        .expect(
            format!(
                "[team] Failed to send reply after user {} left team {}.",
                student.id(),
                team.id()
            )
            .as_str(),
        );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Rename your team (for aesthetic effects)."),
    description_localized("es-ES", "Rename your team (for aesthetic effects).")
)]
#[hermes::log_cmd]
pub async fn rename(
    ctx: Context<'_>,
    #[description = "The new name for the team."] new_name: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let student = get_triggering_student!(ctx);

    // Check if the student is in a team:
    let Some(team_id) = student.get_team_id(&gid) else {
        ctx.reply("You are not in a team in this server.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to rename their team without \
                being in one.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    };

    // Get the team:
    let mut team = team::get_existing_team!(&gid, &team_id);

    // Rename the team:
    team.change_name(new_name.clone());

    // Reply, as confirmation:
    ctx.reply(format!(
        "Team {} has been correctly renamed to \"{}\".",
        team.id(),
        new_name
    ))
    .await
    .expect(
        format!(
            "[team] Failed to send reply after user {} renamed team {} to \"{}\".",
            student.id(),
            team.id(),
            new_name
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Invite other students to join your current team."),
    description_localized("es-ES", "Invite other students to join your current team.")
)]
#[hermes::log_cmd]
pub async fn invite(
    ctx: Context<'_>,
    #[description = "The other students to invite to the team."] others: Vec<User>,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let student = get_triggering_student!(ctx);

    // Check if the student is in a team:
    let Some(team_id) = student.get_team_id(&gid) else {
        ctx.reply("You are not in a team in this server.")
            .await
            .expect(
                format!(
                "[team] Failed to send reply after user {} tried to invite someone to their team \
                without being in one.",
                student.id()
            )
                .as_str(),
            );

        return Ok(());
    };

    // Get the team:
    let team = team::get_existing_team!(&gid, &team_id);

    // Check the team is not confirmed:
    if team.confirmed() {
        ctx.reply("You can no longer invite other students to your team, as it is definitive.")
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to invite someone to a \
                    confirmed team.",
                    student.id()
                )
                .as_str(),
            );

        return Ok(());
    }

    // Check the amount of invited students do not exceed the allowed team size:
    let config = utils::load_config(&gid);
    // FIXME MINOR: This does not account for already existing invitations.
    let remaining_capacity = config.team_capacity as usize - team.members().len();
    if others.len() > remaining_capacity {
        ctx.reply(format!(
            "You can only invite up to {} other student(s) to the team.",
            remaining_capacity
        ))
        .await.expect(
            format!(
                "[team] Failed to send reply after user {} tried to invite more students than allowed to their team.",
                student.id()
            )
            .as_str(),
        );

        return Ok(());
    }

    // Collect the students that can be invited:
    let mut students_to_invite = Vec::with_capacity(others.len());
    for other in others {
        if other.id == student.id() {
            ctx.reply("You cannot invite yourself to your own team.")
                .await
                .expect(
                    format!(
                        "[team] Failed to send reply after user {} tried to invite themself to \
                        their own team.",
                        student.id()
                    )
                    .as_str(),
                );

            continue;
        }

        let other_student = student::get_existing_student!(&other.id);

        // Check if the student is already in a team:
        if other_student.get_team_id(&gid).is_some() {
            ctx.reply(format!(
                "<@{}> is already in a team in this server.",
                other.id
            ))
            .await
            .expect(
                format!(
                    "[team] Failed to send reply after user {} tried to invite to their team \
                    a student already in another team.",
                    student.id()
                )
                .as_str(),
            );

            continue;
        }

        // Add the students to the list of students to invite when the team is created:
        students_to_invite.push(other_student);
    }

    // Send the invitations:
    for mut other_student in students_to_invite {
        other_student.add_team_request(gid, team.id().clone(), student.id().clone());
    }

    // Reply to confirm the sending of the invitations:
    ctx.reply("Invitations to the other students have been sent successfully.")
        .await
        .expect(
            format!(
                "[team] Failed to send reply after user {} correctly invited to their team.",
                student.id()
            )
            .as_str(),
        );

    Ok(())
}
