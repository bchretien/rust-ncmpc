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
    pub clear: ControlKey,
    pub next_song: ControlKey,
    pub play_pause: ControlKey,
    pub previous_song: ControlKey,
    pub quit: ControlKey,
    pub stop: ControlKey,
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
            clear: ControlKey::Char('c'),
            next_song: ControlKey::Char('>'),
            play_pause: ControlKey::Char('p'),
            previous_song: ControlKey::Char('<'),
            quit: ControlKey::Char('q'),
            stop: ControlKey::Char('s'),
        }
    }
}

pub trait toKeyCode
{
    fn keycode(&self) -> i32;
}

impl toKeyCode for i32
{
    fn keycode(&self) -> i32
    {
        *self
    }
}

impl toKeyCode for char
{
    fn keycode(&self) -> i32
    {
        *self as i32
    }
}

impl toKeyCode for ControlKey
{
    fn keycode(&self) -> i32
    {
        match *self
        {
            ControlKey::KeyCode(c) => return c,
            ControlKey::Char(c) => return c.keycode(),
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

        let keys = KeyConfig::new();

        Config
        {
            colors: ColorConfig::new(),
            addr: SocketAddr::new(mpd_ip, mpd_port),
            keys: keys,
        }
    }
}
