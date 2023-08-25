#![allow(dead_code)]

use std::io;

// Todo:
// * Playlist typing in SongInfo
// * Maybe album typing?
// * Extra stuff in the `SongInfo` struct, like if its downloaded, year and stuff like that

mod itunes_interface {
    use powershell_script::{PsScriptBuilder, PsScript};

    // are these lines even? no
    // do they look cool? yes
    //#region -----------------         TYPINGS       --------------------------

    pub struct SongInfo {
        /// Name of the currently playing song
        name: String,
        /// Length (in seconds) of the currently playing song
        duration: u8,
        /// Formated version of the `duration` property in a String format.
        time: String,
        /// Progress (in seconds) of the currently playing song
        progress: u8,
        /// Album Name of the currently playing song
        album: String,
        /// Name of the Artist(s) that created the currently playing song
        artist: String,
    }

    //#endregion

    //#region ----------------- SONG INTERFACE STRUCT --------------------------

    pub struct SongInterface<'t> {
        itunes: &'t Itunes<'t>
    }

    impl<'t> SongInterface<'t> {
        pub fn new(itunes: &'t Itunes<'t>) -> SongInterface<'t> {
            SongInterface { itunes }
        }

        pub fn get_song_info(&self ) -> SongInfo {
            if self.itunes.is_playing_song() {
                return SongInfo {
                    name: String::new(),
                    album: String::new(),
                    artist: String::new(),
                    duration: 0,
                    time: String::new(),
                    progress: 0
                }
            }
            
            return SongInfo { 
                name: self.itunes.get_property("CurrentTrack.Name"),
                album: self.itunes.get_property("CurrentTrack.Album"),
                artist: self.itunes.get_property("CurrentTrack.Artist"),
                duration: self.itunes.get_property("CurrentTrack.Duration").parse::<u8>().unwrap(),
                time: self.itunes.get_property("CurrentTrack.Time"),
                progress: self.itunes.get_property("PlayerPosition").parse::<u8>().unwrap()
            };
        }
    }

    //#endregion   

    //#region ----------------- PLAYER CONTROLS STRUCT -------------------------

    pub struct ItunesPlayerControls<'t> {
        pub(crate) itunes: &'t Itunes<'t>,
        pub(crate) muted: bool,
    }

    impl<'t> ItunesPlayerControls<'t> {
        pub fn new(itunes: &'t Itunes<'t>) -> ItunesPlayerControls<'t> {
            ItunesPlayerControls {
                itunes,
                muted: (|| -> bool {
                    if itunes.get_property("Mute").as_str() == "True" {
                        return true;
                    }

                    return false;
                })(),
            }
        }
    
        pub fn get_volume(&self) -> u8 {
            self.itunes.get_property("SoundVolume").trim().parse::<u8>().unwrap()
        }
    
        pub fn increase_volume(&self, increase_by: u8) {
            self.itunes.property(format!("SoundVolume = {}", self.get_volume() + increase_by).as_str())
        }
    
        pub fn set_volume(&self, volume: u8) {
            self.itunes.property(format!("SoundVolume = {}", volume).as_str())
        }
    
        pub fn pause(&self) {
            self.itunes.property("pause()")
        }
    
        pub fn play(&self) {
            self.itunes.property("play()")
        }

        pub fn next_track(&self) {
            self.itunes.property("NextTrack()")
        }

        pub fn previous_track(&self) {
            self.itunes.property("PreviousTrack()")
        }

        pub fn toggle_mute(&self) {
            if self.muted {
                self.itunes.property("Mute = $False");
                return;
            }

            self.itunes.property("Mute = $True");
        }
    }
    
    //#endregion

    //#region  ------------------- MAIN ITUNES STRUCT ----------------------------

    pub struct Itunes<'a> {
        power_shell: PsScript,
        itunes_echo_script: &'a str,
        itunes_script: &'a str,
    }
    
    impl Itunes<'_> {
        pub fn new() -> Itunes<'static> {
            Itunes {
                power_shell: PsScriptBuilder::new().non_interactive(true).hidden(true).no_profile(true).print_commands(false).build(),
                itunes_echo_script: include_str!("./Powershell/itunesEcho.ps1"),
                itunes_script: include_str!("./Powershell/itunes.ps1"),
            }
        }
    
        fn compile_script(&self, prop: &str, echo: bool) -> String {
            if echo {
                return self.itunes_echo_script.replace("[INPUT]", prop);
            }
    
            return self.itunes_script.replace("[INPUT]", prop)
        }
    
        pub fn get_property(&self, prop: &str) -> String {
            let property = self.power_shell.run(&self.compile_script(prop, true)).unwrap().stdout();
    
            if property == None {
                return String::new();
            }
    
            return property.unwrap();
        }
    
        pub fn property(&self, prop: &str) {
            self.power_shell.run(&self.compile_script(prop, false)).expect("Property To Execute");
        }
        
        pub fn is_playing_song(&self) -> bool {
            let current_track = self.get_property("CurrentTrack");

            if current_track == "" {
                return false
            }

            return true
        }
    }
    
    //#endregion

}

fn main() {
    let itunes = itunes_interface::Itunes::new();
    let player_controls = itunes_interface::ItunesPlayerControls::new(&itunes);

    println!("Is Song Ready: {}", itunes.is_playing_song());
    println!("Detected Volume: {}", player_controls.get_volume());

    player_controls.increase_volume(25);

    println!("Volume Increased By 25, Volume Is Now: {}", player_controls.get_volume());

    loop {
        let mut input = String::from("");

        io::stdin()
        .read_line(&mut input)
        .expect("Error reading input!");

        match input.trim() {
            "play" => player_controls.play(),
            _ => player_controls.pause()
        }
    }
}
