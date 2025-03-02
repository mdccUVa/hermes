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
mod commands;
mod student;
mod team;
mod teamrequest;
mod utils;

use crate::utils::BotConfig;
use getset::Getters;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, fs};

/* General data structures: */

/**
 * Tablón credentials data structure.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct Credentials {
    #[getset(get = "pub")]
    team: String,
    #[getset(get = "pub")]
    password: Option<String>,
}

/* Poise-required data types: */

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data:
pub struct Data {}

async fn ready(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        // Ready (bot is started):
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            match data_about_bot.user.discriminator {
                Some(discriminator) => {
                    println!(
                        "{}#{discriminator:#?} is connected.",
                        data_about_bot.user.name
                    )
                }
                None => println!("{} is connected.", data_about_bot.user.name),
            }

            // Create directories for the persistent data, if necessary:
            utils::init_filesystem();

            // Load "global" data structures:
            let mut guild_map = utils::load_guildmap();
            let mut user_map = utils::load_usermap();

            ctx.set_presence(None, serenity::OnlineStatus::Online);

            // Check guilds and update related information:
            for g in &data_about_bot.guilds {
                let gid = g.id;
                let gname = gid.name(&ctx.cache).expect(
                    format!("Unable to retrieve the name of the guild with id {}.", gid).as_str(),
                );
                println!("Hermes entered the guild {} ({}).", gname, gid);

                // Create the guild's directory if it doesn't exist:
                if fs::metadata(format!("guilds/{}", gid)).is_err() {
                    fs::create_dir(format!("guilds/{}", gid))
                        .expect(format!("Could not create guilds/{} directory.", gid).as_str());
                }

                // Create the guild's request log, if it doesn't exist:
                if fs::metadata(format!("guilds/{}/requests.log", gid)).is_err() {
                    fs::write(format!("guilds/{}/requests.log", gid).as_str(), "")
                        .expect(format!("Could not create guilds/{}/requests.log.", gid).as_str());
                }

                // Check if the configuration file exists, and create it if it doesn't:
                if fs::metadata(format!("guilds/{}/config.json", gid)).is_err() {
                    // Use custom default configuration, if found:
                    if fs::metadata("config.json").is_ok() {
                        let config: BotConfig = serde_json::from_str(
                            fs::read_to_string("config.json")
                                .expect("Could not read the default configuration file.")
                                .as_str(),
                        )
                        .expect(
                            "Could not parse the default configuration file as a BotConfig object.",
                        );
                        utils::update_config_persistence(&config, &gid);
                    } else {
                        // Use the following default configuration as last resort:
                        let config = BotConfig {
                            tablon_url: String::from("https://frontendv.infor.uva.es"),
                            team_capacity: 2,
                            team_prefix: String::from("g"),
                            bot_channel: String::from("bot-commands"),
                            lb_channel: String::from("leaderboards"),
                            notify_leaders: true,
                            leader_count: 5,
                            public_notify: true,
                            bot_news_channel: String::from("bot-news"),
                            column_separator: String::from(" | "),
                        };
                        utils::update_config_persistence(&config, &gid);
                    }
                }

                // Create the guild's team name map, if it doesn't exist:
                if !fs::metadata(format!("guilds/{}/nameMap.json", gid)).is_ok() {
                    let json = serde_json::to_string(&HashMap::<String, String>::new()).expect(
                        format!(
                            "Could not serialize an initial empty name map into JSON for guild {}.",
                            gid
                        )
                        .as_str(),
                    );
                    fs::write(format!("guilds/{}/nameMap.json", gid).as_str(), json)
                        .expect(format!("Could not write guilds/{}/nameMap.json.", gid).as_str());
                }

                // Create the guild's team directory, if it doesn't exist:
                if !fs::metadata(format!("guilds/{}/teams", gid)).is_ok() {
                    fs::create_dir(format!("guilds/{}/teams", gid)).expect(
                        format!("Could not create guilds/{}/teams directory.", gid).as_str(),
                    );
                }

                // New server found? Add to database:
                let std_name = utils::sanitize_name(&gname);
                if !guild_map.contains_key(&std_name) {
                    guild_map.insert(std_name, gid);
                    utils::update_guildmap_persistence(&guild_map);
                }

                // Create or update the student's objects on the database:
                for member in gid.members(&ctx.http, None, None).await.expect(
                    format!("Could not retrieve the members of the guild {}.", gid).as_str(),
                ) {
                    // Ignore bots:
                    if member.user.bot {
                        continue;
                    }

                    let uid = member.user.id;
                    let name = member.user.name;

                    // Create new students:
                    if student::get_student(&uid).is_none() {
                        let _student = student::Student::new(uid, name.clone());
                    }

                    // Add to the user map (if not present):
                    if user_map.insert(name.clone(), uid).is_none() {
                        utils::update_usermap_persistence(&user_map);
                    }

                    // TODO: add students that joined the server after the bot was added to the
                    // system.
                    // TODO: this should probably account for user name changes, too.
                }
            }
        }
        // Guild create (the bot joins a new server):
        serenity::FullEvent::GuildCreate { guild, is_new } => {
            // Only process new guilds:
            if *is_new != Some(true) {
                return Ok(());
            }
            println!("Hermes entered the guild {} ({}).", guild.name, guild.id);

            // Basically, process the guild as they are in the Ready event...

            // Load "global" data structures:
            let mut guild_map = utils::load_guildmap();
            let mut user_map = utils::load_usermap();

            let gid = guild.id;
            let gname = gid.name(&ctx.cache).expect(
                format!("Unable to retrieve the name of the guild with id {}.", gid).as_str(),
            );
            println!("Hermes entered the guild {} ({}).", gname, gid);

            // Create the guild's directory:
            fs::create_dir(format!("guilds/{}", gid))
                .expect(format!("Could not create guilds/{} directory.", gid).as_str());

            // Create the guild's request log:
            fs::write(format!("guilds/{}/requests.log", gid).as_str(), "")
                .expect(format!("Could not create guilds/{}/requests.log.", gid).as_str());

            // Check if the configuration file exists, and create it if it doesn't:
            if fs::metadata(format!("guilds/{}/config.json", gid)).is_err() {
                // Use custom default configuration, if found:
                if fs::metadata("config.json").is_ok() {
                    let config: BotConfig = serde_json::from_str(
                        fs::read_to_string("config.json")
                            .expect("Could not read the default configuration file.")
                            .as_str(),
                    )
                    .expect(
                        "Could not parse the default configuration file as a BotConfig object.",
                    );
                    utils::update_config_persistence(&config, &gid);
                } else {
                    // Use the following default configuration as last resort:
                    let config = BotConfig {
                        tablon_url: String::from("https://frontendv.infor.uva.es"),
                        team_capacity: 2,
                        team_prefix: String::from("g"),
                        bot_channel: String::from("bot-commands"),
                        lb_channel: String::from("leaderboards"),
                        notify_leaders: true,
                        leader_count: 5,
                        public_notify: true,
                        bot_news_channel: String::from("bot-news"),
                        column_separator: String::from(" | "),
                    };
                    utils::update_config_persistence(&config, &gid);
                }
            }

            // Create the guild's team name map, if it doesn't exist:
            if !fs::metadata(format!("guilds/{}/nameMap.json", gid)).is_ok() {
                let json = serde_json::to_string(&HashMap::<String, String>::new()).expect(
                    format!(
                        "Could not serialize an initial empty name map into JSON for guild {}.",
                        gid
                    )
                    .as_str(),
                );
                fs::write(format!("guilds/{}/nameMap.json", gid).as_str(), json)
                    .expect(format!("Could not write guilds/{}/nameMap.json.", gid).as_str());
            }

            // Create the guild's team directory, if it doesn't exist:
            fs::create_dir(format!("guilds/{}/teams", gid))
                .expect(format!("Could not create guilds/{}/teams directory.", gid).as_str());

            // Add guild to database:
            let std_name = utils::sanitize_name(&gname);
            guild_map.insert(std_name, gid);
            utils::update_guildmap_persistence(&guild_map);

            // Create or update the student's objects on the database:
            for member in gid
                .members(&ctx.http, None, None)
                .await
                .expect(format!("Could not retrieve the members of the guild {}.", gid).as_str())
            {
                // Ignore bots:
                if member.user.bot {
                    continue;
                }

                let uid = member.user.id;
                let name = member.user.name;

                // Create new students:
                if student::get_student(&uid).is_none() {
                    let _student = student::Student::new(uid, name.clone());
                }

                // Add to the user map (if not present):
                if user_map.insert(name.clone(), uid).is_none() {
                    utils::update_usermap_persistence(&user_map);
                }

                // TODO: add students that joined the server after the bot was added to the
                // system.
                // TODO: this should probably account for user name changes, too.
            }
        }

        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Discord token not provided (in DISCORD_TOKEN environmental variable).");
    let intents = serenity::GatewayIntents::default()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::botconfig::botconfig(),
                commands::history::history(),
                commands::passwords::passwords(),
                commands::request::request(),
                commands::settings::settings(),
                commands::team::team(),
                commands::teamdump::teamdump(),
                commands::teamedit::teamedit(),
            ],
            event_handler: |ctx, event, framwework, data| {
                Box::pin(ready(ctx, event, framwework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .expect("Could not register the commands.");
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework) // For command handling, using poise.
        .await
        .expect("Could not create the Discord bot client object.");

    client.start().await.expect("The Discord bot crashed.");
}
