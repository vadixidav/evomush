use gapush;

use gapush::simple::{Chromosome, SimpleInstruction, PlainOp};
use rand::Rng;
use rand::distributions::{Exp, IndependentSample};

const DEFAULT_LAMBDA: f64 = 8192.0;
const LAMBDA_SELF_POINT: f64 = 512.0;
const MAXIMUM_MUTATES: usize = 1024;

const INIT_EXECUTION_TIME: usize = 512;
const INIT_LEN: usize = 128;
const INIT_CROSSOVERS: usize = 4;
const CYCLE_LEN: usize = 128;
const CYCLE_CROSSOVERS: usize = 4;
const CONNECTION_ELASTICITY_LEN: usize = 128;
const CONNECTION_ELASTICITY_CROSSOVERS: usize = 4;
const CONNECTION_SIGNAL_LEN: usize = 128;
const CONNECTION_SIGNAL_CROSSOVERS: usize = 4;
const CONNECTION_SEVER_LEN: usize = 128;
const CONNECTION_SEVER_CROSSOVERS: usize = 4;
const REPULSION_LEN: usize = 128;
const REPULSION_CROSSOVERS: usize = 4;
const DIE_LEN: usize = 128;
const DIE_CROSSOVERS: usize = 4;
const DIVIDE_LEN: usize = 128;
const DIVIDE_CROSSOVERS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    /// Runs to initialize the cell; this ignores any yielded instructions.
    init: Chromosome,
    /// Runs each cycle before anything else; is passed the cell's energy on the float stack.
    cycle: Chromosome,
    /// Determines connection elasticity; this is passed the connection instruction on the instruction stack
    /// and the length on the float stack.
    connection_elasticity: Chromosome,
    /// Determines connection signal; this is to be executed directly after connection_elasticity.
    connection_signal: Chromosome,
    /// Determines if the connection will be severed; this is to be executed directly after connection_signal.
    connection_sever: Chromosome,
    /// Doesn't pass anything, but tries to get an i64 back which indicates the cell repulsion magnitude.
    repulsion: Chromosome,
    /// Doesn't pass anything, but tries to get a bool which determines if the cell should die.
    die: Chromosome,
    /// Doesn't pass anything, but tries to get a bool which determines if the cell should divide.
    divide: Chromosome,

    lambda: f64,
}

impl Genome {
    fn new_rand<R: Rng>(rng: &mut R) -> Genome {
        Genome {
            init: Chromosome::new_rand(rng, INIT_LEN, INIT_CROSSOVERS),
            cycle: Chromosome::new_rand(rng, CYCLE_LEN, CYCLE_CROSSOVERS),
            connection_elasticity: Chromosome::new_rand(rng,
                                                        CONNECTION_ELASTICITY_LEN,
                                                        CONNECTION_ELASTICITY_CROSSOVERS),
            connection_signal: Chromosome::new_rand(rng,
                                                    CONNECTION_SIGNAL_LEN,
                                                    CONNECTION_SIGNAL_CROSSOVERS),
            connection_sever: Chromosome::new_rand(rng,
                                                   CONNECTION_SEVER_LEN,
                                                   CONNECTION_SEVER_CROSSOVERS),
            repulsion: Chromosome::new_rand(rng, REPULSION_LEN, REPULSION_CROSSOVERS),
            die: Chromosome::new_rand(rng, DIE_LEN, DIE_CROSSOVERS),
            divide: Chromosome::new_rand(rng, DIVIDE_LEN, DIVIDE_CROSSOVERS),
            lambda: DEFAULT_LAMBDA,
        }
    }

    fn mutate<R: Rng>(&mut self, rng: &mut R) {
        let exp = Exp::new(self.lambda);
        if exp.ind_sample(rng) < LAMBDA_SELF_POINT {
            if rng.gen_range(0, 2usize) == 0 {
                self.lambda += 1.0;
            } else {
                if self.lambda > 2.0 {
                    self.lambda -= 1.0;
                }
            }
        }

        self.init.mutate(MAXIMUM_MUTATES, &exp, rng);
        self.cycle.mutate(MAXIMUM_MUTATES, &exp, rng);
        self.connection_elasticity
            .mutate(MAXIMUM_MUTATES, &exp, rng);
        self.connection_signal.mutate(MAXIMUM_MUTATES, &exp, rng);
        self.connection_sever.mutate(MAXIMUM_MUTATES, &exp, rng);
        self.repulsion.mutate(MAXIMUM_MUTATES, &exp, rng);
        self.die.mutate(MAXIMUM_MUTATES, &exp, rng);
        self.divide.mutate(MAXIMUM_MUTATES, &exp, rng);
    }

    fn mate(&self, other: &Self) -> Self {
        Genome {
            init: self.init.mate(&other.init),
            cycle: self.cycle.mate(&other.cycle),
            connection_elasticity: self.connection_elasticity
                .mate(&other.connection_elasticity),
            connection_signal: self.connection_signal.mate(&other.connection_signal),
            connection_sever: self.connection_sever.mate(&other.connection_sever),
            repulsion: self.repulsion.mate(&other.repulsion),
            die: self.die.mate(&other.die),
            divide: self.divide.mate(&other.divide),
            lambda: (self.lambda + other.lambda) * 0.5,
        }
    }

