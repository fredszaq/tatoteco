# Tatoteco

![logo](tatoteco-logo.svg)

Tabletop terrain controller

Tatoteco is an app used to display images, with controls on a web app. The intended usage is to display maps during 
tabletop/roleplaying games. 

## Setup
 
### Hardware
 - old TV, laid flat on game table (only full hd is properly supported for now, but easy to change in the code if you need another resolution)
 - raspberrypi (tested with rp3 and archlinuxarm)
 - phone for the DM to show the controller

### Data
 - maps/images to want to show on the tv


### Software
 - gtk3
 - run with `cargo run -- -r resources_path` (compilation will take some time on a pi, you probably need to setup some swap)
 - xfce auto login + tatoteco launched at startup
 - navigate to `http://[raspberrypi-ip]:8080` from the phone to get the controller