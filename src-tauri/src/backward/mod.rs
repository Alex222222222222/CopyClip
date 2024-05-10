use log::debug;
use rusqlite::Connection;
use tauri::AppHandle;

use crate::{backward, error::Error};

mod v0_2_x_to_0_3_0_database;
mod v0_3_7_to_0_3_8_database;
mod v0_3_8_to_0_3_9_database;
mod v0_3_x_to_0_3_3_config;
mod v0_3_x_to_0_3_5_database;
mod v0_3_x_to_0_3_7_database;

/// parse the version number string to i32
fn parse_version_number(version: &str) -> Result<(i32, i32, i32), Error> {
    let version = version.split('.').collect::<Vec<&str>>();
    if version.len() != 3 {
        return Err(Error::GetVersionFromDatabaseErr(
            "The version number is not correct".to_string(),
        ));
    }
    // a version number is like [major].[minor].[patch]
    let major = version[0].parse::<i32>();
    let minor = version[1].parse::<i32>();
    let patch = version[2].parse::<i32>();

    if major.is_err() || minor.is_err() || patch.is_err() {
        return Err(Error::GetVersionFromDatabaseErr(
            "The version number is not correct".to_string(),
        ));
    }

    Ok((major.unwrap(), minor.unwrap(), patch.unwrap()))
}

/// deal with the backward comparability based on the save version
///
/// return the current version
#[warn(unused_must_use)]
pub fn backward_comparability(
    app: &AppHandle,
    connection: &Connection,
    save_version: String,
) -> Result<String, Error> {
    debug!("start to deal with the backward comparability");

    // get the three version number from save_version
    let (major, mut minor, mut patch) = parse_version_number(&save_version)?;

    // deal with the backward comparability

    // if the major version is 0, the minor version is smaller than 3, need to upgrade the database to 0.3.0
    if major == 0 && minor < 3 {
        // upgrade the database to 0.3.0
        v0_2_x_to_0_3_0_database::upgrade(connection)?;
        minor = 3;
        patch = 0;
    }

    // if the major version is 0, the minor version is 3, the patch version is smaller than 3, need to upgrade the config file to 0.3.3
    // before 0.3.3, there is search_clip_per_page in the config file, after 0.3.3, this entry changed to search_clip_per_batch
    if major == 0 && minor == 3 && patch < 3 {
        // upgrade the config file to 0.3.3
        let res = v0_3_x_to_0_3_3_config::upgrade(app);
        res?;
        patch = 3;
    }

    // when moving from 0.3.3, 0.3.4, to 0.3.5,
    // I rename the column "favorite" to favourite in clips table, so need to do a sql ALTER command
    if major == 0 && minor == 3 && patch < 5 {
        v0_3_x_to_0_3_5_database::upgrade(connection)?;
        patch = 5;
    }

    // when moving fom 0.3.5, 0.3.6, to 0.3.7,
    // we need to update the pinned clips table
    if major == 0 && minor == 3 && patch < 7 {
        v0_3_x_to_0_3_7_database::upgrade(connection)?;
        patch = 7;
    }

    debug!("current version: {}.{}.{}", major, minor, patch);

    // when moving from 0.3.7 to 0.3.8,
    // we need to update the pinned clips table
    if major == 0 && minor == 3 && patch < 8 {
        backward::v0_3_7_to_0_3_8_database::upgrade(connection)?;
        patch = 8;
    }

    // when moving from 0.3.8 to 0.3.9,
    // we need to update the clips table and pinned clips table and favourite clips table
    if major == 0 && minor == 3 && patch < 9 {
        v0_3_8_to_0_3_9_database::upgrade(connection)?;
        patch = 9;
    }

    Ok(format!("{}.{}.{}", major, minor, patch))
}
