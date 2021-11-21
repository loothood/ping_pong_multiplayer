# ping_pong_multiplayer

Mulpiplayer ping pong.

Developed with Rust and GRPS. 
Game engine is tetra.
Cotrollers: Up and Down buttons.

To it run, install `SDL` lib for your OS.

To build on linux, install the following libs:

``` sudo apt install libasound2-dev libsdl2-dev ```

Starting:

run server: 

``` cargo run --bin server ```

run client: 

``` cargo run --bin client ```

run seсond client on the friend's computer in the same network:

``` cargo run --bin client ```

Play with friend:

![ping pong](https://habrastorage.org/getpro/habr/upload_files/344/c1c/f43/344c1cf43c4e0d696d6068051a33ca7d.gif)
