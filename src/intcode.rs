use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub struct Program {
    program: Vec<isize>,
    memory: HashMap<isize, isize>,
    instruction_pointer: usize,
    inputs: VecDeque<isize>,
    outputs: Vec<isize>,
    relative_base: isize,
}

impl Program {
    pub fn new(program: &[isize]) -> Program {
        Program {
            program: program.to_owned(),
            memory: HashMap::new(),
            instruction_pointer: 0,
            inputs: VecDeque::new(),
            outputs: vec![],
            relative_base: 0,
        }
    }

    pub fn run(&mut self) -> ProgramResult {
        let mut result = ProgramResult::Halted;
        loop {
            let mut instruction =
                Instruction::new(&self.program[self.instruction_pointer..], &mut self.inputs);
            match instruction.execute(self) {
                InstructionResult::OkIncrement(increment) => {
                    self.instruction_pointer += increment;
                }
                InstructionResult::OutputIncrement(output, increment) => {
                    self.outputs.push(output);
                    self.instruction_pointer += increment;
                }
                InstructionResult::OkSet(address) => {
                    self.instruction_pointer = address;
                }
                InstructionResult::OkRelativeBaseIncrement(base_increment, pointer_increment) => {
                    self.relative_base += base_increment;
                    self.instruction_pointer += pointer_increment;
                }
                InstructionResult::AwaitInput => {
                    result = ProgramResult::AwaitingInput;
                    break;
                }
                InstructionResult::Halt => {
                    self.instruction_pointer += 1;
                    break;
                }
            };
        }

        result
    }

    pub fn set_noun_verb_inputs(&mut self, noun: isize, verb: isize) {
        // Happy for this to panic - indices 1 and 2 should always be present
        self.program[1] = noun;
        self.program[2] = verb;
    }

    pub fn add_input(&mut self, input: isize) {
        self.inputs.push_back(input);
    }

    pub fn output_deprecated(&self) -> isize {
        self.get_value_at(0)
    }

    pub fn outputs(&self) -> &Vec<isize> {
        &self.outputs
    }

    pub fn clear_outputs(&mut self) {
        self.outputs = vec![];
    }

    pub fn remove_last_output(&mut self) -> Option<isize> {
        self.outputs.pop()
    }

    pub fn get_value_at(&self, index: isize) -> isize {
        if (index as usize) < self.program.len() {
            self.program[index as usize]
        } else {
            //Get it from memory
            *self.memory.get(&index).unwrap_or(&0)
        }
    }

    pub fn set_value_at(&mut self, index: isize, value: isize) {
        if (index as usize) < self.program.len() {
            self.program[index as usize] = value;
        } else {
            //Set it in memory
            self.memory.insert(index, value);
        }
    }

    pub fn initialize(&mut self, initial_values: &[isize]) {
        initial_values.clone_into(&mut self.program);
        self.memory = HashMap::new();
        self.instruction_pointer = 0;
        self.inputs = VecDeque::new();
        self.outputs = vec![];
        self.relative_base = 0;
    }
}

#[derive(Debug, PartialEq)]
pub enum ProgramResult {
    AwaitingInput,
    Halted,
}

struct Instruction {
    op_code: OpCode,
    parameters: Vec<Parameter>,
    input: Option<isize>, //Only relevant if OpCode is Input
}

impl Instruction {
    //Instructions know how to build themselves from the program fragment starting at the beginning
    //of the instruction (the number of parameters to extract depends on the op code which is
    //encapsulated in the Instruction struct/impl).
    pub fn new(program_fragment: &[isize], inputs: &mut VecDeque<isize>) -> Instruction {
        use OpCode::*;
        let op_code = OpCode::try_from(program_fragment[0]).unwrap();
        let parameters: Vec<Parameter> =
            Instruction::extract_parameters(program_fragment, &op_code);

        let input = match op_code {
            Input => {
                if !inputs.is_empty() {
                    Some(inputs.pop_front().expect("Expected sufficient inputs!"))
                } else {
                    None
                }
            }
            _ => None,
        };

        Instruction {
            op_code,
            parameters,
            input,
        }
    }

