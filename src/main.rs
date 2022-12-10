use std::env;

use clap::{Parser, Subcommand};
use notify_rust::{Notification, Hint};
use pulsectl::controllers::{SinkController, DeviceControl};

const VOLUME_NOTIFICATION_SUMMARY: &str = "Volume";
const VOLUME_NOTIFICATION_ID: u32 = 2022;
const VOLUME_NOTIFICATION_APPNAME: &str = "volume-control-rs";
const VOLUME_NOTIFICATION_TIMEOUT: i32 = 1300;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Increase volume by given percent")]
    Increase { percent: f64 },

    #[command(about = "Decrease volume by given percent")]
    Decrease { percent: f64 },

    #[command(about = "Mute output")]
    Mute,
}

fn show_volume_notification(volume: u32, is_mute: bool, icon_prefix: String) {
    let volume_progress = volume%101;

    let vol_format = format!("{}%", volume);
    let body = if is_mute { "Muted" } else { vol_format.as_str() };

    let mut icon = icon_prefix;
    if is_mute {
        icon += "audio-volume-muted.svg";
    } else {
        icon += match volume {
            0..=33 => { "audio-volume-low.svg" }
            34..=66 => { "audio-volume-medium.svg" }
            67.. => { "audio-volume-high.svg" }
        };
    }

    _ =  Notification::new()
       .summary(VOLUME_NOTIFICATION_SUMMARY)
       .body(body)
       .hint(Hint::CustomInt("value".to_string(), volume_progress.try_into().unwrap()))
       .appname(VOLUME_NOTIFICATION_APPNAME)
       .timeout(VOLUME_NOTIFICATION_TIMEOUT)
       .id(VOLUME_NOTIFICATION_ID)
       .icon(&icon)
       .show();
}


fn main() {
    let cli = Cli::parse();
    let mut handler = SinkController::create().unwrap();
    let default_device_index = handler.get_default_device().unwrap().index;

    let icon_path_prefix = format!("{}/.local/share/icons/Colloid-orange-dark/status/24/", env!("HOME")); 

    match &cli.command {
        Commands::Increase { percent } => {
            handler.increase_device_volume_by_percent(default_device_index, percent.to_owned()/100.0);

            let volume = handler.get_default_device().unwrap().volume.avg().0/655;
            show_volume_notification(volume, false, icon_path_prefix);
        } 
        Commands::Decrease { percent } => {
            handler.decrease_device_volume_by_percent(default_device_index, percent.to_owned()/100.0);

            let volume = handler.get_default_device().unwrap().volume.avg().0/655;
            show_volume_notification(volume, false, icon_path_prefix);
        }
        Commands::Mute => {
            let is_mute = handler.get_default_device().unwrap().mute;
            handler.set_device_mute_by_index(default_device_index, !is_mute);
            
            let new_state = handler.get_default_device().unwrap();
            let volume = new_state.volume.avg().0/655;
            let mute = new_state.mute;
            show_volume_notification(volume, mute, icon_path_prefix);
        }
    }
}
