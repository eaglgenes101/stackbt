/*use {FIELD_HEIGHT, FIELD_WIDTH};
use stackbt::automata_impl::{
    automaton::Automaton,
    internal_state_machine::{InternalTransition, InternalStateMachine},
    map_wrappers::{CustomConstructor, CustomConstructedMachine, InputMachineMap, 
        InputMappedMachine},
    dual_state_machine::{DualTransition, DualStateMachine}
};
use stackbt::behavior_tree::{
    behavior_tree_node::{BehaviorTreeNode, NodeResult, Statepoint},
    base_nodes::{PredicateWait, WaitCondition, MachineWrapper},
    node_compositions::SerialRepeater,
    control_wrappers::{GuardFailure, GuardedNode, NodeGuard},
    serial_node::{EnumNode, NontermReturn, SerialBranchNode, SerialDecider, 
        NontermDecision, TermDecision},
    map_wrappers::{InputMappedNode, InputNodeMap, OutputMappedNode, OutputNodeMap},
    node_runner::NodeRunner
};
use rand::Rng;
use rand::distributions::{Pareto, Uniform};

// The basic structure of the boid AI is this: 
//     In panic? 
//         Yes: act erratically, avoid other boids
//         No: Impending obstacle? 
//             Yes: avoid
//             No: Boids nearby? 
//                 Yes: Fly in a flock, may panic based on proximity
//                 No: Conduct levy flight
// (There is no actual obstacle collision, it's mostly so boids stay
// on screen rather than flying off)

//On initialization, a timer is set to tick down per frame, 
//and when it fires off, the engine gets started. 

const LEVY_FLIGHT_SHAPE: f64 = 1.5;
const LEVY_FLIGHT_SCALE: f64 = 1.0/60.0;
const APPROACH_DISTANCE: f32 = 40.0;
const SEPARATE_DISTANCE: f32 = 20.0;
const PANIC_THRESHOLD: f32 = 120.0;
const PANIC_SEEKING_DECAY: f32 = 0.004;
const PANIC_FLOCKING_DECAY: f32 = 0.01;
const PANIC_RETURNING_DECAY: f32 = 0.005;

lazy_static! {
    static ref LEVY_FLIGHT_DISTRIBUTION: Pareto = {
        Pareto::new(LEVY_FLIGHT_SCALE, LEVY_FLIGHT_SHAPE)
    };
    static ref TURN_DISTRIBUTION: Uniform<f32> = {
        Uniform::new(-1.0_f32, 1.0_f32)
    };
}

fn relative_facing(self_sin_cos: (f32, f32), other_sin_cos: (f32, f32)) -> f32 {
    let crossprod = self_sin_cos.0*other_sin_cos.1 - self_sin_cos.1*other_sin_cos.0;
    crossprod.asin().to_degrees()
}

fn boid_info_gather(input: &RawInfo) -> BoidInfo {
    assert!(input.about_crowd.len() > 0, "Expected nonempty slice");
    let mut running_x_sum = 0.0_f32;
    let mut running_y_sum = 0.0_f32;
    let mut running_sqrdist_sum = 0.0_f32;
    let mut running_sin_sum = 0.0_f32;
    let mut running_cos_sum = 0.0_f32;
    for entry in input.about_crowd.iter() {
        let xoffset = entry.xpos - input.about_self.xpos;
        let yoffset = entry.ypos - input.about_self.ypos;
        running_x_sum += xoffset;
        running_y_sum += yoffset;
        running_sqrdist_sum += xoffset*xoffset + yoffset*yoffset;
        let face_sin_cos = entry.facing.to_radians().sin_cos();
        running_sin_sum += face_sin_cos.0;
        running_cos_sum += face_sin_cos.1;
    }
    BoidInfo {
        panic_level: input.panic_level,
        facing: input.about_self.facing,
        dx: running_x_sum / (input.about_crowd.len() as f32),
        dy: running_y_sum / (input.about_crowd.len() as f32),
        average_face_cos: running_cos_sum / (input.about_crowd.len() as f32),
        average_face_sin: running_sin_sum / (input.about_crowd.len() as f32),
        stdev: (running_sqrdist_sum / (input.about_crowd.len() as f32)).sqrt()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoidStatus {
    xpos: f32,
    ypos: f32,
    facing: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct RawInfo {
    panic_level: f32,
    about_self: BoidStatus,
    about_crowd: Box<[BoidStatus]>
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoidInfo {
    panic_level: f32,
    facing: f32,
    dx: f32,
    dy: f32,
    average_face_cos: f32, 
    average_face_sin: f32,
    stdev: f32
}

pub enum LevyFlight {
    Straight,
    Turning
}

impl DualTransition for LevyFlight {
    type Input = RawInfo;
    type Internal = (i32, f32);
    type Action = Statepoint<(f32, f32), ()>;

    fn step(self, input: &RawInfo, internal: &mut (i32, f32)) -> (Self::Action, Self) {
        if internal.0 <= 0 {
            let mut rng = rand::thread_rng();
            match self {
                LevyFlight::Straight => {
                    *internal = (179, rng.sample(*TURN_DISTRIBUTION))
                },
                LevyFlight::Turning => {
                    *internal = (
                        1 + (rng.sample(*LEVY_FLIGHT_DISTRIBUTION).ceil() as i32),
                        0.0_f32
                    )
                }
            }
            (
                Statepoint::Nonterminal((
                    internal.1, 
                    input.panic_level*(1.0_f32 - PANIC_SEEKING_DECAY)
                )), 
                self
            )
        } else {
            internal.0 -= 1;
            (
                Statepoint::Nonterminal((
                    internal.1, 
                    input.panic_level*(1.0_f32 - PANIC_SEEKING_DECAY)
                )),
                self
            )
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BoidNavigate;

impl WaitCondition for BoidNavigate {
    type Input = RawInfo;
    type Nonterminal = (f32, f32);
    type Terminal = ();
    fn do_end(&self, info: &RawInfo) -> Statepoint<(f32, f32), ()> {
        let input = boid_info_gather(info);
        if input.stdev > APPROACH_DISTANCE {
            if input.dx == 0.0_f32 && input.dy == 0.0_f32 {
                Statepoint::Nonterminal((
                    0.0_f32, 
                    input.panic_level*(1.0_f32-PANIC_SEEKING_DECAY)
                ))
            } else {
                let hypdist = input.dx.hypot(input.dy);
                let canonicalized_diff = relative_facing(
                    input.facing.to_radians().sin_cos(), 
                    (input.dx/hypdist, input.dy/hypdist)
                );
                if canonicalized_diff.abs() < 1.0_f32 {
                    Statepoint::Nonterminal((
                        canonicalized_diff,
                        input.panic_level*(1.0_f32-PANIC_SEEKING_DECAY)
                    ))
                } else {
                    Statepoint::Nonterminal((
                        canonicalized_diff.signum(),
                        input.panic_level*(1.0_f32-PANIC_SEEKING_DECAY)
                    ))
                }
            }
        } else if input.stdev > SEPARATE_DISTANCE {
            if input.average_face_sin == 0.0 && input.average_face_cos == 0.0 {
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    Statepoint::Nonterminal((
                        -1.0_f32,
                        input.panic_level*(1.0_f32-PANIC_FLOCKING_DECAY)
                    ))
                } else {
                    Statepoint::Nonterminal((
                        1.0_f32,
                        input.panic_level*(1.0_f32-PANIC_FLOCKING_DECAY)
                    ))
                }
            } else {
                let iface_mag = input.average_face_cos.hypot(
                    input.average_face_sin
                );
                let canonicalized_diff = relative_facing(
                    input.facing.to_radians().sin_cos(), 
                    (input.average_face_cos/iface_mag, input.average_face_sin/iface_mag)
                );
                if canonicalized_diff.abs() < 1.0_f32 {
                    Statepoint::Nonterminal((
                        canonicalized_diff,
                        input.panic_level*(1.0_f32-PANIC_FLOCKING_DECAY)
                    ))
                } else {
                    Statepoint::Nonterminal((
                        canonicalized_diff.signum(),
                        input.panic_level*(1.0_f32-PANIC_FLOCKING_DECAY)
                    ))
                }

            }
        } else if input.dx == 0.0_f32 && input.dy == 0.0_f32 {
            Statepoint::Nonterminal((
                0.0_f32,
                input.panic_level+1.0_f32
            ))
        } else {
            let hypdist = input.dx.hypot(input.dy);
            let canonicalized_diff = relative_facing(
                input.facing.to_radians().sin_cos(), 
                (input.dx/hypdist, input.dy/hypdist)
            );
            if canonicalized_diff.abs() < 1.0_f32 {
                Statepoint::Nonterminal((
                    -canonicalized_diff,
                    input.panic_level+1.0_f32
                ))
            } else {
                Statepoint::Nonterminal((
                    -canonicalized_diff.signum(),
                    input.panic_level+1.0_f32
                ))
            }
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct BoidRecenter;

impl WaitCondition for BoidRecenter {
    type Input = RawInfo;
    type Nonterminal = (f32, f32);
    type Terminal = ();

    fn do_end(&self, info: &RawInfo) -> Statepoint<(f32, f32), ()> {
        assert!(info.about_self.xpos != 0.0 || info.about_self.ypos != 0.0);
        //The probability of turning a particular direction is 
        //guided by the angle between current facing and 
        let diff_dist = info.about_self.xpos.hypot(info.about_self.ypos);
        let offset = relative_facing(
            info.about_self.facing.to_radians().sin_cos(),
            (-info.about_self.xpos/diff_dist, -info.about_self.ypos/diff_dist)
        );
        if offset.abs() < 60.0 {
            Statepoint::Nonterminal((
                0.0_f32,
                info.panic_level*(1.0_f32-PANIC_RETURNING_DECAY)
            ))
        } else {
            Statepoint::Nonterminal((
                offset.signum(),
                info.panic_level*(1.0_f32-PANIC_RETURNING_DECAY)
            ))
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct BoidPanic;

impl WaitCondition for BoidPanic {
    type Input = RawInfo;
    type Nonterminal = (f32, f32);
    type Terminal = ();

    // Calm down once no boids are in sight, otherwise turn erratically and 
    // avoid facing the same way as other boids
    fn do_end(&self, input: &RawInfo) -> Statepoint<(f32, f32), ()> {
        let (prob_left, increment) = match input.about_crowd.len() {
            0 ... 1 => (0.5_f32, 0.0_f32),
            _ => {
                let mut running_x_sum = 0.0_f32;
                let mut running_y_sum = 0.0_f32;
                for entry in input.about_crowd.iter() {
                    running_x_sum += entry.xpos - input.about_self.xpos;
                    running_y_sum += entry.ypos - input.about_self.ypos;
                }
                let dx = running_x_sum;
                let dy = running_y_sum;
                if dx == 0.0_f32 && dy == 0.0_f32 {
                    (0.5_f32, 1.0_f32)
                } else {
                    let facing_sin_cos = input.about_self.facing.to_radians().sin_cos();
                    //The probability of turning a particular direction is 
                    //guided by the angle between current facing and 
                    let diff_dist = dx.hypot(dy);
                    let norm_dx = dx/diff_dist;
                    let norm_dy = dy/diff_dist;
                    (
                        (facing_sin_cos.0*norm_dy-facing_sin_cos.1*norm_dx+1.5_f32)/3.0_f32,
                        1.0_f32
                    )
                }
            }
        };
        let mut rng = rand::thread_rng();
        if rng.gen_bool(prob_left as f64) {
            Statepoint::Nonterminal((
                -1.0_f32,
                input.panic_level*(1.0_f32-PANIC_SEEKING_DECAY) + increment
            ))
        } else {
            Statepoint::Nonterminal((
                1.0_f32,
                input.panic_level*(1.0_f32-PANIC_SEEKING_DECAY) + increment
            ))
        }
    }
}
*/