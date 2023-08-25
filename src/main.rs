#![allow(dead_code)]

// Todo:
// * Playlist typing in SongInfo
// * Maybe album typing?
// * Extra stuff in the `SongInfo` struct, like if its downloaded, year and stuff like that
//! * Implement a function in PlayerControls that lets you get *JUST* the duration and/or progress of the current song 

mod itunes_interface {
    use powershell_script::{PsScriptBuilder, PsScript};

    // are these lines even? no
    // do they look cool? yes
    //#region -----------------         TYPINGS       --------------------------

    pub struct SongInfo {
        /// Name of the currently plalbuaying song
        pub name: String,
        /// Length (in seconds) of the currently playing song
        pub duration: u16,
        /// Formated (M:S) version of the `duration` property in a String format
        pub time: String,
        /// Progress (in seconds) of the currently playing song
        pub progress: u8,
        /// Formatted (M:S) version of the `progress` property in a String format
        pub formatted_progress: String,
        /// Album Name of the currently playing song
        pub album: String,
        /// Name of the Artist(s) that created the currently playing song
        pub artist: String,
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

        pub fn get_song_info(&self) -> SongInfo {
            if !self.itunes.is_song_ready() {
                return SongInfo {
                    name: String::new(),
                    album: String::new(),
                    artist: String::new(),
                    duration: 0,
                    time: String::from("0:0"),
                    progress: 0,
                    formatted_progress: String::from("0:0"),
                }
            }
            
            let progress = self.itunes.get_property("PlayerPosition").trim().parse::<u8>().unwrap();

            return SongInfo { 
                name: self.itunes.get_property("CurrentTrack.Name"),
                album: self.itunes.get_property("CurrentTrack.Album"),
                artist: self.itunes.get_property("CurrentTrack.Artist"),
                duration: self.itunes.get_property("CurrentTrack.Duration").trim().parse::<u16>().unwrap(),
                time: self.itunes.get_property("CurrentTrack.Time"),
                progress,
                formatted_progress: self.format_progress(progress)
            };
        }

        pub fn format_progress(&self, progress: u8) -> String {
            let mut x = progress/60;
            // remove da decimal
            x = format!("{:.0}", x).parse().unwrap();
            
            let y = progress - x * 60;

            format!("{}:{}", x,y)
        }
    }

    //#endregion   

    //#region ----------------- PLAYER CONTROLS STRUCT -------------------------

    pub struct ItunesPlayerControls<'t> {
        pub(crate) itunes: &'t Itunes<'t>,
        pub(crate) muted: bool,
        pub(crate) song_interface: SongInterface<'t>,
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
                song_interface: SongInterface { itunes: &itunes }
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

        /// Checks if the currently ready song is playing 
        pub fn is_playing(&self) -> bool {
            match self.itunes.get_property("PlayerState").trim() {
                "1" => true,
                _ => false,
            } 
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
        
        pub fn is_song_ready(&self) -> bool {
            let current_track = self.get_property("CurrentTrack");

            if current_track == "" {
                return false
            }

            return true
        }
    }
    
    //#endregion

}

// a little example

fn main() {
    let itunes = itunes_interface::Itunes::new();
    let player_controls = itunes_interface::ItunesPlayerControls::new(&itunes);

    if !itunes.is_song_ready() {
        println!("You're not listening to a song!");
        return;
    }

    let song_info = player_controls.song_interface.get_song_info();

    println!("\n\nYou're listening to {} by {}", song_info.name, song_info.artist);
    println!("{} -- {}", song_info.formatted_progress, song_info.time);
}
