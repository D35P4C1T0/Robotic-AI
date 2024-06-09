use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::sync::Mutex;

use lazy_static::lazy_static;
use oxagaudiotool::sound_config::OxAgSoundConfig;
use robotics_lib::event::events::Event;
use robotics_lib::world::tile::Content::Garbage;

lazy_static! {
    static ref SOUNDS_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

#[derive(Debug)]
struct MissingFilesError {
    missing_files: Vec<String>,
}

impl fmt::Display for MissingFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Missing files: {:?}", self.missing_files)
    }
}

impl Error for MissingFilesError {}

fn check_mp3_files(directory: &str, filenames: Vec<&str>) -> Result<(), Box<dyn Error>> {
    let mut missing_files = Vec::new();

    for filename in filenames {
        let mut file_path = PathBuf::from(directory);
        file_path.push(filename);

        if !file_path.exists() {
            missing_files.push(filename.to_string());
        }
    }

    if missing_files.is_empty() {
        Ok(())
    } else {
        Err(Box::new(MissingFilesError { missing_files }))
    }
}

pub(crate) fn populate_sounds() -> HashMap<Event, OxAgSoundConfig> {
    let directory_guard = SOUNDS_DIR.lock().unwrap();
    let folder_path = directory_guard
        .as_ref()
        .ok_or("Directory not set")
        .unwrap()
        .clone()
        .to_str()
        .unwrap()
        .to_string();
    // 🥴

    let mut map = HashMap::new();
    map.insert(
        Event::Terminated,
        OxAgSoundConfig::new(&format!("{}/terminated.mp3", folder_path)),
    );
    //loops around every possible quantity of content to assign the sound to all of them
    for quantity in 0..=20 {
        //sounds picking something off the ground
        map.insert(
            Event::AddedToBackpack(Garbage(0), quantity),
            OxAgSoundConfig::new(&format!("{}/get_garbage.mp3", folder_path)),
        );
        map.insert(
            Event::RemovedFromBackpack(Garbage(0), quantity),
            OxAgSoundConfig::new(&format!("{}/throw_garbage.mp3", folder_path)),
        );
    }
    map
}

fn set_sounds_directory(path: &str) {
    let mut directory = SOUNDS_DIR.lock().unwrap();
    *directory = Some(PathBuf::from(path));
}

pub(crate) fn populate_sounds_given_path(folder_path: String) {
    // check if files exist in the folder
    let files_vec = vec!["terminated.mp3", "get_garbage.mp3", "throw_garbage.mp3"];
    match check_mp3_files(&folder_path, files_vec) {
        Ok(_) => {
            // populate lazy static sounds directory
            set_sounds_directory(&folder_path);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
