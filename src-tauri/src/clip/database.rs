use sqlite::{State, Connection};
use tauri::{AppHandle, Manager};

use super::ClipDataMutex;

pub fn init_database_connection(app: &AppHandle) -> Result<(), String> {
      // get the app data dir
      let app_data_dir = get_and_create_app_data_dir(app);
      if app_data_dir.is_err() {
            return Err(app_data_dir.err().unwrap());
      }
      let app_data_dir = app_data_dir.unwrap();

      // create the database dir if it does not exist
      let connection = get_and_create_database(app_data_dir);
      if connection.is_err() {
            return Err(connection.err().unwrap());
      }
      let connection = connection.unwrap();

      // init the version of the database
      let version = init_version_table(&connection, app);
      if version.is_err() {
            return Err(version.err().unwrap());
      }

      // init the clips table
      let clips = init_clips_table(connection,app);
      if let Err(err) = clips {
            return Err(err);
      }

      Ok(())
}

fn get_and_create_app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
      // get the app data dir
      let app_data_dir = app.path_resolver().app_data_dir();
      if app_data_dir.is_none() {
            return Err("Failed to get app data dir".to_string());
      }
      let app_data_dir = app_data_dir.unwrap();

      // if the app data dir does not exist, create it
      if app_data_dir.exists() == false {
            if let Err(_) = std::fs::create_dir_all(&app_data_dir) {
                  return Err("Failed to create app data dir".to_string());
            }
      }

      return Ok(app_data_dir);
}

fn get_and_create_database(app_data_dir: std::path::PathBuf) -> Result<Connection, String> {
      // create the database dir if it does not exist
      let database_path = app_data_dir.join("database");

      let connection = sqlite::open(database_path.as_path());
      if connection.is_err() {
            return Err("Failed to open database".to_string());
      }

      let connection = connection.unwrap();

      return Ok(connection);
}

fn get_current_version(app: &AppHandle) -> Option<String>{
      let app_version = app.config().package.version.clone();
      if app_version.is_none() {
            return None;
      }
      let app_version = app_version.unwrap();

      return Some(app_version.to_string());
}

fn get_save_version(connection: &Connection) -> Result<String, String> {
      // get the latest version, if it exists
      // if not exists, it is the first time the app is launched return 0.0.0

      let mut statement = connection.prepare("SELECT version FROM version ORDER BY id DESC LIMIT 1").unwrap();
      let state = statement.next();
      if let Ok(State::Row) = state {
            let version = statement.read::<String,_>("version");
            if version.is_err() {
                  return Err("Failed to get version".to_string());
            }
            let version = version.unwrap();

            return Ok(version);
      } else if state.is_err() {
            return Err("Failed to get version: ".to_string() + &state.err().unwrap().message.unwrap().to_string());
      }

      if let Ok(State::Done) = state {
            return Ok("0.0.0".to_string()); // if there is no version, it is the first time the app is launched
      }

      return Err("Failed to get version".to_string());
}

fn init_version_table(connection: &Connection, app: &AppHandle) -> Result<(), String> {
      // create the version table if it does not exist
      let mut statement = connection.prepare("CREATE TABLE IF NOT EXISTS version (id INTEGER PRIMARY KEY AUTOINCREMENT, version TEXT)").unwrap();
      let state = statement.next();
      if let Ok(State::Done) = state {
            drop(statement);
            // try get the save version
            let save_version = get_save_version(connection);
            if save_version.is_err() {
                  return Err(save_version.err().unwrap());
            }
            let save_version = save_version.unwrap();
            if save_version == "0.0.0".to_string() {
                  // if it is the first time the app is launched, insert the current version
                  let current_version = get_current_version(app);
                  if current_version.is_none() {
                        // TODO: handle error
                        panic!("Failed to get current version")
                  }
                  let current_version = current_version.unwrap();

                  let mut statement = connection.prepare("INSERT INTO version (version) VALUES (?)").unwrap();
                  let res = statement.bind((1, current_version.as_str()));
                  if res.is_err() {
                        return Err("Failed to insert version: ".to_string() + &res.err().unwrap().message.unwrap().to_string());
                  }

                  let state = statement.next();
                  if let Ok(State::Done) = state {
                        return Ok(());
                  } else if state.is_err() {
                        return Err("Failed to insert version: ".to_string() + &state.err().unwrap().message.unwrap().to_string());
                  }

                  return Err("Failed to insert version".to_string());
            } else {
                  // if it is not the first time the app is launched, check if the save version is the same as the current version
                  let current_version = get_current_version(app);
                  if current_version.is_none() {
                        panic!("Failed to get current version")
                  }
                  let current_version = current_version.unwrap();

                  if save_version != current_version {
                        // if the save version is not the same as the current version, deal with the backward comparability
                        let backward_comparability = backward_comparability(app, save_version);
                        if backward_comparability.is_err() {
                              return Err(backward_comparability.err().unwrap());
                        }

                        // update the version
                        let mut statement = connection.prepare("INSERT INTO version (version) VALUES (?)").unwrap();
                        let res = statement.bind((1, current_version.as_str()));
                        if res.is_err() {
                              return Err("Failed to update version: ".to_string() + &res.err().unwrap().message.unwrap().to_string());
                        }
                        let state = statement.next();
                        if let Ok(State::Done) = state {
                              return Ok(());
                        } else if state.is_err() {
                              return Err("Failed to update version: ".to_string() + &state.err().unwrap().message.unwrap().to_string());
                        }

                        return Err("Failed to update version".to_string());
                  }
            }

            return Ok(());
      } else if state.is_err() {
            return Err("Failed to create version table: ".to_string() + &state.err().unwrap().message.unwrap().to_string());
      }

      return Err("Failed to create version table".to_string());
}

fn backward_comparability(_app: &AppHandle,_save_version: String) -> Result<(), String> {
      // deal with the backward comparability based on the save version

      Ok(())
}

fn init_clips_table(connection: Connection, app: &AppHandle) -> Result<(), String> {
      // create the clips table if it does not exist
      let mut statement = connection.prepare("CREATE TABLE IF NOT EXISTS clips (id INTEGER PRIMARY KEY AUTOINCREMENT, text TEXT, timestamp INTEGER, favorite INTEGER)").unwrap();
      let state = statement.next();
      if let Ok(State::Done) = state {
            let clip_data_mutex = app.state::<ClipDataMutex>();
            let mut clip_data = clip_data_mutex.clip_data.lock().unwrap();
            drop(statement);
            clip_data.database_connection = Some(connection);

            // get the whole clips ids
            let mut ids = Vec::new();
            let mut statement = clip_data.database_connection.as_ref().unwrap().prepare("SELECT id FROM clips").unwrap();
            while let Ok(State::Row) = statement.next() {
                  let id = statement.read::<i64,_>("id");
                  if id.is_err() {
                        return Err("Failed to get id of clip".to_string());
                  }
                  let id = id.unwrap();

                  ids.push(id);
            }
            drop(statement);
            clip_data.clips.whole_list_of_ids = ids;

            return Ok(());
      } else if state.is_err() {
            return Err("Failed to create clips table: ".to_string() + &state.err().unwrap().message.unwrap().to_string());
      }

      return Err("Failed to create clips table".to_string());
}