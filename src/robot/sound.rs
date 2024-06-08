use std::collections::HashMap;

use oxagaudiotool::sound_config::OxAgSoundConfig;
use robotics_lib::event::events::Event;
use robotics_lib::world::tile::Content::Garbage;

pub(crate) fn populate_sound() -> HashMap<Event, OxAgSoundConfig> {
    let mut map = HashMap::new();
    map.insert(
        Event::Terminated,
        OxAgSoundConfig::new("./sounds/terminated.mp3"),
    );
    //loops around every possible quantity of content to assign the sound to all of them
    for quantity in 0..=20 {
        //sounds picking something off the ground
        map.insert(
            Event::AddedToBackpack(Garbage(0), quantity),
            OxAgSoundConfig::new("./sounds/get_garbage.mp3"),
        );
        map.insert(
            Event::RemovedFromBackpack(Garbage(0), quantity),
            OxAgSoundConfig::new("./sounds/throw_garbage.mp3"),
        );
    }
    map
}
