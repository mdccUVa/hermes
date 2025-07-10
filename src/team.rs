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
use crate::{student, student::Student, team, utils};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::{
    collections::{HashMap, HashSet},
    fs,
};

/// Data structure defining a team of students that communicate with Tablón and compete in its
/// leaderboards.
///
/// Team IDs must be fomed by a team prefix and a number (e.g. g110).
///
/// The team's identifier is temporarily set as its name.
///
/// Confirmed teams are "definitive", and ready to be used to authenticate in Tablón (if a password
/// has been set).
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Serialize, Deserialize, Getters, CopyGetters)]
pub struct Team {
    /// Team identifier (immutable).
    #[getset(get = "pub")]
    id: String,
    /// Password for the team (admin-managed).
    #[getset(get = "pub")]
    pass: Option<String>,
    /// Identifier for the server the team belongs to.
    #[getset(get = "pub")]
    guild: GuildId,
    /// Team name (mutable).
    #[getset(get = "pub")]
    name: String,
    /// Team members (stored as Discord identifiers).
    #[getset(get = "pub")]
    members: HashSet<UserId>,
    /// Status of the formation of the team.
    #[getset(get_copy = "pub")]
    confirmed: bool,
}

impl Team {
    /// Constructor for a team given the identifier of the guild it belongs to, and the team's
    /// identifier.
    pub fn new(guild_id: GuildId, id: String) -> Team {
        // Get the password for the team, if in the guild's team info:
        let pass = match team::get_guild_team_info(&guild_id) {
            Some(info) => info.passwords.get(&id).cloned(),
            None => None,
        };

        let res = Self {
            id: id.clone(),
            pass,
            guild: guild_id,
            name: id,
            members: HashSet::with_capacity(2),
            confirmed: false,
        };

        res.save();

        // Add team to the corresponding guild's name map:
        let mut name_map = utils::load_namemap(&res.guild);
        name_map.insert(res.id.clone(), res.id.clone());
        utils::update_namemap_persistence(&name_map, &res.guild);

        res
    }

    /// Adds the given user to the team. If the team reaches its maximum capacity, it also confirms
    /// the team.
    ///
    /// Team capacity must have been set as an environmental variable beforehand.
    pub fn add_member(&mut self, student: &mut Student) {
        if !self.members.insert(student.id().clone()) {
            return;
        }

        student.add_team(self.guild.clone(), self.id.clone(), self.pass.clone());

        self.save();
    }

    /// Changes the team's name, for customization purposes.
    pub fn change_name(&mut self, name: String) {
        let mut name_map = utils::load_namemap(&self.guild);

        if name_map.contains_key(name.as_str()) {
            return;
        }

        self.name = name;
        name_map.insert(self.name.clone(), self.id.clone());
        utils::update_namemap_persistence(&name_map, &self.guild);

        self.save();
    }

    /// Sets the team's password.
    pub fn set_password(&mut self, password: String) {
        self.pass = Some(password.clone());

        for member in self.members.iter() {
            student::get_student(member)
                .expect(
                    format!(
                        "[Team {}-{}]Could not find student with ID {}.",
                        self.guild, self.name, member
                    )
                    .as_str(),
                )
                .set_password(&self.guild, password.clone());
        }

        self.save();
    }

    /// Removes the given user from the team.
    pub fn remove_member(&mut self, student: &mut Student) {
        if !self.members.remove(&student.id()) {
            return;
        }

        student.remove_team(&self.guild);

        if !self.members.is_empty() {
            self.save();
        } else {
            // Delete from the system if it was emptied out:
            self.delete();
            let mut name_map = utils::load_namemap(&self.guild);
            name_map.remove(&self.name);
            utils::update_namemap_persistence(&name_map, &self.guild);

            // Note down a hole in the guild's team identifiers:
            let mut info = get_existing_guild_team_info!(&self.guild);
            info.holes.push(self.id.clone());
            info.save();
        }
    }

    /// Confirms the team, making it immutable.
    pub fn confirm(&mut self) {
        self.confirmed = true;

        self.save();
    }

    /// Unconfirms the team, making it mutable again.
    pub fn unconfirm(&mut self) {
        self.confirmed = false;

        self.save();
    }

