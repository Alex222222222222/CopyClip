use std::path::PathBuf;

use crate::error::Error;

/// Takes a vector of bytes and returns a hash of the image in base64
fn hash_img(img: &Vec<u8>) -> String {
    let res = format!("{:x}", md5::compute(img));
    // if length of the hash is less than 16, pad it with 0
    if res.len() < 16 {
        let pad = 16 - res.len();
        let mut res = res;
        for _ in 0..pad {
            res = format!("{}0", res);
        }
        return res;
    }

    res
}

/// Calculate the path to store the image
/// The path is calculated based on the hash of the image
/// The path is in the format of `UserDataDir/img/[first 2 chars of hash]/[next to chars for hash]/[all the other chars of hash].png`
///
/// If the exactly same image is already stored in the path, return the (path, true)
/// If the image is not stored in the path, return the (path, false)
fn get_img_path(user_data_dir: PathBuf, img: &Vec<u8>) -> (PathBuf, bool) {
    let mut path = user_data_dir;
    path.push("img");
    let hash = hash_img(img);
    path.push(&hash[0..2]);
    path.push(&hash[2..4]);
    path.push(&hash[4..]);
    let mut path_final = path.with_extension("png");

    let mut index = 1;
    while path_final.exists() {
        // test if the given img and the img in the path are the same
        let img_in_path = std::fs::read(&path_final).unwrap();
        if img_in_path == *img {
            return (path_final, true);
        }

        path.pop();
        path.push(&format!("{}-{}", &hash[4..], index));
        index += 1;
        path_final = path.with_extension("png");
    }

    (path_final, false)
}

/// Store the image in the calculated path, and return the path
pub fn store_img_return_path(user_data_dir: PathBuf, img: &Vec<u8>) -> Result<String, Error> {
    let (path, exist) = get_img_path(user_data_dir, img);
    if !exist {
        // create the directory if not exist
        let dir = match path.parent() {
            Some(dir) => dir,
            None => return Err(Error::PathError("path has no parent".to_string())),
        };
        let dir_1 = match dir.parent() {
            Some(dir) => dir,
            None => return Err(Error::PathError("path has no parent".to_string())),
        };
        let dir_2 = match dir_1.parent() {
            Some(dir) => dir,
            None => return Err(Error::PathError("path has no parent".to_string())),
        };
        if !dir_2.exists() {
            match std::fs::create_dir_all(dir_2) {
                Ok(_) => (),
                Err(err) => return Err(Error::PathError(format!("failed to create dir: {}", err))),
            }
        }
        if !dir_1.exists() {
            match std::fs::create_dir_all(dir_1) {
                Ok(_) => (),
                Err(err) => return Err(Error::PathError(format!("failed to create dir: {}", err))),
            }
        }
        if !dir.exists() {
            match std::fs::create_dir_all(dir) {
                Ok(_) => (),
                Err(err) => return Err(Error::PathError(format!("failed to create dir: {}", err))),
            }
        }
        std::fs::write(&path, img).unwrap();
    }

    Ok(path.to_string_lossy().to_string())
}

/// Get the image from the path
pub fn get_img(path: &str) -> Result<Vec<u8>, Error> {
    match std::fs::read(path) {
        Ok(img) => Ok(img),
        Err(err) => Err(Error::PathError(format!("failed to read image: {}", err))),
    }
}