    fn extract_parameters(program_fragment: &[isize], op_code: &OpCode) -> Vec<Parameter> {
        use OpCode::*;

        let num_parameters = match op_code {
            Input | Output | RelativeBaseOffset => 1,
            JumpIfTrue | JumpIfFalse => 2,
            Add | Multiply | LessThan | Equals => 3,
            Halt => 0,
        };

        if let Halt = op_code {
            vec![]
        } else {
            std::iter::repeat_with({
                let mut mode_digits = program_fragment[0] / 100;
                move || {
                    let mode = mode_digits % 10;
                    mode_digits /= 10;
                    Mode::try_from(mode).unwrap()
                }
            })
            .take(num_parameters)
            .zip(program_fragment[1..(num_parameters + 1)].iter())
            .map(Parameter::new)
            .collect()
        }
    }

    // Operate performs the relevant operation on operands and returns Ok or Halt
    pub fn execute(&mut self, program: &mut Program) -> InstructionResult {
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
            RelativeBaseOffset => self.do_relative_base(program),
            Halt => InstructionResult::Halt,
        }
    }

    fn do_op(&mut self, program: &mut Program, op: OpCode) -> InstructionResult {
        let mut output_location = self.parameters.last().unwrap().value;
        if let Mode::Relative = self.parameters.last().unwrap().mode {
            output_location += program.relative_base;
        }
        let operands = self.parameters[0..(self.parameters.len() - 1)]
            .iter()
            .map(|x| x.mode_adjusted_value(program));
        program.set_value_at(
            output_location,
            match op {
                OpCode::Add => operands.sum(),
                OpCode::Multiply => operands.product(),
                _ => panic!("Only currently valid for Add and Multiply"),
            },
        );
        // Parameters vec currently only contains the indices of program elements to add together;
        // the instruction also contained the op code and the output index, so need to add 2 to
        // increment the instruction pointer by the correct amount
        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }

    fn do_input(&mut self, program: &mut Program) -> InstructionResult {
        let mut output_location = self.parameters[0].value;
        if let Mode::Relative = self.parameters[0].mode {
            output_location += program.relative_base;
        }
        match self.input {
            Some(x) => {
                program.set_value_at(output_location, x);
                InstructionResult::OkIncrement(self.parameters.len() + 1)
            }
            None => InstructionResult::AwaitInput,
        }
    }

    fn do_output(&mut self, program: &mut Program) -> InstructionResult {
        let value = self.parameters[0].mode_adjusted_value(program);
        InstructionResult::OutputIncrement(value, self.parameters.len() + 1)
    }

    fn do_relative_base(&mut self, program: &mut Program) -> InstructionResult {
        let value = self.parameters[0].mode_adjusted_value(program);
        InstructionResult::OkRelativeBaseIncrement(value, self.parameters.len() + 1)
    }

    fn do_jump(&mut self, program: &mut Program, jump_if_true: bool) -> InstructionResult {
        let do_jump = self.parameters[0].mode_adjusted_value(program);

        if (jump_if_true && do_jump != 0) || (!jump_if_true && do_jump == 0) {
            let jump_to = self.parameters[1].mode_adjusted_value(program);
            return InstructionResult::OkSet(jump_to as usize);
        }

        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }

    fn do_comparison(&mut self, program: &mut Program, op_code: OpCode) -> InstructionResult {
        let mut output_location = self.parameters.last().unwrap().value;
        if let Mode::Relative = self.parameters.last().unwrap().mode {
            output_location += program.relative_base;
        }
        let first = self.parameters[0].mode_adjusted_value(program);
        let second = self.parameters[1].mode_adjusted_value(program);

        match op_code {
            OpCode::LessThan => {
                if first < second {
                    program.set_value_at(output_location, 1);
                } else {
                    program.set_value_at(output_location, 0);
                }
            }
            OpCode::Equals => {
                if first == second {
                    program.set_value_at(output_location, 1);
                } else {
                    program.set_value_at(output_location, 0);
                }
            }
            _ => panic!("Bad op code {:?}", op_code),
        }

        InstructionResult::OkIncrement(self.parameters.len() + 1)
    }
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
    RelativeBaseOffset,
    Halt,
}

