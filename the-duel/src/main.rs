use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;



pub enum Action {
    ATTACK,
    COUNTER
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
            (Action::ATTACK,Action::COUNTER) => {
                state.player_two_state.current_hit_points -= 2;
            },
            // First Counters, Second Attacks!
            (Action::COUNTER,Action::ATTACK) => {
                state.player_one_state.current_hit_points -= 2;
            },
            (Action::COUNTER,Action::COUNTER) => {
                // Tumbleweed dances across the scene
            }
        }
    }

    fn check_end_condition(&self, state: &GameState) -> GameOutcome{
        if state.player_one_state.current_hit_points <= 0 && state.player_one_state.current_hit_points <= 0 {
            return GameOutcome::TIE;
        }
        if state.player_one_state.current_hit_points <= 0 {
            return GameOutcome::WIN(1);
        }
        if state.player_two_state.current_hit_points <= 0 {
            return GameOutcome::WIN(0);
        }
        return GameOutcome::CONTINUE;
    }

}

struct AttackAgent;

impl GameAgent for AttackAgent {
    fn decide_action(&mut self, own_player_state: &PlayerState, opposing_player_state: &Option<PlayerState>) -> Action {
        return Action::ATTACK;
    }
}

struct RandomAgent<T : Rng> {
    current_random: T
}

impl<T : Rng> GameAgent for RandomAgent<T> {
    fn decide_action(&mut self, own_player_state: &PlayerState, opposing_player_state: &Option<PlayerState>) -> Action {
        let decision = self.current_random.random_bool(0.5);
        if decision {
            return Action::ATTACK;
        } else {
            return Action::COUNTER;
        }
    }
}

fn main() {
    println!("Initializing Game");

    let max_hp = 100;
    let rng = ChaCha12Rng::seed_from_u64( 100 );
    
    let mut game = Game {
        player_one_agent: Box::new(AttackAgent),
        player_two_agent: Box::new(RandomAgent {
            current_random: rng
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
    loop {
        // step
        game.step_game(&mut state);

        // check
        let condition = game.check_end_condition(&state);
        match condition {
            GameOutcome::WIN(id) => {
                println!("Status [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP", &state.player_one_state.current_hit_points, 
                &state.player_one_state.max_hit_points, 
                &state.player_two_state.current_hit_points,
                &state.player_two_state.max_hit_points);
                println!("Player {} wins!",id);
                break;
            },
            GameOutcome::TIE => {
                println!("Status [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP", &state.player_one_state.current_hit_points, 
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
                println!("Status [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP", &state.player_one_state.current_hit_points, 
                &state.player_one_state.max_hit_points, 
                &state.player_two_state.current_hit_points,
                &state.player_two_state.max_hit_points);
            }
        }
    }
    println!("Game finished!");
}
