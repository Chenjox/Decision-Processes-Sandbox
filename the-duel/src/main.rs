use std::fs::File;
use std::io::Write;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;



#[derive(Clone)]
pub enum Action {
    ATTACK,
    FINCH
}
pub enum GameOutcome {
    WIN(u64),
    TIE,
    CONTINUE,
    INTERRUPTED
}

struct PlayerState {
    max_hit_points: i64,
    current_hit_points: i64,
}

struct GameState {
    player_one_state: PlayerState,
    player_two_state: PlayerState
}

trait GameAgent {

    fn decide_action(&mut self, own_player_state: &PlayerState, opposing_player_state: &Option<PlayerState>) -> Action;

    fn strategy_name(&self) -> String;
}

struct Game {
    player_one_agent: Box<dyn GameAgent>,
    player_two_agent: Box<dyn GameAgent>
}

impl Game {
    
    fn step_game(&mut self, state: &mut GameState){
        // get actions for current game state
        let player_one_action = self.player_one_agent.decide_action(&state.player_one_state, &None);
        let player_two_action = self.player_two_agent.decide_action(&state.player_two_state, &None);
        // Decide what happens

        // Check whether player attacks, or if player blocks
        match (player_one_action,player_two_action) {
            // Both Attack!
            (Action::ATTACK,Action::ATTACK) => {
                state.player_one_state.current_hit_points -= 1;
                state.player_two_state.current_hit_points -= 1;
            },
            // First Attacks, Second Counters!
            (Action::ATTACK,Action::FINCH) => {
                state.player_one_state.current_hit_points -= 2;
            },
            // First Counters, Second Attacks!
            (Action::FINCH,Action::ATTACK) => {
                state.player_two_state.current_hit_points -= 2;
            },
            (Action::FINCH,Action::FINCH) => {
                state.player_one_state.current_hit_points -= 1;
                state.player_two_state.current_hit_points -= 1;
            }
        }
    }

    fn check_end_condition(&self, state: &GameState) -> GameOutcome{
        if state.player_one_state.current_hit_points <= 0 && state.player_two_state.current_hit_points <= 0 {
            return GameOutcome::TIE;
        }
        if state.player_one_state.current_hit_points <= 0 {
            return GameOutcome::WIN(2);
        }
        if state.player_two_state.current_hit_points <= 0 {
            return GameOutcome::WIN(1);
        }
        return GameOutcome::CONTINUE;
    }

}

struct AttackAgent;

impl GameAgent for AttackAgent {
    fn decide_action(&mut self, _own_player_state: &PlayerState, _opposing_player_state: &Option<PlayerState>) -> Action {
        return Action::ATTACK;
    }

    fn strategy_name(&self) -> String {
        return String::from("Always Attack");
    }
}

struct RandomAgent<T : Rng> {
    current_random: T,
    probability_of_attack: f64
}

impl<T : Rng> GameAgent for RandomAgent<T> {
    fn decide_action(&mut self, _own_player_state: &PlayerState, _opposing_player_state: &Option<PlayerState>) -> Action {
        let decision = self.current_random.random_bool(self.probability_of_attack);
        if decision {
            return Action::ATTACK;
        } else {
            return Action::FINCH;
        }
    }

    fn strategy_name(&self) -> String {
        return format!("Attack with probability {}", self.probability_of_attack);
    }
}

struct MarkovRandomAgent<T : Rng> {
    current_random: T,
    change_to_attack_prob: f64,
    change_to_finch_prob: f64,
    current_strategy: Action
}

impl<T : Rng> GameAgent for MarkovRandomAgent<T> {

    fn decide_action(&mut self, _own_player_state: &PlayerState, _opposing_player_state: &Option<PlayerState>) -> Action {
        
        match self.current_strategy {
            Action::ATTACK => {
                let decision = self.current_random.random_bool(self.change_to_finch_prob);
                if decision {
                    self.current_strategy = Action::FINCH;
                }
            }
            Action::FINCH => {
                let decision = self.current_random.random_bool(self.change_to_attack_prob);
                if decision {
                    self.current_strategy = Action::ATTACK;
                }
            }
        }

        return self.current_strategy.clone();
    }

    fn strategy_name(&self) -> String {
        return format!("Markov Chain with probabilities {}, {}", self.change_to_attack_prob, self.change_to_finch_prob);
    }
}

fn main() {
    println!("Initializing Game");

    let max_hp = 100;
    let rng = ChaCha12Rng::seed_from_u64( 102 );
    
    let mut game = Game {
        player_one_agent: Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.05,
            change_to_finch_prob: 0.8,
            current_strategy: Action::ATTACK
        }),
        player_two_agent: Box::new(RandomAgent {
            current_random: rng,
            probability_of_attack: 0.1
        })
    };

    let mut state = GameState { 
        player_one_state: PlayerState {
            max_hit_points: max_hp,
            current_hit_points: max_hp
        },
        player_two_state: PlayerState {
            max_hit_points: max_hp,
            current_hit_points: max_hp
        } 
    };
    let path = "results.csv";
    let mut output = File::create(path).unwrap();
    let mut step_count = 0;
    loop {
        // step
        game.step_game(&mut state);
        
        // writeout
        
        write!(output, "{},{},{}\n", 
        step_count, 
        &state.player_one_state.current_hit_points, 
        &state.player_two_state.current_hit_points).unwrap();

        // check
        let condition = game.check_end_condition(&state);
        match condition {
            GameOutcome::WIN(id) => {
                println!("Status {} [Current/Max]:\n Player 1: {}/{} HP running strategy: {}\n Player 2: {}/{} HP running strategy: {}", 
                step_count,
                &state.player_one_state.current_hit_points, 
                &state.player_one_state.max_hit_points, 
                &game.player_one_agent.strategy_name(),
                &state.player_two_state.current_hit_points,
                &state.player_two_state.max_hit_points,
                &game.player_two_agent.strategy_name(),);
                println!("Player {} wins!",id);
                break;
            },
            GameOutcome::TIE => {
                println!("Status {} [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP", 
                step_count,
                &state.player_one_state.current_hit_points, 
                &state.player_one_state.max_hit_points, 
                &state.player_two_state.current_hit_points,
                &state.player_two_state.max_hit_points);
                println!("Game ended in a Tie");
                break;
            },
            GameOutcome::INTERRUPTED => {
                panic!("Unexpected Event happened");
            }
            GameOutcome::CONTINUE => {
                println!("Status {} [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP", 
                step_count,
                &state.player_one_state.current_hit_points, 
                &state.player_one_state.max_hit_points, 
                &state.player_two_state.current_hit_points,
                &state.player_two_state.max_hit_points);
            }
        }
        step_count += 1;
    }
    println!("Game finished!");
}
