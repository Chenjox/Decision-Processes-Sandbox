use core::num;
use std::io::Write;
use std::rc::Rc;
use std::{cell::RefCell, fs::File};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

#[derive(Clone)]
pub enum Action {
    ATTACK,
    FINCH,
}
pub enum GameOutcome {
    WIN(u64),
    TIE,
    CONTINUE,
    INTERRUPTED,
}

struct PlayerState {
    max_hit_points: i64,
    current_hit_points: i64,
}

struct GameState {
    player_one_state: PlayerState,
    player_two_state: PlayerState,
    player_one_action: Option<Action>,
    player_two_action: Option<Action>,
}

trait GameAgent {
    fn decide_action(
        &mut self,
        own_player_state: &PlayerState,
        opposing_player_actions: &Option<Action>,
        opposing_player_state: &Option<PlayerState>,
    ) -> Action;

    fn strategy_name(&self) -> String;

    fn copy_self_to_anom(&self) -> Box<dyn GameAgent>;
}

struct Game {
    player_one_agent: Box<dyn GameAgent>,
    player_two_agent: Box<dyn GameAgent>,
}

impl Game {
    fn step_game(&mut self, state: &mut GameState) {
        // get actions for current game state
        let player_one_action = self.player_one_agent.decide_action(
            &state.player_one_state,
            &state.player_two_action,
            &None,
        );
        let player_two_action = self.player_two_agent.decide_action(
            &state.player_two_state,
            &state.player_one_action,
            &None,
        );
        // Decide what happens

        // Check whether player attacks, or if player blocks
        match (player_one_action, player_two_action) {
            // Both Attack!
            (Action::ATTACK, Action::ATTACK) => {
                state.player_one_state.current_hit_points -= 1;
                state.player_two_state.current_hit_points -= 1;
            }
            // First Attacks, Second Counters!
            (Action::ATTACK, Action::FINCH) => {
                state.player_one_state.current_hit_points -= 1;
            }
            // First Counters, Second Attacks!
            (Action::FINCH, Action::ATTACK) => {
                state.player_two_state.current_hit_points -= 1;
            }
            (Action::FINCH, Action::FINCH) => {
                state.player_one_state.current_hit_points -= 1;
                state.player_two_state.current_hit_points -= 1;
            }
        }
    }

