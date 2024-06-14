pub struct Program {
    program: Vec<usize>,
    instruction_pointer: usize,
}

enum OpCode {
    Add,
    Multiply,
    Halt,
}

pub enum InstructionResult {
    Ok(usize), //Contains the increment to the instruction pointer
    Halt,
}

struct Instruction {
    op_code: OpCode,
    parameters: Vec<usize>,
}

impl Instruction {
    //Instructions know how to build themselves from the program fragment starting at the beginning
    //of the instruction (the number of parameters to extract depends on the op code which is
    //encapsulated in the Instruction struct/impl).
    pub fn new(program_fragment: &[usize]) -> Instruction {
        let op_code = Instruction::value_to_op_code(program_fragment[0]);
        let parameters: Vec<usize> = match op_code {
            OpCode::Add => {
                //Add currently takes 3 parameters
                program_fragment[1..4].to_vec()
            }
            OpCode::Multiply => {
                //Multiply currently takes 3 parameters
                program_fragment[1..4].to_vec()
            }
            OpCode::Halt => {
                //Halt takes no parameters
                vec![]
            }
        };
        Instruction {
            op_code,
            parameters,
        }
    }

    // Operate performs the relevant operation on operands and returns Ok or Halt
    pub fn execute(&mut self, program: &mut [usize]) -> InstructionResult {
        match self.op_code {
            OpCode::Add => self.do_op(program, OpCode::Add),
            OpCode::Multiply => self.do_op(program, OpCode::Multiply),
            OpCode::Halt => InstructionResult::Halt,
        }
    }

    // Add and multiply are common up to the aggregation function used on the iterator, so
    // for now can combine in a simple function do_op. Might reinstate do_add, do_multiply in future if necessary.

    // fn do_add(&mut self, program: &mut Vec<usize>) -> InstructionResult {
    //     let output_location = self.parameters.pop().unwrap();
    //     program[output_location] = self.parameters.iter().map(|x| program[*x]).sum();
    //     // Parameters vec currently only contains the indices of program elements to add together;
    //     // the instruction also contained the op code and the output index, so need to add 2 to
    //     // increment the instruction pointer by the correct amount
    //     InstructionResult::Ok(self.parameters.len() + 2)
    // }

    // fn do_multiply(&mut self, program: &mut Vec<usize>) -> InstructionResult {
    //     let output_location = self.parameters.pop().unwrap();
    //     program[output_location] = self.parameters.iter().map(|x| program[*x]).product();
    //     // Parameters vec currently only contains the indices of program elements to add together;
    //     // the instruction also contained the op code and the output index, so need to add 2 to
    //     // increment the instruction pointer by the correct amount
    //     InstructionResult::Ok(self.parameters.len() + 2)
    // }

    fn do_op(&mut self, program: &mut [usize], op: OpCode) -> InstructionResult {
        let output_location = self.parameters.pop().unwrap();
        let operands = self.parameters.iter().map(|x| program[*x]);
        program[output_location] = match op {
            OpCode::Add => operands.sum(),
            OpCode::Multiply => operands.product(),
            _ => panic!("Only currently valid for Add and Multiply"),
        };
        // Parameters vec currently only contains the indices of program elements to add together;
        // the instruction also contained the op code and the output index, so need to add 2 to
        // increment the instruction pointer by the correct amount
        InstructionResult::Ok(self.parameters.len() + 2)
    }

    fn value_to_op_code(op_code: usize) -> OpCode {
        match op_code {
            1 => OpCode::Add,
            2 => OpCode::Multiply,
            99 => OpCode::Halt,
            _ => panic!("Op Code not implemented!"),
        }
    }
}

impl Program {
    pub fn new(program: &[usize]) -> Program {
        Program {
            program: program.to_owned(),
            instruction_pointer: 0,
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut instruction = Instruction::new(&self.program[self.instruction_pointer..]);
            match instruction.execute(&mut self.program) {
                InstructionResult::Ok(x) => {
                    self.instruction_pointer += x;
                    continue;
                }
                InstructionResult::Halt => {
                    self.instruction_pointer += 1;
                    break;
                }
            };
        }
    }

    pub fn set_inputs(&mut self, noun: usize, verb: usize) {
        // Happy for this to panic - indices 1 and 2 should always be present
        self.program[1] = noun;
        self.program[2] = verb;
    }

    pub fn output(&self) -> usize {
        self.get_value_at(0)
    }

    pub fn get_value_at(&self, index: usize) -> usize {
        self.program[index]
    }

    pub fn initialize(&mut self, initial_values: &[usize]) {
        self.program = initial_values.to_owned();
        self.instruction_pointer = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_programs() {
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
}