    /// Deletes the team from the system.
    pub fn delete(&self) {
        // Remove all members from the team, if any reamining:
        for member in self.members.iter() {
            student::get_student(member)
                .expect(
                    format!(
                        "[Team {}-{}] Could not find student with ID {}.",
                        self.guild, self.id, member
                    )
                    .as_str(),
                )
                .remove_team(&self.guild);
        }

        // Delete the persistance file for this team:
        fs::remove_file(format!("guilds/{}/teams/{}.json", self.guild, self.id)).expect(
            format!(
                "[Team {}-{}] Could not delete the persistance (JSON) file for the team.",
                self.guild, self.id,
            )
            .as_str(),
        );
    }

    /// Saves the team's information to disk as a JSON file.
    ///
    /// Team files are saved as `<guild_id>/teams/<team_id>.json`.
    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).expect(
            format!(
                "[Team {}-{}] Could not serialize team struct.",
                self.guild, self.id
            )
            .as_str(),
        );

        // Suppose `guilds/<gid>/teams/` directory exists.
        fs::write(
            format!("guilds/{}/teams/{}.json", self.guild, self.id),
            json,
        )
        .expect(
            format!(
                "[Team {}-{}] Could not write team file to disk.",
                self.guild, self.id
            )
            .as_str(),
        );
    }

    /// Loads a Team instance from a JSON string and returns it.
    pub fn from_json(json: &str) -> Team {
        serde_json::from_str(json).expect("[Team] Could not parse data as valid JSON.")
    }

    /// Loads a Team instance saved as JSON from disk and returns it.
    pub fn load(guild_id: &String, team_id: &String) -> Team {
        let json_str = fs::read_to_string(format!("guilds/{}/teams/{}.json", guild_id, team_id))
            .expect(
                format!(
                    "[Team] Could not load file guilds/{}/teams/{}.json.",
                    guild_id, team_id
                )
                .as_str(),
            );
        Self::from_json(&json_str)
    }
}

/// Data structure grouping some persistent per-guild information about teams.
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Serialize, Deserialize, Getters, CopyGetters)]
pub struct GuildTeamInfo {
    /// Identifier for the guild corresponding to this information, for convenience reasons.
    guild_id: GuildId,
    /// The prefix for the team identifiers.
    // It is responsibility of the bot to propagate this value if it changes in the Guild's
    // BotConfig.
    prefix: String,
    /// Number of teams created in the guild.
    #[getset(get_copy = "pub")]
    count: u16,
    /// Passwords for each team, already created or future.
    passwords: HashMap<String, String>,
    /// Team identifiers that were used in the past, but not anymore.
    #[getset(get = "pub")]
    holes: Vec<String>,
}

impl GuildTeamInfo {
    /// Constructor for a GuildTeamInfo object.
    pub fn new(guild_id: GuildId, prefix: String) -> GuildTeamInfo {
        let res = Self {
            guild_id,
            prefix,
            count: 0,
            passwords: HashMap::with_capacity(13),
            holes: Vec::new(),
        };

        res.save();

        res
    }

    /// Updates the prefix for the guild's team's identifiers.
    pub fn update_prefix(&mut self, new_prefix: String) {
        self.prefix = new_prefix;

        self.save();
    }

    /// Registers a new team creation in the guild, returning the identifier it should use, and
    /// incrementing the count if a new identifier is used.
    pub fn register_new_team(&mut self) -> String {
        // Return a previously used identifier, if available:
        if !self.holes.is_empty() {
            let reused_id = self.holes.pop().expect(
                format!(
                    "[GuildTeamInfo] Could not pop a hole from the guild {}'s team identifiers.",
                    self.guild_id
                )
                .as_str(),
            );

            self.save();

            return reused_id;
        }

        // Increment the count and return a new team's identifier:
        self.count += 1;

        self.save();

        format!("{}{:02}", self.prefix, self.count)
    }

    /// Registers a specific team creation in the guild, given its identifier.
    /// The team count is incremented accordingly.
    /// Panics if the identifier is already in use.
    pub fn register_specific_team(&mut self, team_id: &String) {
        // Extract the number from the identifier, to update the count if necessary:
        let team_num = team_id
            .chars()
            .skip(self.prefix.len())
            .collect::<String>()
            .parse::<u16>()
            .expect(
                format!(
                    "[GuildTeamInfo] Could not parse team number from identifier {}.",
                    team_id
                )
                .as_str(),
            );

        if team_num > self.count {
            // Add the in-between teams as holes:
            for i in self.count..team_num {
                self.holes.push(format!("{}{:02}", self.prefix, i));
            }
            self.count = team_num;
        } else {
            // Check if the identifier is already in use, or a hole:
            if self.holes.contains(&team_id) {
                self.holes.retain(|id| id != team_id);
            } else {
                // FIXME MINOR: This should probably propagate an error.
                panic!(
                    "[GuildTeamInfo] Team identifier {} is already in use in guild {}.",
                    team_id, self.guild_id
                );
            }
        }

        self.save();
    }

