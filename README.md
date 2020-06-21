Client library for [Lobby](https://github.com/LeonardBesson/lobby-server)

## Features

* It integrates easily into any engine by hooking into your game loop.
* Aims to keep the overhead as minimal as possible. Lightweight, non-blocking networking built on top of [Mio](https://github.com/tokio-rs/mio).
* Unopinionated treading. It doesn't matter whether your game is single threaded or you use a complex parallel ECS like [Legion](https://github.com/TomGillen/legion) or [Specs](https://github.com/amethyst/specs)

## Content

`lobby-lib` contains the client lib. 

`src` contains a debug GUI client implementing the library. It serves as a visual aid, as well as an example implementation. It uses ImGUI with [wgpu](https://github.com/gfx-rs/wgpu-rs) as backend and looks like this:

![debug-gui](screenshots/debug-gui.png?raw=true "Debug GUI")
