# Communication

## Server

Keeps a list of running games, with their states + worlds. Automatically updates all
movable parts in them and applies client inputs.

### API

GET /games
<-
{
    "games": [
        {
            "id": uint,
            "name": string,
            "players": [string, ...],
            "finished": bool
        }, ...
    ]
}

POST /create
->
{
    "name": string,
    "width": uint,
    "height": uint,
    "mob_cnt": uint,
    "candy_cnt": uint
}
<-
{
    "game_id": uint
}

POST /join
->
{
    "game_id": uint,
    "player_name": string
}
<-
{
    "player_id": Option<uint>
}

GET /state/{game_id}
<-
{

}

## Client

Client can either list all games, create a new one or join an existing one. Listing and creating
actions are one timers, after joining an existing one (using game id) the client receives unique
player id which it uses to move the player. It also has to periodically poll the game state.
Once the game is over (or the player is dead), server will stop accepting client inputs.
