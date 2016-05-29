extern crate mpd;

use std::process;
use std::net::TcpStream;

use view::*;
use config::*;

pub struct Model<'m>
{
    client: mpd::Client<TcpStream>,
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
    }

    pub fn playlist_play(&mut self) {
        self.client.play();
        self.view.playlist_play();
    }

    pub fn playlist_stop(&mut self) {
        self.client.stop();
        self.view.playlist_stop();
    }

    pub fn deinit(&mut self)
    {
        self.view.exit();
    }
}
