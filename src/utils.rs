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
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::collections::HashMap;
use std::fs;

/* Data structures: */

// Bot configuration struct:
/**
 * Data structure encapsulating the per-guild configuration of the bot.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Deserialize, Serialize)]
pub struct BotConfig {
    /// The URL for this guild's Tablón endpoint:
    pub tablon_url: String,
    /// The number of members a team of students must have, exactly.
    pub team_capacity: u8,
    /// The prefix for the teams' identifiers (e.g. "g" for "g110").
    pub team_prefix: String,
    /// The name of the guild's (private) channel dedicated for special bot admin commands and
    /// activity monitoring.
    pub bot_channel: String,
    /// The name of the guild's public channel dedicated to leaderboard visualizations.
    pub lb_channel: String,
    /// Whether to notify the top teams on leaderboards of when their position changes.
    pub notify_leaders: bool,
    /// Amount of top teams susceptible of being notified of position changes (see
    /// `notify_leaders`).
    pub leader_count: u8,
    /// Whether to post the leaderboard notifications in a public channel in the guild, or just
    /// privately.
    pub public_notify: bool,
    /// The name of the guild's public channel where news and notifications (e.g. position updates)
    /// should be sent, if any.
    pub bot_news_channel: String,
    /// The field separator for multi-field columns in leaderboard visualizations. This is used
    /// when visualizing more than 3 fields of a leaderboard: the remaining fields will be grouped
    /// in the last column, separated by this.
    pub column_separator: String,
}

/**
 * Macro for logging to stderr the usage of a command.
 */
macro_rules! elog_cmd {
    ($ctx:ident) => {
        eprintln!(
            "Executing command `{}`, triggered by <@{}> ({}).",
            $ctx.invocation_string(),
            $ctx.author().id,
            $ctx.author().tag()
        );
    };
}
pub(crate) use elog_cmd;

/**
 * Macro for retrieving the guild ID from a Context object.
 */
macro_rules! get_guild_id {
    ($ctx:ident) => {
        $ctx.guild_id()
            .expect("The command was not executed in a guild.")
    };
}
pub(crate) use get_guild_id;

/**
 * Macro for retrieving the student object from the author of a command.
 */
macro_rules! get_triggering_student {
    ($ctx:ident) => {
        student::get_student(&$ctx.author().id)
            .expect(format!("Student {} not found in the system.", $ctx.author().id,).as_str())
    };
}
pub(crate) use get_triggering_student;

/**
 * Loads the bot configuration for a guild from its persistent configuration file.
 * If the configuration file does not exist, it is created with default values.
 */
pub fn load_config(guild_id: &GuildId) -> BotConfig {
    let json = fs::read_to_string(format!("guilds/{}/config.json", guild_id))
        .expect(format!("Could not read guild {}'s configuration file.", guild_id).as_str());
    serde_json::from_str(&json).expect(
        format!(
            "Could not parse guild {}'s configuration as valid JSON.",
            guild_id
        )
        .as_str(),
    )
}

/**
 * Creates the directories and files expected for the bot to function properly.
 */
pub fn init_filesystem() {
    fs::create_dir_all("guilds").expect("Could not create guilds directory.");
    fs::create_dir_all("users").expect("Could not create users directory.");
    if !fs::exists("guilds/guildMap.json")
        .expect("Could not check existence of guilds/guildMap.json")
    {
        let json = serde_json::to_string_pretty(&HashMap::<String, GuildId>::new())
            .expect("Could not serialize the initial empty guild map into JSON.");
        fs::write("guilds/guildMap.json", json).expect("Could not create guilds/guildMap.json");
    }
    if !fs::exists("users/userMap.json").expect("Could not check existence of users/userMap.json") {
        let json = serde_json::to_string_pretty(&HashMap::<String, UserId>::new())
            .expect("Could not serialize the initial empty user map into JSON.");
        fs::write("users/userMap.json", json).expect("Could not create users/userMap.json");
    }
}

/**
 * Updates the persistent configuration file for a guild.
 * It is assumed that the config file exists on disk, since it should have been loaded with
 * `load_config` beforehand.
 */
pub fn update_config_persistence(config: &BotConfig, guild_id: &GuildId) {
    let json = serde_json::to_string_pretty(config).expect(
        format!(
            "Could not serialize guild {}'s configuration into JSON.",
            guild_id
        )
        .as_str(),
    );
    fs::write(format!("guilds/{}/config.json", guild_id), json)
        .expect(format!("Could not write guild {}'s configuration file.", guild_id).as_str());
}

/**
 * Loads the persistent guildMap.json file into a HashMap object.
 */
pub fn load_guildmap() -> HashMap<String, GuildId> {
    let json =
        fs::read_to_string("guilds/guildMap.json").expect("Could not read guilds/guildMap.json");
    serde_json::from_str(&json).expect("Could not parse guilds/guildMap.json as valid JSON data.")
}

/**
 * Updates the persistent guildMap.json file, which maps Guild names into their IDs.
 */
pub fn update_guildmap_persistence(guild_map: &HashMap<String, GuildId>) {
    let json = serde_json::to_string_pretty(guild_map)
        .expect("Could not serialize the guild map into JSON.");
    fs::write("guilds/guildMap.json", json).expect("Could not write guilds/guildMap.json.");
}

/**
 * Loads the persistent userMap.json file into a HashMap object.
 */
pub fn load_usermap() -> HashMap<String, UserId> {
    let json = fs::read_to_string("users/userMap.json").expect("Could not read users/userMap.json");
    serde_json::from_str(&json).expect("Could not parse users/userMap.json as valid JSON data.")
}

/**
 * Updates the persistent userMap.json file, which maps User names into their IDs.
 */
pub fn update_usermap_persistence(user_map: &HashMap<String, UserId>) {
    let json = serde_json::to_string_pretty(user_map)
        .expect("Could not serialize the user map into JSON.");
    fs::write("users/userMap.json", json).expect("Could not write users/userMap.json.");
}

/**
 * Load the name map for a specific guild.
 * If the file does not exist, it is created with an empty map.
 *
 * The name map maps the name of a team to its ID.
 */
pub fn load_namemap(guild_id: &GuildId) -> HashMap<String, String> {
    let json = fs::read_to_string(format!("guilds/{}/nameMap.json", guild_id).as_str())
        .expect(format!("Could not read name map for server {}.", guild_id).as_str());
    serde_json::from_str(&json).expect(
        format!(
            "Could not parse guilds/{}/nameMap.json as valid JSON data.",
            guild_id
        )
        .as_str(),
    )
}

/**
 * Updates the persistent nameMap.json file for a specific guild, which maps team names into their
 * IDs.
 */
pub fn update_namemap_persistence(name_map: &HashMap<String, String>, guild_id: &GuildId) {
    let json = serde_json::to_string_pretty(name_map).expect(
        format!(
            "Could not serialize the name map for server {} into JSON.",
            guild_id
        )
        .as_str(),
    );
    fs::write(format!("guilds/{}/nameMap.json", guild_id).as_str(), json)
        .expect(format!("Could not write guilds/{}/nameMap.json.", guild_id).as_str());
}

/**
 * Transform a guild's name into a custom safe guild name.
 *
 * This basically substitutes all spaces with underscores, and slashes with hyphens.
 *
 * This is done so a path containing the guild's name can be created without causing any issues.
 */
pub fn sanitize_name(name: &String) -> String {
    name.replace(" ", "_").replace("/", "-")
}
