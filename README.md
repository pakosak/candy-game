# Game interface

Use arrow keys to move, space to shoot, ESC to exit.
Collect all candies, don't get yourself killed and enter the exit.

```
>   player
⏾   candy
*   mob
X   exit
```

## Multiplayer

The game was meant to be enjoyed with your friends. If you want to play local coop, run a server:

```
cargo run --bin server
```

And then connect with clients:

```
cargo run --bin client -s localhost
```

<img width="600" src="vhs/demo.gif" />