impl TryFrom<isize> for OpCode {
    type Error = String;

    fn try_from(d: isize) -> Result<Self, Self::Error> {
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
            9 => Ok(RelativeBaseOffset),
            99 => Ok(Halt),
            _ => Err(format!("Invalid OpCode '{}'", x)),
        }
    }
}

pub enum InstructionResult {
    OkIncrement(usize), //Contains the increment to the instruction pointer
    OkSet(usize),       //Contains absolute value for instruction pointer
    OutputIncrement(isize, usize),
    AwaitInput,
    OkRelativeBaseIncrement(isize, usize),
    Halt,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode {
    Position,
    Immediate,
    Relative,
}

impl TryFrom<isize> for Mode {
    type Error = String;

    fn try_from(d: isize) -> Result<Self, Self::Error> {
        use Mode::*;

        match d {
            0 => Ok(Position),
            1 => Ok(Immediate),
            2 => Ok(Relative),
            _ => Err(format!("Invalid Mode '{}'", d)),
        }
    }
}

struct Parameter {
    mode: Mode,
    value: isize,
}

impl Parameter {
    fn new(parameter: (Mode, &isize)) -> Parameter {
        Parameter {
            mode: parameter.0,
            value: *parameter.1,
        }
    }

    fn mode_adjusted_value(&self, program: &Program) -> isize {
        match self.mode {
            Mode::Position => program.get_value_at(self.value),
            Mode::Immediate => self.value,
            Mode::Relative => program.get_value_at(self.value + program.relative_base),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_simple_programs() {
        let mut program = Program::new(&vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50]);
        program.run();
        assert_eq!(3500, program.output_deprecated());

        program = Program::new(&vec![1, 0, 0, 0, 99]);
        program.run();
        assert_eq!(2, program.output_deprecated());

        program = Program::new(&vec![2, 3, 0, 3, 99]);
        program.run();
        assert_eq!(6, program.get_value_at(3));

        program = Program::new(&vec![2, 4, 4, 5, 99, 0]);
        program.run();
        assert_eq!(9801, program.get_value_at(5));

        program = Program::new(&vec![1, 1, 1, 4, 99, 5, 6, 0, 99]);
        program.run();
        assert_eq!(30, program.output_deprecated());
        assert_eq!(2, program.get_value_at(4));
    }

    #[test]
    fn test_input_output_and_modes() {
        let mut program = Program::new(&vec![1002, 4, 3, 4, 33]);
        program.run();
        assert_eq!(99, program.get_value_at(4));

        program = Program::new(&vec![3, 0, 4, 0, 99]);
        program.add_input(-5);
        program.run();
        assert_eq!(-5, *program.outputs().last().unwrap());
    }

    #[test]
    fn test_equals_less_than() {
        let mut program = Program::new(&vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.add_input(8);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.add_input(-3);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.add_input(6);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8]);
        program.add_input(33);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1108, -1, 8, 3, 4, 3, 99]);
        program.add_input(8);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1108, -1, 8, 3, 4, 3, 99]);
        program.add_input(120);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1107, -1, 8, 3, 4, 3, 99]);
        program.add_input(-99);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1107, -1, 8, 3, 4, 3, 99]);
        program.add_input(8);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());
    }

    #[test]
    fn test_jumps() {
        let mut program = Program::new(&vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);
        program.add_input(0);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![
            3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9,
        ]);
        program.add_input(28);
        program.run();
        assert_eq!(1, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);
        program.add_input(0);
        program.run();
        assert_eq!(0, *program.outputs().last().unwrap());

        program = Program::new(&vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1]);
        program.add_input(-12);
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
        program.add_input(0);
        program.run();
        assert_eq!(999, *program.outputs().last().unwrap());

        program = Program::new(&vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);
        program.add_input(8);
        program.run();
        assert_eq!(1000, *program.outputs().last().unwrap());

        program = Program::new(&vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ]);
        program.add_input(282);
        program.run();
        assert_eq!(1001, *program.outputs().last().unwrap());
    }
}
