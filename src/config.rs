use std::net::SocketAddr;
use std::env;

#[derive(Clone,Copy)]
pub enum ControlKey
{
    KeyCode(i32),
    Char(char),
}

#[derive(Clone,Copy)]
pub struct KeyConfig
{
    play_pause: ControlKey,
    quit: ControlKey,
}

#[derive(Clone,Copy)]
pub struct ColorConfig
{
    bg: i16,
    fg: i16,
}

#[derive(Clone,Copy)]
pub struct Config
{
    pub addr: SocketAddr,
    pub colors: ColorConfig,
    pub keys: KeyConfig,
}

impl KeyConfig
{
    pub fn new() -> KeyConfig
    {
        KeyConfig
        {
            play_pause: ControlKey::Char('p'),
            quit: ControlKey::Char('q'),
        }
    }
}

impl ColorConfig
{
    pub fn new() -> ColorConfig
    {
        ColorConfig
        {
            bg: 0,
            fg: 0,
        }
    }
}

impl Config
{
    pub fn new() -> Config
    {
        // TODO: support MPD_SOCK
        let addr = env::var("MPD_SOCK").unwrap_or("127.0.0.1:6600".to_owned());

        // Search for the MPD_PORT environment variable
        let mpd_ip = "127.0.0.1".parse().unwrap();
        let mpd_port = env::var("MPD_PORT")
            .unwrap_or("6600".to_owned())
            .parse::<u16>().unwrap_or(6600);
        println!("MPD: {}:{}", mpd_ip, mpd_port);

        Config
        {
            colors: ColorConfig::new(),
            addr: SocketAddr::new(mpd_ip, mpd_port),
            keys: KeyConfig::new(),
        }
    }
}
