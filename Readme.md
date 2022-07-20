# Tag Simulation

A agent-based simulation of the game Tag. Can be run on the command line and visualises the game field with Ascii art. '*' is "it", the other players are shown as '.' 

## Run

Run it using cargo on the command line, the first parameter to provide is the number of players, it is optional and defaults to 5. The second is the number of steps to run, and it defaults to 100.

```sh
cargo run [10 [200]]
```

## Test

```sh
cargo test
```

## Modules

### environment

Representations of the state of play and the interactions the agents can make with the environment.

### agent

The limitations and strategies by which the players play.

### simulation

Runs through the process of activating the agent to make a decision and applying those mutations back to the environment.

### viewer

Renders the environment and actions as ascii art on the command line.
