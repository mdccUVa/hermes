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
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use serenity::all::UserId;

/// Data structure defining a team request.
///
/// Team requests are sent by students to other students in the same server, to invite them to join
/// an already existing team.
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Serialize, Deserialize, Getters, Setters)]
pub struct TeamRequest {
    #[getset(get = "pub")]
    team_id: String,
    #[getset(get = "pub")]
    sender_id: UserId,
}

/// Conversion from a (String, UserId)-tuple to a `TeamRequest` object.
impl Into<TeamRequest> for (String, UserId) {
    fn into(self) -> TeamRequest {
        TeamRequest {
            team_id: self.0,
            sender_id: self.1,
        }
    }
}

/// Conversion from a `TeamRequest` object to a (String, UserId)-tuple.
impl Into<(String, UserId)> for TeamRequest {
    fn into(self) -> (String, UserId) {
        (self.team_id, self.sender_id)
    }
}

/// Conversion from a `&TeamRequest` object to a (&String, &UserId)-tuple.
// Using from instead of into, just for fun.
// (I'd hppe this implementation was automatically derived, but it does not seem to be the case.)
impl<'a> From<&'a TeamRequest> for (&'a String, &'a UserId) {
    fn from(req: &'a TeamRequest) -> (&'a String, &'a UserId) {
        (&req.team_id, &req.sender_id)
    }
}

/// Comparison of `TeamRequest` objects.
///
/// Two `TeamRequest` objects are considered equal if they have the same `team_id`.
// The way the bot is implemented right now, there cannot be two TeamRequests with the same team ID
// and different sender.
impl PartialEq for TeamRequest {
    fn eq(&self, other: &Self) -> bool {
        self.team_id == other.team_id
    }
}
