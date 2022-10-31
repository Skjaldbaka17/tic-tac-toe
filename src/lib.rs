#![no_std]

use soroban_sdk::{
    contracterror, contractimpl, contracttype, panic_error, symbol, vec, Address, Env, Symbol, Vec,
};

const GAME_COUNT: Symbol = symbol!("GAME_COUNT");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game {
    pub challenger: Address,
    pub opposition: Address,
    pub p_turn: Address, // Player's turn
    pub board: Vec<CellState>,
    pub game_state: GameState,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameState {
    InPlay,
    Winner(Address),
    Draw,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellState {
    Empty,
    X,
    O,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    GameDoesNotExist = 1,
    GameFinito = 2,
    NotYourTurn = 3,
    InvalidPlay = 4,
    NonPlayer = 5,
}

pub struct TicTacToeContract;

pub trait TicTacToeTrait {
    fn create(env: Env, opposition: Address) -> u32;
    fn play(env: Env, game_id: u32, pos: u32) -> bool;
    fn get_game(env: Env, game_id: u32) -> Game;
}

#[contractimpl]
impl TicTacToeTrait for TicTacToeContract {
    fn create(env: Env, opposition: Address) -> u32 {
        let game_id = get_next_game_id(&env);
        let new_board = get_empty_board(&env);

        // Save new game to the chain
        env.data().set(
            game_id,
            Game {
                challenger: env.invoker(),
                opposition: opposition.clone(),
                board: new_board,
                p_turn: env.invoker(),
                game_state: GameState::InPlay,
            },
        );

        // increment num of games
        env.data().set(GAME_COUNT, game_id + 1);
        return game_id;
    }

    fn play(env: Env, game_id: u32, pos: u32) -> bool {
        assert!(pos <= 8, "Position supplied is not on board");

        match env.data().get::<_, Game>(game_id) {
            None => {
                panic_error!(&env, Error::GameDoesNotExist)
            }
            Some(w_game) => {
                let mut game = w_game.unwrap();

                // Assert the game is still "InPlay"
                if game.game_state != GameState::InPlay {
                    panic_error!(&env, Error::GameFinito)
                }

                // Assert the invoker has the next turn of the game
                if !env.invoker().eq(&game.p_turn) {
                    panic_error!(&env, Error::NotYourTurn)
                }

                let posit = game.board.get(pos).unwrap_or(Ok(CellState::X)).unwrap();

                if posit != CellState::Empty {
                    panic_error!(&env, Error::InvalidPlay);
                }

                if env.invoker().eq(&game.opposition) {
                    game.board.set(pos, CellState::O); // Opposition is 'O'
                    game.p_turn = game.challenger.clone();
                } else if env.invoker().eq(&game.challenger) {
                    game.board.set(pos, CellState::X); // Challenger is 'X'
                    game.p_turn = game.opposition.clone();
                } else {
                    panic_error!(&env, Error::NonPlayer)
                }

                game.game_state = get_current_state(&game);
                env.data().set(game_id, &game);

                return game.game_state != GameState::InPlay;
            }
        }
    }

    fn get_game(env: Env, game_id: u32) -> Game {
        match env.data().get::<_, Game>(game_id) {
            None => {
                panic_error!(&env, Error::GameDoesNotExist)
            }
            Some(game) => game.unwrap(),
        }
    }
}

fn get_empty_board(env: &Env) -> Vec<CellState> {
    let mut new_board = vec![env];
    for _ in 1..10 {
        new_board.push_back(CellState::Empty);
    }
    new_board
}

fn get_cell_state(game: &Game, pos: u32) -> CellState {
    game.board.get_unchecked(pos).unwrap()
}

fn get_game_state(game: &Game, winner: CellState) -> GameState {
    match winner {
        CellState::X => return GameState::Winner(game.challenger.clone()),
        CellState::O => return GameState::Winner(game.opposition.clone()),
        _ => return GameState::InPlay,
    }
}

fn get_current_state(game: &Game) -> GameState {
    for tmp in 0..3 {
        if get_cell_state(&game, tmp) != CellState::Empty
            && get_cell_state(&game, tmp) == get_cell_state(&game, tmp + 3)
            && get_cell_state(&game, tmp) == get_cell_state(&game, tmp + 6)
        {
            return get_game_state(game, get_cell_state(&game, tmp));
        }

        let tmp = tmp * 3;

        if get_cell_state(&game, tmp) != CellState::Empty
            && get_cell_state(&game, tmp) == get_cell_state(&game, tmp + 1)
            && get_cell_state(&game, tmp) == get_cell_state(&game, tmp + 2)
        {
            return get_game_state(game, get_cell_state(&game, tmp));
        }
    }

    if (get_cell_state(&game, 4) != CellState::Empty
        && get_cell_state(&game, 0) == get_cell_state(&game, 4)
        && get_cell_state(&game, 0) == get_cell_state(&game, 8))
        || (get_cell_state(&game, 2) == get_cell_state(&game, 4)
            && get_cell_state(&game, 2) == get_cell_state(&game, 6))
    {
        return get_game_state(game, get_cell_state(&game, 4));
    }

    for tmp in 0..9 {
        if get_cell_state(game, tmp) == CellState::Empty {
            return GameState::InPlay;
        }
    }

    GameState::Draw
}

fn get_next_game_id(env: &Env) -> u32 {
    env.data().get(GAME_COUNT).unwrap_or(Ok(0)).unwrap()
}

#[cfg(test)]
mod test;