    fn check_end_condition(&self, state: &GameState) -> GameOutcome {
        if state.player_one_state.current_hit_points <= 0
            && state.player_two_state.current_hit_points <= 0
        {
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

#[derive(Clone)]
struct AttackAgent;

impl GameAgent for AttackAgent {
    fn decide_action(
        &mut self,
        _own_player_state: &PlayerState,
        _opposing_player_actions: &Option<Action>,
        _opposing_player_state: &Option<PlayerState>,
    ) -> Action {
        return Action::ATTACK;
    }

    fn strategy_name(&self) -> String {
        return String::from("Always Attack");
    }

    fn copy_self_to_anom(&self) -> Box<dyn GameAgent> {
        Box::new(Self {})
    }
}

#[derive(Clone)]
struct MirrorAgent;

impl GameAgent for MirrorAgent {
    fn decide_action(
        &mut self,
        _own_player_state: &PlayerState,
        opposing_player_actions: &Option<Action>,
        _opposing_player_state: &Option<PlayerState>,
    ) -> Action {
        if let Some(action) = opposing_player_actions {
            return action.clone();
        } else {
            return Action::ATTACK;
        }
    }

    fn strategy_name(&self) -> String {
        return String::from("Always Mirror the opposing action");
    }

    fn copy_self_to_anom(&self) -> Box<dyn GameAgent> {
        Box::new(Self {})
    }
}

#[derive(Clone)]
struct RandomAgent<T: Rng + 'static> {
    current_random: Rc<RefCell<T>>,
    probability_of_attack: f64,
}

impl<T: Rng> GameAgent for RandomAgent<T> {
    fn decide_action(
        &mut self,
        _own_player_state: &PlayerState,
        _opposing_player_actions: &Option<Action>,
        _opposing_player_state: &Option<PlayerState>,
    ) -> Action {
        let decision = self
            .current_random
            .borrow_mut()
            .random_bool(self.probability_of_attack);
        if decision {
            return Action::ATTACK;
        } else {
            return Action::FINCH;
        }
    }

    fn strategy_name(&self) -> String {
        return format!("Attack with probability {}", self.probability_of_attack);
    }

    fn copy_self_to_anom(&self) -> Box<dyn GameAgent> {
        Box::new(Self {
            current_random: self.current_random.clone(),
            probability_of_attack: self.probability_of_attack,
        })
    }
}

#[derive(Clone)]
struct OneStepDecisionProcessAgent {
    cost_losing_hp: f64,
    cost_not_losing_hp: f64,
    cost_equivalent_exchange: f64,
    num_turns: i64,
    num_attacks: i64,
}

impl GameAgent for OneStepDecisionProcessAgent {
    fn decide_action(
        &mut self,
        _own_player_state: &PlayerState,
        opposing_player_actions: &Option<Action>,
        _opposing_player_state: &Option<PlayerState>,
    ) -> Action {
        if let Some(ack) = opposing_player_actions {
            match ack {
                Action::ATTACK => {
                    self.num_attacks += 1;
                }
                Action::FINCH => {}
            };
        }
        self.num_turns += 1;

        // Guesstimate probability of attack
        let prob = (self.num_attacks as f64) / (self.num_turns as f64);

        let attack_reward =
            self.cost_losing_hp * (1.0 - prob) + self.cost_equivalent_exchange * prob;
        let finch_reward =
            self.cost_not_losing_hp * prob + self.cost_equivalent_exchange * (1.0 - prob);

        if attack_reward > finch_reward {
            return Action::ATTACK;
        } else {
            return Action::FINCH;
        }
    }

    fn strategy_name(&self) -> String {
        return format!("Estimate Probability of Attack, and design optimal one-step decision.");
    }

    fn copy_self_to_anom(&self) -> Box<dyn GameAgent> {
        Box::new(Self {
            cost_losing_hp: self.cost_losing_hp,
            cost_not_losing_hp: self.cost_not_losing_hp,
            cost_equivalent_exchange: self.cost_equivalent_exchange,
            num_turns: self.num_turns,
            num_attacks: self.num_attacks,
        })
    }
}

#[derive(Clone)]
struct MarkovRandomAgent<T: Rng + 'static> {
    current_random: Rc<RefCell<T>>,
    change_to_attack_prob: f64,
    change_to_finch_prob: f64,
    current_strategy: Action,
}

impl<T: Rng + 'static> GameAgent for MarkovRandomAgent<T> {
    fn decide_action(
        &mut self,
        _own_player_state: &PlayerState,
        _opposing_player_actions: &Option<Action>,
        _opposing_player_state: &Option<PlayerState>,
    ) -> Action {
        match self.current_strategy {
            Action::ATTACK => {
                let decision = self
                    .current_random
                    .borrow_mut()
                    .random_bool(self.change_to_finch_prob);
                if decision {
                    self.current_strategy = Action::FINCH;
                }
            }
            Action::FINCH => {
                let decision = self
                    .current_random
                    .borrow_mut()
                    .random_bool(self.change_to_attack_prob);
                if decision {
                    self.current_strategy = Action::ATTACK;
                }
            }
        }

        return self.current_strategy.clone();
    }

    fn strategy_name(&self) -> String {
        return format!(
            "Markov Chain with probabilities {}, {}",
            self.change_to_attack_prob, self.change_to_finch_prob
        );
    }

    fn copy_self_to_anom(&self) -> Box<dyn GameAgent> {
        Box::new(Self {
            current_random: self.current_random.clone(),
            change_to_attack_prob: self.change_to_attack_prob,
            change_to_finch_prob: self.change_to_finch_prob,
            current_strategy: self.current_strategy.clone(),
        })
    }
}

