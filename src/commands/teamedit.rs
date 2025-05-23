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
use crate::student;
use crate::team;
use crate::team::GuildTeamInfo;
use crate::utils;
use crate::utils::get_guild_id;
use crate::{Context, Error};
use poise::serenity_prelude::User;

#[poise::command(
    slash_command,
    subcommands(
        "r#move",
        "add",
        "remove",
        "unconfirm",
        "confirm",
        "password",
        "rename"
    ),
    subcommand_required,
    default_member_permissions = "MANAGE_GUILD",
    guild_only
)]
pub async fn teamedit(_: Context<'_>) -> Result<(), Error> {
    // This function will not be executed, as the command has subcommands.
    Ok(())
}

// TODO: Difference between move and add?
#[poise::command(
    slash_command,
    ephemeral,
    description_localized(
        "en-US",
        "Move a student to a team, leaving their previous one (if any)."
    ),
    description_localized(
        "es-ES",
        "Move a student to a team, leaving their previous one (if any)."
    )
)]
#[hermes::log_cmd]
pub async fn r#move(
    ctx: Context<'_>,
    #[description = "The student to move."]
    #[rename = "student"]
    user: User,
    #[description = "The new team to move the student to."] new_team: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut student = student::get_student_from_user!(user);

    // Retrieve the old team of the student, if any, and remove the student from it:
    if let Some(old_team_id) = student.get_team_id(&gid) {
        team::get_existing_team!(&gid, &old_team_id).remove_member(&mut student);
    }

    // Add the student to the new team:
    team::get_or_create_team(&gid, &new_team).add_member(&mut student);

    // Reply, as confirmation:
    ctx.reply(
        format!(
            "Correctly moved student <@{}> to team {}.",
            user.id, new_team
        )
        .to_string(),
    )
    .await
    .expect(
        format!(
            "[teamedit] Failed to send reply after moving {} to team {} in guild {}.",
            user.id, new_team, gid
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
        "Add a student to a specific team. Creates the team if it does not exist."
    ),
    description_localized(
        "es-ES",
        "Add a student to a specific team. Creates the team if it does not exist."
    )
)]
#[hermes::log_cmd]
pub async fn add(
    ctx: Context<'_>,
    #[description = "The student to add to the team."]
    #[rename = "student"]
    user: User,
    #[description = "The team to add the student to."] team: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut student = student::get_student_from_user!(user);

    // Register team, if it does not exist:
    if team::get_team(&gid, &team).is_none() {
        // Create guild team info file, if it does not exist:
        let mut info = match team::get_guild_team_info(&gid) {
            Some(info) => info,
            None => {
                let prefix = utils::load_config(&gid).team_prefix;
                GuildTeamInfo::new(gid, prefix)
            }
        };

        // Register the team:
        info.register_specific_team(&team);
    }

    // Add the student to the team:
    team::get_or_create_team(&gid, &team).add_member(&mut student);

    // Reply, as confirmation:
    ctx.reply(format!("Correctly added student <@{}> to team {}.", user.id, team).to_string())
        .await
        .expect(
            format!(
                "[teamedit] Failed to send reply after adding {} to team {} in guild {}.",
                user.id, team, gid
            )
            .as_str(),
        );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Remove a student from their team."),
    description_localized("es-ES", "Remove a student from their team.")
)]
#[hermes::log_cmd]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The student to remove from the team."]
    #[rename = "student"]
    user: User,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);
    let mut student = student::get_student_from_user!(user);

    // Retrieve the team of the student, or notify if they are not in any team:
    let Some(team_id) = student.get_team_id(&gid) else {
        ctx.reply(
            format!("Student <@{}> is not in any team on this guild.", user.id).to_string(),
).await.expect(
            format!(
                "[teamedit] Failed to send reply after attempting to remove {} from their non-existant team in {}.",
                user.id, gid
            )
            .as_str(),
        );

        return Ok(());
    };

    // Remove the student from the team:
    team::get_existing_team!(&gid, &team_id).remove_member(&mut student);

    // Reply, as confirmation:
    ctx.reply(
        format!(
            "Correctly removed student <@{}> from team {}.",
            user.id, team_id
        )
        .to_string(),
    )
    .await
    .expect(
        format!(
            "[teamedit] Failed to send reply after removing {} from team {} in guild {}.",
            user.id, team_id, gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Unconfirm a team, to make it modifiable."),
    description_localized("es-ES", "Unconfirm a team, to make it modifiable.")
)]
#[hermes::log_cmd]
pub async fn unconfirm(
    ctx: Context<'_>,
    #[description = "The team to unconfirm."]
    #[rename = "team"]
    team_id: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    // Retrieve the team, or notify if it does not exist:
    let Some(mut team) = team::get_team(&gid, &team_id) else {
        ctx.reply(format!("Team {} does not exist in this guild.", team_id).to_string()
).await.expect(
            format!(
                "[teamedit] Failed to send reply after attempting to unconfirm non-existant team {} in guild {}.",
                team_id, gid
            )
            .as_str(),
        );

        return Ok(());
    };

    // Unconfirm the team:
    team.unconfirm();

    // Reply, as confirmation:
    ctx.reply(
        format!(
            "Correctly unconfirmed team {}. It is now editable.",
            team_id
        )
        .to_string(),
    )
    .await
    .expect(
        format!(
            "[teamedit] Failed to send reply after unconfirming team {} in guild {}.",
            team_id, gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Confirm a team, to make it ready to be used with Tablón."),
    description_localized("es-ES", "Confirm a team, to make it ready to be used with Tablón.")
)]
#[hermes::log_cmd]
pub async fn confirm(
    ctx: Context<'_>,
    #[description = "The team to confirm."]
    #[rename = "team"]
    team_id: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    // Retrieve the team, or notify if it does not exist:
    let Some(mut team) = team::get_team(&gid, &team_id) else {
        ctx.reply(format!("Team {} does not exist in this guild.", team_id).to_string()
).await.expect(
            format!(
                "[teamedit] Failed to send reply after attempting to confirm non-existant team {} in guild {}.",
                team_id, gid
            )
            .as_str(),
        );

        return Ok(());
    };

    // Confirm the team:
    team.confirm();

    // Reply, as confirmation:
    ctx.reply(
        format!(
            "Correctly confirmed team {}. It is no longer editable.",
            team_id
        )
        .to_string(),
    )
    .await
    .expect(
        format!(
            "[teamedit] Failed to send reply after confirming team {} in guild {}.",
            team_id, gid
        )
        .as_str(),
    );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Set the password of a team."),
    description_localized("es-ES", "Set the password of a team.")
)]
#[hermes::log_cmd]
pub async fn password(
    ctx: Context<'_>,
    #[description = "The team to set the password for."]
    #[rename = "team"]
    team_id: String,
    #[description = "The new password for the team."] password: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    // Retrieve the team, or notify if it does not exist:
    let Some(mut team) = team::get_team(&gid, &team_id) else {
        ctx.reply(format!("Team {} does not exist in this guild.", team_id).to_string()
).await.expect(
            format!(
                "[teamedit] Failed to send reply after attempting to set password for non-existant team {} in guild {}.",
                team_id, gid
            )
            .as_str(),
        );

        return Ok(());
    };

    // Set the password for the team:
    team.set_password(password);

    // Reply, as confirmation:
    ctx.reply(format!("Correctly updated teh password for team {}.", team_id).to_string())
        .await
        .expect(
            format!(
                "[teamedit] Failed to send reply after updating password for team {} in guild {}.",
                team_id, gid
            )
            .as_str(),
        );

    Ok(())
}