    /// Discards an identifier for a team that was registered but will not be used.
    pub fn discard_team(&mut self, team_id: String) {
        self.holes.push(team_id);

        self.save();
    }

    /// Sets the passwords for the guild's teams.
    pub fn update_passwords(&mut self, passwords: HashMap<String, String>) {
        self.passwords = passwords;

        self.save();
    }

    /// Saves the guild's team information to disk as a JSON file.
    ///
    /// Team files are saved as `<guild_id>/teams/info.json`.
    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).expect(
            format!(
                "[GuildTeamInfo] Could not serialize guild team info struct for guild {}.",
                self.guild_id
            )
            .as_str(),
        );

        // Suppose `guilds/<gid>/teams/` directory exists.
        fs::write(format!("guilds/{}/teams/info.json", self.guild_id), json).expect(
            format!(
                "[GuildTeamInfo] Could not write guild team info file for guild {} to disk.",
                self.guild_id
            )
            .as_str(),
        );
    }

    /// Loads a GuildTeamInfo instance from a JSON string and returns it.
    pub fn from_json(json: &str) -> GuildTeamInfo {
        serde_json::from_str(json).expect("[GuildTeamInfo] Could not parse data as valid JSON.")
    }

    /// Loads a GuildTeamInfo instance saved as JSON from disk and returns it.
    pub fn load(guild_id: &String) -> GuildTeamInfo {
        let json_str = fs::read_to_string(format!("guilds/{}/teams/info.json", guild_id)).expect(
            format!(
                "[GuildTeamInfo] Could not load file guilds/{}/teams/info.json.",
                guild_id
            )
            .as_str(),
        );
        Self::from_json(&json_str)
    }
}

/* Static methods: */

/// Retrieve a Team object given its guild and ID, if it exists.
pub fn get_team(guild_id: &GuildId, team_id: &String) -> Option<Team> {
    if let Ok(json) =
        fs::read_to_string(format!("guilds/{}/teams/{}.json", guild_id, team_id).as_str())
    {
        Some(
            serde_json::from_str(&json).expect(
                format!(
                    "[Team] Could not parse guilds/{}/teams/{}.json as valid JSON.",
                    guild_id, team_id
                )
                .as_str(),
            ),
        )
    } else {
        None
    }
}

macro_rules! get_existing_team {
    ($guild_id:expr, $team_id:expr) => {
        team::get_team($guild_id, $team_id).expect(
            format!(
                "[Team] Could not find team with ID {} in guild {} in the system.",
                $team_id, $guild_id
            )
            .as_str(),
        )
    };
}
pub(crate) use get_existing_team;

/// Retrieve a Team object given its guild and ID, if it exists, or create it otherwise.
pub fn get_or_create_team(guild_id: &GuildId, team_id: &String) -> Team {
    if let Some(team) = get_team(guild_id, team_id) {
        team
    } else {
        Team::new(guild_id.clone(), team_id.clone())
    }
}

/// Retrieve a GuildTeamInfo object given its guild, if it exists.
pub fn get_guild_team_info(guild_id: &GuildId) -> Option<GuildTeamInfo> {
    if let Ok(json) = fs::read_to_string(format!("guilds/{}/teams/info.json", guild_id).as_str()) {
        Some(
            serde_json::from_str(&json).expect(
                format!(
                    "[GuildTeamInfo] Could not parse guilds/{}/teams/info.json as valid JSON.",
                    guild_id
                )
                .as_str(),
            ),
        )
    } else {
        None
    }
}

macro_rules! get_existing_guild_team_info {
    ($guild_id:expr) => {
        team::get_guild_team_info($guild_id).expect(
            format!(
                "[GuildTeamInfo] Could not find team info for guild {}.",
                $guild_id
            )
            .as_str(),
        )
    };
}
pub(crate) use get_existing_guild_team_info;

/// Registers a new team creation in the given guild, returning the identifier it should use, and
/// incrementing the count.
///
/// This is a convenience shortcut method that avoids having to retrieve the guild's team info
/// object.
///
/// The guild's team info object must have been created beforehand.
pub fn register_team(guild_id: &GuildId) -> String {
    get_existing_guild_team_info!(guild_id).register_new_team()
}
