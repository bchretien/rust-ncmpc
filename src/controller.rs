extern crate ncurses;
use ncurses as nc;

use config::*;
use model::*;

pub struct Controller<'c, 'm: 'c>
{
    model: &'c mut Model<'m>,
    config: &'c Config,
}

impl<'c, 'm> Controller<'c, 'm>
{
    pub fn new(model: &'c mut Model<'m>, config: &'c Config) -> Controller<'c,'m>
    {
        Controller
        {
            model: model,
            config: config,
        }
    }

    pub fn run(&mut self)
    {
        self.model.init();

        let mut ch = nc::getch();
        // q
        while ch != 113
        {
            match ch
            {
                // play
                112 => {
                    self.model.playlist_play();
                }
                // stop
                115 => {
                    self.model.playlist_stop();
                }
                _ => {
                    nc::mvprintw(2, 4, &format!("Pressed {}", ch));
                }
            }
            nc::refresh();
            ch = nc::getch();
        }

        self.model.deinit();
    }
}