#[poise::command(
    slash_command,
    ephemeral,
    description_localized("en-US", "Rename a team."),
    description_localized("es-ES", "Rename a team.")
)]
#[hermes::log_cmd]
pub async fn rename(
    ctx: Context<'_>,
    #[description = "The team to rename."]
    #[rename = "team"]
    team_id: String,
    #[description = "The new name for the team."] new_name: String,
) -> Result<(), Error> {
    let gid = get_guild_id!(ctx);

    // Retrieve the team, or notify if it does not exist:
    let Some(mut team) = team::get_team(&gid, &team_id) else {
        ctx.reply(format!("Team {} does not exist in this guild.", team_id).to_string()
).await.expect(
            format!(
                "[teamedit] Failed to send reply after attempting to rename non-existant team {} in guild {}.",
                team_id, gid
            )
            .as_str(),
        );

        return Ok(());
    };

    // Rename the team:
    team.change_name(new_name);

    // Reply, as confirmation:
    ctx.reply(format!("Correctly renamed team {} to {}.", team_id, team.name()).to_string())
        .await
        .expect(
            format!(
                "[teamedit] Failed to send reply after renaming team {} to {} in guild {}.",
                team_id,
                team.name(),
                gid
            )
            .as_str(),
        );

    Ok(())
}
