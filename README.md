# PomodoroTimer
A pomodoro timer/study habits app that incorporates all the features that I can't find in paid ones. 

# How to Use 
If you're on MacOS, for now, there's a build availalbe in ```releases/```. 

For Windows/Linux users, you're gonna have to build the application on your own system (until I figure out how to do it myself :D). 

### Steps: 
Ensure you have Tauri and Rust downloaded, using [their setup guide](https://v2.tauri.app/start/prerequisites/). 

Then clone the repo, run ```npm i``` to install dependencies, and build the app using the command

```npm run tauri build```. 

In ```src-tauri/target/release/bundle```, there should be an executable for your system. 

Apologies for the trouble while I figure out how to make applications for other platforms. 

# Changelog
- added a macos build
- made the repo, implemented phase shifts and semi-configurable phase times. Fixed several race conditions. 


# Motivation 
I paid 10 dollars for a pomodoro app on my mac that blocked apps and allowed me to change durations of breaks (i had an itunes giftcard so it wasn't a scam). I think
it's entirely possible to create a pomodoro app that is fast, compact, and **actually has features we'd want to use**. 

Shoutout to my friends (yong and tida) for giving me feature ideas!
