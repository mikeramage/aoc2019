pub struct Program {
    program: Vec<i32>,
    instruction_pointer: usize,
    input: i32,
    outputs: Vec<i32>,
}

#[derive(Debug)]
enum OpCode {
    Add,
    Multiply,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equals,
    Halt,
}

impl TryFrom<i32> for OpCode {
    type Error = String;

    fn try_from(d: i32) -> Result<Self, Self::Error> {
        use OpCode::*;

        //Take the rightmost 2 digits of d.
        let x = d % 10 + 10 * ((d / 10) % 10);
        match x {
            1 => Ok(Add),
            2 => Ok(Multiply),
            3 => Ok(Input),
            4 => Ok(Output),
            5 => Ok(JumpIfTrue),
            6 => Ok(JumpIfFalse),
            7 => Ok(LessThan),
            8 => Ok(Equals),
            99 => Ok(Halt),
            _ => Err(format!("Invalid OpCode '{}'", x)),
        }
    }
}

pub enum InstructionResult {
    OkIncrement(usize), //Contains the increment to the instruction pointer
    OkSet(usize),       //Contains absolute value for instruction pointer
    OutputIncrement(i32, usize),
    Halt,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode {
    Position,
    Immediate,
}

impl TryFrom<i32> for Mode {
    type Error = String;

    fn try_from(d: i32) -> Result<Self, Self::Error> {
        use Mode::*;

        match d {
            0 => Ok(Position),
            1 => Ok(Immediate),
            _ => Err(format!("Invalid Mode '{}'", d)),
        }
    }
}

struct Parameter {
    mode: Mode,
    value: i32,
}

impl Parameter {
    fn new(parameter: (&Mode, &i32)) -> Parameter {
        Parameter {
            mode: *parameter.0,
            value: *parameter.1,
        }
    }

    fn get_effective_value(&self, program: &[i32]) -> i32 {
        match self.mode {
            Mode::Position => program[self.value as usize],
            Mode::Immediate => self.value,
        }
    }
}

struct Instruction {
    op_code: OpCode,
    parameters: Vec<Parameter>,
    input: i32,
}

impl Instruction {
    //Instructions know how to build themselves from the program fragment starting at the beginning
    //of the instruction (the number of parameters to extract depends on the op code which is
    //encapsulated in the Instruction struct/impl).
    pub fn new(program_fragment: &[i32], input: i32) -> Instruction {
        use OpCode::*;
        let op_code = OpCode::try_from(program_fragment[0]).unwrap();
        let parameters: Vec<Parameter> = match op_code {
            Input | Output => {
                //Input and output take a single parameter
                extract_parameters(program_fragment, 1)
            }
            JumpIfTrue | JumpIfFalse => {
                //Jump take 2 parameters
                extract_parameters(program_fragment, 2)
            }
            Add | Multiply | LessThan | Equals => {
                //Add and multiply, less than, equals currently take 3 parameters
                extract_parameters(program_fragment, 3)
            }
            Halt => {
                //Halt takes no parameters
                vec![]
            }
        };
        Instruction {
            op_code,
            parameters,
            input,
        }
    }

    // Operate performs the relevant operation on operands and returns Ok or Halt
    pub fn execute(&mut self, program: &mut [i32]) -> InstructionResult {
        use OpCode::*;
        match self.op_code {
            Add => self.do_op(program, Add),
            Multiply => self.do_op(program, Multiply),
            Input => self.do_input(program),
            Output => self.do_output(program),
            JumpIfTrue => self.do_jump(program, true),
            JumpIfFalse => self.do_jump(program, false),
            LessThan => self.do_comparison(program, LessThan),
            Equals => self.do_comparison(program, Equals),
            Halt => InstructionResult::Halt,
        }
    }

    fn modes_from_instruction(instruction: i32, num_modes: usize) -> Vec<Mode> {
        let mut modes: Vec<Mode> = vec![];
        let mut x = instruction / 100;
        for _i in 0..num_modes {
            modes.push(Mode::try_from(x % 10).unwrap());
            x /= 10;
        }
        modes
    }

    fn do_op(&mut self, program: &mut [i32], op: OpCode) -> InstructionResult {
        let output_location = self.parameters.last().unwrap().value;
        let operands = self.parameters[0..(self.parameters.len() - 1)]
            .iter()
            .map(|x| x.get_effective_value(program));
        program[output_location as usize] = match op {
            OpCode::Add => operands.sum(),
            OpCode::Multiply => operands.product(),
            _ => panic!("Only currently valid for Add and Multiply"),
        };
        // Parameters vec currently only contains the indices of program elements to add together;
        // the instruction also contained the op code and the output index, so need to add 2 to
        // increment the instruction pointer by the correct amount
        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }

    fn do_input(&mut self, program: &mut [i32]) -> InstructionResult {
        let output_location = self.parameters[0].value;
        program[output_location as usize] = self.input;
        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }

    fn do_output(&mut self, program: &mut [i32]) -> InstructionResult {
        let value = self.parameters[0].get_effective_value(program);
        InstructionResult::OutputIncrement(value, self.parameters.len() + 1)
    }