    /// Gets the size which is left over after considering the size of the genome.
    fn leftover_size_from(&self, size: usize) -> usize {
        size.checked_sub(self.init.gene_len())
            .and_then(|n| n.checked_sub(self.cycle.gene_len()))
            .and_then(|n| n.checked_sub(self.connection_elasticity.gene_len()))
            .and_then(|n| n.checked_sub(self.connection_signal.gene_len()))
            .and_then(|n| n.checked_sub(self.connection_sever.gene_len()))
            .and_then(|n| n.checked_sub(self.repulsion.gene_len()))
            .and_then(|n| n.checked_sub(self.die.gene_len()))
            .and_then(|n| n.checked_sub(self.divide.gene_len()))
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
pub struct Brain {
    genome: Genome,
    machine:
        gapush::Machine<SimpleInstruction, fn() -> SimpleInstruction, fn() -> i64, fn() -> f64>,
}

impl Brain {
    pub fn new_rand<R: Rng>(max_size: usize, rng: &mut R) -> Brain {
        let genome = Genome::new_rand(rng);
        let mut machine = gapush::Machine::new(max_size,
                                               instruction_handler as fn() -> SimpleInstruction,
                                               int_handler as fn() -> i64,
                                               float_handler as fn() -> f64);
        // Execute the initialization routine.
        machine.provide_and_cycle_until(INIT_EXECUTION_TIME, (&genome.init).into());
        Brain {
            genome: genome,
            machine: machine,
        }
    }

    pub fn mate(&self, other: &Self, child_max_size: usize) -> Brain {
        let genome = self.genome.mate(&other.genome);
        let mut machine = gapush::Machine::new(child_max_size,
                                               instruction_handler as fn() -> SimpleInstruction,
                                               int_handler as fn() -> i64,
                                               float_handler as fn() -> f64);
        // Execute the initialization routine.
        machine.provide_and_cycle_until(INIT_EXECUTION_TIME, (&genome.init).into());
        Brain {
            genome: genome,
            machine: machine,
        }
    }

    pub fn mutate<R: Rng>(&mut self, rng: &mut R) {
        self.genome.mutate(rng);
    }

    pub fn set_size(&mut self, size: usize) {
        self.machine.state.max_size = self.genome.leftover_size_from(size);
    }

    /// Runs the cycle and returns the number of cycles executed.
    pub fn run_cycle(&mut self, energy: f64) -> usize {
        self.machine.state.push_float(energy).ok();
        self.machine
            .provide_and_cycle_until(INIT_EXECUTION_TIME, (&self.genome.cycle).into())
            .1
    }

    /// Runs the connection chromosomes. Gives back an i64 that corresponds to the desired elasticity and
    /// an instruction corresponding to the signal.
    pub fn run_connection(&mut self,
                          length: f64,
                          ins: SimpleInstruction)
                          -> (Option<i64>, Option<SimpleInstruction>, Option<bool>, usize) {
        self.machine.state.push_float(length).ok();
        self.machine.state.push_ins(ins).ok();
        let (elasticity, elen) =
            self.machine
                .provide_and_cycle_until(INIT_EXECUTION_TIME,
                                         (&self.genome.connection_elasticity).into());
        let (signal, slen) =
            self.machine
                .provide_and_cycle_until(INIT_EXECUTION_TIME,
                                         (&self.genome.connection_signal).into());
        let (sever, svlen) =
            self.machine
                .provide_and_cycle_until(INIT_EXECUTION_TIME,
                                         (&self.genome.connection_sever).into());
        (elasticity.and_then(|ins| match ins {
                                 SimpleInstruction::Pushi64(n) => Some(n),
                                 _ => None,
                             }),
         signal,
         sever.and_then(|ins| match ins {
                            SimpleInstruction::Pushb(b) => Some(b),
                            _ => None,
                        }),
         elen + slen + svlen)
    }

    /// Runs the repulsion chromosome. Gets an i64 back that indicates the desired repulsion.
    pub fn run_repulsion(&mut self) -> (Option<i64>, usize) {
        let (repulsion, len) =
            self.machine
                .provide_and_cycle_until(INIT_EXECUTION_TIME, (&self.genome.repulsion).into());
        (repulsion.and_then(|ins| match ins {
                                SimpleInstruction::Pushi64(n) => Some(n),
                                _ => None,
                            }),
         len)
    }

    /// Runs the die chromosome. Gets a bool back that indicates whether to die or not.
    pub fn run_die(&mut self) -> (Option<bool>, usize) {
        let (die, len) =
            self.machine
                .provide_and_cycle_until(INIT_EXECUTION_TIME, (&self.genome.die).into());
        (die.and_then(|ins| match ins {
                          SimpleInstruction::Pushb(b) => Some(b),
                          _ => None,
                      }),
         len)
    }

    /// Runs the divide chromosome. Gets a bool back that indicates whether to divide or not.
    pub fn run_divide(&mut self) -> (Option<bool>, usize) {
        let (divide, len) =
            self.machine
                .provide_and_cycle_until(INIT_EXECUTION_TIME, (&self.genome.divide).into());
        (divide.and_then(|ins| match ins {
                             SimpleInstruction::Pushb(b) => Some(b),
                             _ => None,
                         }),
         len)
    }
}

fn instruction_handler() -> SimpleInstruction {
    SimpleInstruction::PlainOp(PlainOp::Nop)
}

fn int_handler() -> i64 {
    0
}

fn float_handler() -> f64 {
    0.0
}