fn pit_agents_against_each_other() {
    let rng = Rc::new(RefCell::new(ChaCha12Rng::seed_from_u64(106)));

    let num_retrials = 5000;

    let max_hp = 600;
    let list_of_agents: Vec<Box<dyn GameAgent>> = vec![
        Box::new(RandomAgent {
            current_random: rng.clone(),
            probability_of_attack: 0.1,
        }),
        Box::new(RandomAgent {
            current_random: rng.clone(),
            probability_of_attack: 0.3,
        }),
        Box::new(RandomAgent {
            current_random: rng.clone(),
            probability_of_attack: 0.5,
        }),
        Box::new(RandomAgent {
            current_random: rng.clone(),
            probability_of_attack: 0.7,
        }),
        Box::new(RandomAgent {
            current_random: rng.clone(),
            probability_of_attack: 0.9,
        }),
        Box::new(AttackAgent {}),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.1,
            change_to_finch_prob: 0.1,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.5,
            change_to_finch_prob: 0.1,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.9,
            change_to_finch_prob: 0.1,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.1,
            change_to_finch_prob: 0.5,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.5,
            change_to_finch_prob: 0.5,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.9,
            change_to_finch_prob: 0.5,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.1,
            change_to_finch_prob: 0.9,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.5,
            change_to_finch_prob: 0.9,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MarkovRandomAgent {
            current_random: rng.clone(),
            change_to_attack_prob: 0.9,
            change_to_finch_prob: 0.9,
            current_strategy: Action::ATTACK,
        }),
        Box::new(MirrorAgent),
        Box::new(OneStepDecisionProcessAgent {
            cost_equivalent_exchange: -3.0,
            cost_losing_hp: -3.0,
            cost_not_losing_hp: -1.0,
            num_turns: 0,
            num_attacks: 0,
        }),
    ];

    let num_agents = list_of_agents.len();
    let mut win_matrix = vec![vec![0; num_agents]; num_agents];

    // Fight two against each other
    for agent1 in list_of_agents.iter().enumerate() {
        for agent2 in list_of_agents.iter().enumerate() {
            for i in 0..num_retrials {
                let mut game = Game {
                    player_one_agent: agent1.1.copy_self_to_anom(),
                    player_two_agent: agent2.1.copy_self_to_anom(),
                };

                let mut state = GameState {
                    player_one_state: PlayerState {
                        max_hit_points: max_hp,
                        current_hit_points: max_hp,
                    },
                    player_two_state: PlayerState {
                        max_hit_points: max_hp,
                        current_hit_points: max_hp,
                    },
                    player_one_action: None,
                    player_two_action: None,
                };

                loop {
                    // step
                    game.step_game(&mut state);
                    let condition = game.check_end_condition(&state);
                    match condition {
                        GameOutcome::WIN(id) => {
                            if id == 1 {
                                win_matrix[agent1.0][agent2.0] += 1;
                            } else {
                                win_matrix[agent2.0][agent1.0] += 1;
                            }
                            break;
                        }
                        GameOutcome::TIE => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    println!("{:?}", win_matrix);

    let path = "pitting-results.csv";
    let mut output = File::create(path).unwrap();
    for i in 0..num_agents {
        write!(output, "{}", win_matrix[0][i]).unwrap();
        for j in 1..num_agents {
            write!(output, ",{}", win_matrix[j][i]).unwrap();
        }
        write!(output, "\n").unwrap();
    }
}

fn main() {
    println!("Initializing Game");

    let max_hp = 600;
    let rng_cell = Rc::new(RefCell::new(ChaCha12Rng::seed_from_u64(106)));

    let mut game = Game {
        player_one_agent: Box::new(OneStepDecisionProcessAgent {
            cost_equivalent_exchange: -3.0,
            cost_losing_hp: -3.0,
            cost_not_losing_hp: -1.0,
            num_turns: 0,
            num_attacks: 0,
        }),
        player_two_agent: Box::new(MarkovRandomAgent {
            current_random: rng_cell.clone(),
            change_to_attack_prob: 0.3,
            change_to_finch_prob: 0.6,
            current_strategy: Action::FINCH,
        }),
    };

    let mut state = GameState {
        player_one_state: PlayerState {
            max_hit_points: max_hp,
            current_hit_points: max_hp,
        },
        player_two_state: PlayerState {
            max_hit_points: max_hp,
            current_hit_points: max_hp,
        },
        player_one_action: None,
        player_two_action: None,
    };
    let path = "results.csv";
    let mut output = File::create(path).unwrap();
    let mut step_count = 0;
    loop {
        // step
        game.step_game(&mut state);

        // writeout

        write!(
            output,
            "{},{},{}\n",
            step_count,
            &state.player_one_state.current_hit_points,
            &state.player_two_state.current_hit_points
        )
        .unwrap();

        // check
        let condition = game.check_end_condition(&state);
        match condition {
            GameOutcome::WIN(id) => {
                println!(
                    "Status {} [Current/Max]:\n Player 1: {}/{} HP running strategy: {}\n Player 2: {}/{} HP running strategy: {}",
                    step_count,
                    &state.player_one_state.current_hit_points,
                    &state.player_one_state.max_hit_points,
                    &game.player_one_agent.strategy_name(),
                    &state.player_two_state.current_hit_points,
                    &state.player_two_state.max_hit_points,
                    &game.player_two_agent.strategy_name(),
                );
                println!("Player {} wins!", id);
                break;
            }
            GameOutcome::TIE => {
                println!(
                    "Status {} [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP",
                    step_count,
                    &state.player_one_state.current_hit_points,
                    &state.player_one_state.max_hit_points,
                    &state.player_two_state.current_hit_points,
                    &state.player_two_state.max_hit_points
                );
                println!("Game ended in a Tie");
                break;
            }
            GameOutcome::INTERRUPTED => {
                panic!("Unexpected Event happened");
            }
            GameOutcome::CONTINUE => {
                println!(
                    "Status {} [Current/Max]:\n Player 1: {}/{} HP\n Player 2: {}/{} HP",
                    step_count,
                    &state.player_one_state.current_hit_points,
                    &state.player_one_state.max_hit_points,
                    &state.player_two_state.current_hit_points,
                    &state.player_two_state.max_hit_points
                );
            }
        }
        step_count += 1;
    }
    println!("Game finished!");

    //pit_agents_against_each_other();
}
