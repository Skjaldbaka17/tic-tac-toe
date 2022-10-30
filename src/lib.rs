#![no_std]

use soroban_sdk::{contractimpl, contracttype, symbol, vec, AccountId, Address, Env, Symbol, Vec};

const GAME_COUNT: Symbol = symbol!("GAME_COUNT");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game {
    pub challenger: AccountId,
    pub opposition: AccountId,
    pub p_turn: AccountId, // Player's turn
    pub board: Vec<CellState>,
    pub game_state: GameState,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameState {
    InPlay,
    Winner(AccountId),
    Draw,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellState {
    Empty,
    X,
    O,
}

pub struct TicTacToeContract;

pub trait TicTacToeTrait {
    fn create(env: Env, opposition: AccountId) -> u32;
    fn play(env: Env, game_id: u32, pos: u32) -> bool;
    fn get_game(env: Env, game_id: u32) -> Game;
}

#[contractimpl]
impl TicTacToeTrait for TicTacToeContract {
    fn create(env: Env, opposition: AccountId) -> u32 {
        // Check if invoker is an account (not a contract)
        if let Address::Account(address_id) = env.invoker() {
            let game_id = get_next_game_id(&env);
            let new_board = get_empty_board(&env);

            // Save new game to the chain
            env.data().set(
                game_id,
                Game {
                    challenger: address_id.clone(),
                    opposition,
                    board: new_board,
                    p_turn: address_id.clone(),
                    game_state: GameState::InPlay,
                },
            );

            // increment num of games
            env.data().set(GAME_COUNT, game_id + 1);
            return game_id;
        }
        panic!()
    }

    fn play(env: Env, game_id: u32, pos: u32) -> bool {
        assert!(pos <= 8, "Position supplied is not on board");

        if let Address::Account(address_id) = env.invoker() {
            match env.data().get::<_, Game>(game_id) {
                None => {
                    panic!("This game does not exist");
                }
                Some(w_game) => {
                    let mut game = w_game.unwrap();

                    // Assert the game is still "InPlay"
                    assert!(
                        game.game_state == GameState::InPlay,
                        "This game is finished!"
                    );

                    // Assert the invoker has the next turn of the game
                    assert_eq!(&address_id, &game.p_turn, "It's not your turn!");

                    let posit = game.board.get(pos).unwrap_or(Ok(CellState::X)).unwrap();
                    assert!(
                        posit == CellState::Empty,
                        "This play has already been made. {:?}",
                        posit
                    );

                    if address_id.eq(&game.opposition) {
                        game.board.set(pos, CellState::O); // Opposition is 'O'
                        game.p_turn = game.challenger.clone();
                    } else if address_id.eq(&game.challenger) {
                        game.board.set(pos, CellState::X); // Challenger is 'X'
                        game.p_turn = game.opposition.clone();
                    } else {
                        panic!()
                    }

                    game.game_state = get_current_state(&game);
                    env.data().set(game_id, &game);

                    return game.game_state != GameState::InPlay;
                }
            }
        } else {
            panic!();
        }
    }

    fn get_game(env: Env, game_id: u32) -> Game {
        match env.data().get::<_, Game>(game_id) {
            None => {
                panic!("This game does not exist");
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


mod test;
