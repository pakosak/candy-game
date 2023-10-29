# Communication

## Server

Keeps a list of running games, their worlds and current/past players. Automatically updates all
movable parts in the worlds and applies client inputs. Allows game listing, creating, polling
and application of actions.

### API

```
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
    "player_id": uint
}

POST /action
->
{
    "game_id": uint,
    "player_id": uint,
    "action": Action
}

Action =
{
    "type": "shoot"
}
|
{
    "type": "move",
    "direction": "up" | "down" | "left" | "right"
}

POST /state
->
{
    "game_id": uint,
    "player_id": uint
}
<-
{
    "map": string,
    "finished": bool,
    "dead_players": [string, ...],
    "logs": [string, ...]
}

```

## Client

Client can either list all games, create a new one or join an existing one. Listing and creating
actions are one timers, after joining an existing one (using game id) the client receives unique
player id which it uses to move the player. It also has to periodically poll the game state.
Once the game is over (or the player is dead), server will stop accepting client inputs.
