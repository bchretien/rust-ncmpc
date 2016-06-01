extern crate time;
extern crate mpd;

use std::process;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use time::Duration;

use view::*;
use config::*;
use mpd::status::State as State;
use mpd::status::Status as Status;
use mpd::song::Song as Song;

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

fn get_song_info(song: &Song, tag: &String) -> String
{
    let unknown = "unknown".to_string();
    let zero = "0".to_string();
    if tag == "Title" {
        return song.clone().title.unwrap_or(unknown);
    }
    else if tag == "Time" || tag == "Duration" {
        let dur = song.clone().duration.unwrap_or(Duration::seconds(0));
        let min = dur.num_minutes();
        let sec = dur.num_seconds()%60;
        return format!("{min}:{sec:>02}", min=min, sec=sec);
    }
    else if tag == "Track" {
        let track = song.tags.get(tag).unwrap_or(&zero).to_string();
        let track_s = track.parse::<u32>().unwrap_or(0);
        return format!("{:>02}", track_s);
    }
    else // Use tags as is
    {
        return song.tags.get(tag).unwrap_or(&unknown).to_string();
    }
}

fn get_song_time(status: &Status) -> (Duration, Duration)
{
    status.time.unwrap_or((Duration::seconds(0), Duration::seconds(0)))
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
        self.display_now_playing();
    }

    pub fn playlist_play(&mut self)
    {
        self.client.play();
        self.view.set_debug_prompt("Playing");
    }

    pub fn playlist_pause(&mut self)
    {
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

    pub fn display_playlist(&mut self)
    {
        // Get playlist
        let playlist = self.client.queue().unwrap();

        let columns = [("Artist".to_string(), 20),
                       ("Track".to_string(), 2),
                       ("Title".to_string(), 40),
                       ("Album".to_string(), 40),
                       ("Time".to_string(), 5)];
        let n_cols = columns.len();
        let n_entries = playlist.len();
        let mut grid_raw = vec![String::from("a"); n_cols * n_entries];
        let mut grid_base: Vec<_> = grid_raw.as_mut_slice().chunks_mut(n_cols).collect();
        let mut grid: &mut [&mut [String]] = grid_base.as_mut_slice();

        for i in 0..n_entries {
            grid[i][0] = get_song_info(&playlist[i], &"Artist".to_string());
            grid[i][1] = get_song_info(&playlist[i], &"Track".to_string());
            grid[i][2] = get_song_info(&playlist[i], &"Title".to_string());
            grid[i][3] = get_song_info(&playlist[i], &"Album".to_string());
            grid[i][4] = get_song_info(&playlist[i], &"Time".to_string());
        }

        self.view.set_playlist(&columns, grid);
    }

    pub fn display_play_bar(&mut self)
    {
        let (e, d) = get_song_time(&self.client.status().unwrap());
        let elapsed = e.num_seconds();
        let duration = d.num_seconds();

        let mut pct: f32 = 0.;
        if duration > 0
        {
            pct = (100*elapsed/duration) as f32;
        }
        self.view.set_play_bar(pct);
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
                let artist = get_song_info(&song, &"Artist".to_string());
                let album = get_song_info(&song, &"Album".to_string());
                let title = song.title.unwrap_or("Unknown title".to_string());
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
}
