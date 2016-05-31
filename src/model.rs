extern crate mpd;

use std::process;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use view::*;
use config::*;

pub type SharedModel<'m> = Arc<Mutex<Model<'m>>>;

// TODO: update names once concat_idents can be used here for the function name
macro_rules! register_action(
    ($model_fun:ident) => (
        pub fn $model_fun(shared_model: &mut SharedModel)
        {
            let mut model = shared_model.lock().unwrap();
            model.$model_fun();
        }
    )
);

// Register actions for closures
register_action!(playlist_play);
register_action!(playlist_pause);
register_action!(playlist_stop);
register_action!(playlist_clear);
register_action!(playlist_previous);
register_action!(playlist_next);

pub struct Model<'m>
{
    /// MPD client.
    client: mpd::Client<TcpStream>,
    /// TUI view.
    view: &'m mut View,
}

fn start_client(config: &Config) -> Result<mpd::Client, mpd::error::Error>
{
    mpd::Client::connect(config.addr)
}

impl<'m> Model<'m>
{
    pub fn new(view: &'m mut View, config: &'m Config) -> Model<'m>
    {
        // Instantiate client.
        let res = start_client(config);
        if res.is_err() {
            println!("MPD not running. Exiting...");
            process::exit(2);
        }
        let mut client = res.unwrap();

        Model
        {
            client: client,
            view: view,
        }
    }

    pub fn init(&mut self)
    {
        self.view.init();
        self.display_now_playing();
    }

    pub fn playlist_play(&mut self)
    {
        self.client.play();
        self.view.set_debug_prompt("Playing");
    }

    pub fn playlist_pause(&mut self)
    {
        use mpd::status::State as State;

        let status = self.client.status();
        if status.is_err()
        {
            self.view.set_debug_prompt(&format!("{}", status.unwrap_err()));
            return;
        }
        let state = status.unwrap().state;

        match state
        {
            State::Play => {
                self.client.pause(true);
                self.view.set_debug_prompt("Pausing");
            }
            State::Pause => {
                self.client.pause(false);
                self.view.set_debug_prompt("Playing");
            }
            State::Stop => {
                // do nothing
            }
        }
    }

    pub fn playlist_stop(&mut self)
    {
        self.client.stop();
        self.view.set_debug_prompt("Stopping");
    }

    pub fn playlist_previous(&mut self)
    {
        self.client.prev();
        self.view.set_debug_prompt("Previous song");
    }

    pub fn playlist_next(&mut self)
    {
        self.client.next();
        self.view.set_debug_prompt("Next song");
    }

    pub fn playlist_clear(&mut self)
    {
        self.client.clear();
        self.view.set_debug_prompt("Cleared playlist");
    }

    pub fn display_now_playing(&mut self)
    {
        use mpd::status::State as State;

        let mut msg = String::from("");

        let query = self.client.currentsong();
        if query.is_ok()
        {
            let data = query.unwrap();
            if data.is_some()
            {
                let unknown_artist = "Unknown artist".to_string();
                let unknown_album = "Unknown album".to_string();

                let state = self.client.status().unwrap().state;
                let mut state_msg = String::from("");
                match state
                {
                    State::Play => {
                        state_msg = "Playing".to_string();
                    }
                    State::Pause => {
                        state_msg = "Paused".to_string();
                    }
                    State::Stop => {
                        state_msg = "Stopped".to_string();
                    }
                }

                let song = data.unwrap();
                let artist = song.tags
                                 .get(&"Artist".to_string())
                                 .unwrap_or(&unknown_artist);
                let title = song.title.unwrap_or("Unknown title".to_string());
                let album = song.tags
                                .get(&"Album".to_string())
                                .unwrap_or(&unknown_album);
                msg = format!("{}: {} - {} - {}", state_msg, artist, title, album);
            }
            else
            {
                msg = "No song playing".to_string();
            }
        }
        else
        {
            msg = "No MPD status available".to_string();
        }
        self.view.set_playing_line(&msg);
    }

    pub fn display_message(&mut self, msg: &str)
    {
        self.view.set_debug_prompt(msg);
    }

    pub fn deinit(&mut self)
    {
        self.view.exit();
    }
}