    fn do_jump(&mut self, program: &mut [i32], jump_if_true: bool) -> InstructionResult {
        let do_jump = self.parameters[0].get_effective_value(program);

        if (jump_if_true && do_jump != 0) || (!jump_if_true && do_jump == 0) {
            let jump_to = self.parameters[1].get_effective_value(program);
            return InstructionResult::OkSet(jump_to as usize);
        }

        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }

    fn do_comparison(&mut self, program: &mut [i32], op_code: OpCode) -> InstructionResult {
        let output_location = self.parameters.last().unwrap().value;
        let first = self.parameters[0].get_effective_value(program);
        let second = self.parameters[1].get_effective_value(program);

        match op_code {
            OpCode::LessThan => {
                if first < second {
                    program[output_location as usize] = 1;
                } else {
                    program[output_location as usize] = 0;
                }
            }
            OpCode::Equals => {
                if first == second {
                    program[output_location as usize] = 1;
                } else {
                    program[output_location as usize] = 0;
                }
            }
            _ => panic!("Bad op code {:?}", op_code),
        }

        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }
}

fn extract_parameters(program_fragment: &[i32], num_parameters: usize) -> Vec<Parameter> {
    let modes: Vec<Mode> = Instruction::modes_from_instruction(program_fragment[0], num_parameters);
    modes
        .iter()
        .zip(program_fragment[1..(num_parameters + 1)].iter())
        .map(Parameter::new)
        .collect()
}

impl Program {
    pub fn new(program: &[i32]) -> Program {
        Program {
            program: program.to_owned(),
            instruction_pointer: 0,
            input: 0,
            outputs: vec![],
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut instruction =
                Instruction::new(&self.program[self.instruction_pointer..], self.input);
            match instruction.execute(&mut self.program) {
                InstructionResult::OkIncrement(increment) => {
                    self.instruction_pointer += increment;
                    continue;
                }
                InstructionResult::OutputIncrement(output, increment) => {
                    self.outputs.push(output);
                    self.instruction_pointer += increment;
                    continue;
                }
                InstructionResult::OkSet(address) => {
                    self.instruction_pointer = address;
                    continue;
                }
                InstructionResult::Halt => {
                    self.instruction_pointer += 1;
                    break;
                }
            };
        }
    }

    pub fn set_noun_verb_inputs(&mut self, noun: i32, verb: i32) {
        // Happy for this to panic - indices 1 and 2 should always be present
        self.program[1] = noun;
        self.program[2] = verb;
    }

    pub fn set_input(&mut self, input: i32) {
        self.input = input;
    }

    pub fn output(&self) -> i32 {
        self.get_value_at(0)
    }

    pub fn outputs(&self) -> &Vec<i32> {
        &self.outputs
    }

    pub fn get_value_at(&self, index: i32) -> i32 {
        self.program[index as usize]
    }

    pub fn initialize(&mut self, initial_values: &[i32]) {
        self.program = initial_values.to_owned();
        self.instruction_pointer = 0;
        self.input = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_simple_programs() {
        let mut program = Program::new(&vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50]);
        program.run();
        assert_eq!(3500, program.output());

        program = Program::new(&vec![1, 0, 0, 0, 99]);
        program.run();
        assert_eq!(2, program.output());

        program = Program::new(&vec![2, 3, 0, 3, 99]);
        program.run();
        assert_eq!(6, program.get_value_at(3));

        program = Program::new(&vec![2, 4, 4, 5, 99, 0]);
        program.run();
        assert_eq!(9801, program.get_value_at(5));

        program = Program::new(&vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        program.run();
        assert_eq!(30, program.output());
        assert_eq!(2, program.get_value_at(4));
    }

    #[test]
    fn test_input_output_and_modes() {
        let mut program = Program::new(&vec![1002, 4, 3, 4, 33]);
        program.run();
        assert_eq!(99, program.get_value_at(4));

        program = Program::new(&vec![3, 0, 4, 0, 99]);
        program.set_input(-5);
        program.run();
        assert_eq!(-5, *program.outputs().last().unwrap());
    }

    #[test]
    fn test_equals_less_than() {
        let mut program = Program::new(&vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.set_input(8);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.set_input(-3);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.set_input(6);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.set_input(33);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1108, -1, 8, 3, 4, 3, 99]);
        program.set_input(8);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1108, -1, 8, 3, 4, 3, 99]);
        program.set_input(120);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1107, -1, 8, 3, 4, 3, 99]);
        program.set_input(-99);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1107, -1, 8, 3, 4, 3, 99]);
        program.set_input(8);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());
    }

    #[test]
    fn test_jumps() {
        let mut program = Program::new(&vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);
        program.set_input(0);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);
        program.set_input(28);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);
        program.set_input(0);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);
        program.set_input(-12);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());
    }

    #[test]
    fn test_complicated_jumps_and_stuff() {
        let mut program = Program::new(&vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);
        program.set_input(0);
        program.run();
        assert_eq!(999, *program.outputs().last().unwrap());

        program = Program::new(&vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);
        program.set_input(8);
        program.run();
        assert_eq!(1000, *program.outputs().last().unwrap());

        program = Program::new(&vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);
        program.set_input(282);
        program.run();
        assert_eq!(1001, *program.outputs().last().unwrap());
    }
}
