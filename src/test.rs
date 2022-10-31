#![cfg(test)]

extern crate std;

use super::*;
use soroban_sdk::{testutils::Accounts, Env, Address};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TicTacToeContract);
    let client = TicTacToeContractClient::new(&env, &contract_id);
    let challenger = env.accounts().generate();
    let opposition = env.accounts().generate();
    std::println!("challenger: {:?}", challenger);
    std::println!("opposition: {:?}", opposition);

    let game_id = client.with_source_account(&challenger).create(&Address::Account(opposition.clone()));
    std::println!("Game Id: {}", game_id);
    for tmp in 0..3 {
        client.with_source_account(&challenger).play(&game_id, &tmp);
        if tmp == 2 {
            let game_result = client.get_game(&game_id);
            assert!(
                game_result.game_state != GameState::InPlay,
                "Game should be done"
            );
            assert_eq!(
                game_result.game_state,
                GameState::Winner(Address::Account(challenger.clone())),
                "Challenger should've won"
            );
            std::println!("{:?}", game_result.board.clone());
        } else {
            client.with_source_account(&opposition).play(&game_id, &(tmp + 3));
        }
    }
    let game = client.with_source_account(&opposition).get_game(&game_id);
    std::print!("{:?}", game.board);
}
