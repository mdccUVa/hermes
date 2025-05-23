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
use crate::teamrequest::TeamRequest;
use crate::Credentials;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/**
 * Data structure defining a student and its preferences / configuration in the system.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Serialize, Deserialize, Getters, Setters)]
pub struct Student {
    #[getset(get = "pub")]
    id: UserId,
    #[getset(get = "pub", set = "pub")]
    name: String,
    credentials: HashMap<GuildId, Credentials>,
    preferred_queue: HashMap<GuildId, String>,
    last_command: HashMap<GuildId, String>,
    team_requests: HashMap<GuildId, Vec<TeamRequest>>,
    #[getset(get = "pub")]
    request_history: HashMap<GuildId, Vec<u16>>,
}

impl Student {
    /**
     * Constructor for a student, given a server they are in.
     *
     * Every student must be in at least one server.
     */
    pub fn new(user_id: UserId, user_name: String) -> Student {
        let res: Student = Self {
            id: user_id,
            name: user_name.clone(),
            // Containers of size 1, because it is not expected for students to be in more than one
            // server.
            credentials: HashMap::with_capacity(1),
            preferred_queue: HashMap::with_capacity(1),
            last_command: HashMap::with_capacity(1),
            team_requests: HashMap::with_capacity(1),
            request_history: HashMap::with_capacity(1),
        };

        res.save();

        res
    }

    /* Field accessors: */

    pub fn get_credentials(&self, guild: &GuildId) -> Option<&Credentials> {
        self.credentials.get(&guild)
    }

    #[inline]
    pub fn get_existing_credentials(&self, guild: &GuildId) -> &Credentials {
        self.credentials.get(&guild).expect(
            format!(
                "[Student {}] Could not find credentials for guild {}.",
                self.name, guild
            )
            .as_str(),
        )
    }

    pub fn get_preferred_queue(&self, guild: &GuildId) -> Option<&String> {
        self.preferred_queue.get(&guild)
    }

    #[inline]
    pub fn get_existing_preferred_queue(&self, guild: &GuildId) -> &String {
        self.preferred_queue.get(&guild).expect(
            format!(
                "[Student {}] Could not find a preferred queue for guild {}.",
                self.name, guild
            )
            .as_str(),
        )
    }

    pub fn get_last_command(&self, guild: &GuildId) -> Option<&String> {
        self.last_command.get(&guild)
    }

    #[inline]
    pub fn get_existing_last_command(&self, guild: &GuildId) -> &String {
        self.last_command.get(&guild).expect(
            format!(
                "[Student {}] Could not find a last command for guild {}.",
                self.name, guild
            )
            .as_str(),
        )
    }

    pub fn get_team_requests(&self, guild: &GuildId) -> Option<&Vec<TeamRequest>> {
        self.team_requests.get(&guild)
    }

    #[inline]
    pub fn get_existing_team_requests(&self, guild: &GuildId) -> &Vec<TeamRequest> {
        self.team_requests.get(&guild).expect(
            format!(
                "[Student {}] Could not find team requests for guild {}.",
                self.name, guild
            )
            .as_str(),
        )
    }

    /* Other methods: */

    /**
     * Adds a team for one of the guilds this student is in.
     */
    pub fn add_team(&mut self, guild_id: GuildId, team_id: String, team_password: Option<String>) {
        let cred = Credentials {
            team: team_id,
            password: team_password,
        };

        self.credentials.insert(guild_id, cred);

        // Remove any team requests for this guild, since the student is now in a team.
        self.team_requests.remove(&guild_id);

        self.save();
    }

    /**
     * Adds the password to the credentials of a guild the student is in.
     */
    pub fn set_password(&mut self, guild_id: &GuildId, password: String) {
        assert!(self.credentials.contains_key(guild_id));

        self.credentials
            .get_mut(guild_id)
            .expect(
                format!(
                    "[Student {}] Could not find credentials for guild {}.",
                    self.name, guild_id
                )
                .as_str(),
            )
            .password = Some(password);

        self.save();
    }

    /**
     * Removes the team for one of the guilds the student is in, given the guild identifier.
     *
     * The removed team is probably not confirmed (definitive), so their members could join and
     * leave at will.
     */
    pub fn remove_team(&mut self, guild_id: &GuildId) {
        self.credentials.remove(guild_id);

        self.save();
    }

    /**
     * Retrieves the team ID for a guild the student might be in.
     *
     * Returns `None` if the student does not have a team in the provided guild.
     */
    pub fn get_team_id(&self, guild_id: &GuildId) -> Option<String> {
        if let Some(credentials) = self.credentials.get(guild_id) {
            Some(credentials.team.clone())
        } else {
            None
        }
    }

    /**
     * Adds a new team request for the student.
     */
    pub fn add_team_request(&mut self, guild_id: GuildId, team_id: String, sender_id: UserId) {
        let request = (team_id, sender_id).into();

        if let Some(requests) = self.team_requests.get_mut(&guild_id) {
            requests.push(request);
        } else {
            self.team_requests.insert(guild_id, vec![request]);
        }

        self.save();
    }

    /**
     * Sets the preferred queue of the student for a given guild.
     */
    pub fn set_preferred_queue(&mut self, guild_id: GuildId, queue_name: String) {
        self.preferred_queue.insert(guild_id, queue_name);

        self.save();
    }

    /**
     * Sets the last request command the student used in a guild.
     */
    pub fn set_last_command(&mut self, guild_id: GuildId, command: String) {
        self.last_command.insert(guild_id, command);

        self.save();
    }

    /**
     * Adds a request to the student's request history.
     */
    pub fn add_request(&mut self, gid: &GuildId, request_id: u16) {
        if self.request_history.contains_key(gid) {
            self.request_history
                .get_mut(gid)
                .expect(
                    format!(
                        "[Student {}] No request history for guild {}.",
                        self.name, gid
                    )
                    .as_str(),
                )
                .push(request_id);
        } else {
            self.request_history.insert(gid.clone(), vec![request_id]);
        }

        self.save();
    }

    /**
     * Saves the student's information to disk as a JSON file.
     *
     * Student files are saved as `users/<username>[#discriminator].json`, for readability reasons.
     */
    pub fn save(&self) {
        let json = serde_json::to_string_pretty(self).expect(
            format!(
                "[Student {}] Could not serialize student struct.",
                self.name
            )
            .as_str(),
        );

        // Assume `users/` directory exists.
        // FIXME MINOR: Account for name changes.
        fs::write(format!("users/{}.json", self.id), json).expect(
            format!(
                "[Student {}] Could not write student file to disk.",
                self.id
            )
            .as_str(),
        );
    }

    /**
     * Loads a Student instance from a JSON string and returns it.
     */
    pub fn from_json(json: &str) -> Student {
        serde_json::from_str(json).expect("[Student] Could not parse data as valid JSON.")
    }

    /**
     * Loads a Student instance saved as JSON from disk and returns it.
     */
    pub fn load(path: &Path) -> Student {
        let json_str = fs::read_to_string(path)
            .expect(format!("[Student] Could not load file {}.", path.display()).as_str());
        Self::from_json(&json_str)
    }
}

/**
 * Retrieves a Student object given its Discord ID, if it exists in the system.
 */
pub fn get_student(id: &UserId) -> Option<Student> {
    if let Ok(json) = fs::read_to_string(format!("users/{}.json", id).as_str()) {
        Some(
            serde_json::from_str(&json).expect(
                format!(
                    "[Student] Could not parse {}'s user file as valid JSON.",
                    id
                )
                .as_str(),
            ),
        )
    } else {
        None
    }
}

macro_rules! get_existing_student {
    ($id:expr) => {
        student::get_student(&$id)
            .expect(format!("[Student] Could not find student {} in the system.", $id).as_str())
    };
}
pub(crate) use get_existing_student;

/**
 * Retrieves a Student object from a generic object that contains is Discord ID.
 */
// TODO: Move to utils?
macro_rules! get_student_from_user {
    ($user:ident) => {
        student::get_student(&$user.id).expect(
            format!(
                "[Student] Could not find student {} in the system.",
                $user.id
            )
            .as_str(),
        )
    };
}
pub(crate) use get_student_from_user;
