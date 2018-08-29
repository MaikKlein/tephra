use rspirv::binary::Parser;
use rspirv::mr::{Instruction, Loader, Module, Operand};
use spirv_headers as spirv;
macro_rules! extract {
    ($val:expr, $name:path) => {
        match $val {
            $name(inner) => inner,
            _ => panic!("Extract failed"),
        }
    };
}

fn find_result_id<'a>(module: &'a Module, id: u32) -> Option<&'a Instruction> {
    module.global_inst_iter().find(|inst| match inst.result_id {
        Some(result_id) => result_id == id,
        _ => false,
    })
}
fn filter_ids<'a>(
    module: &'a Module,
    operands: &'a [Operand],
) -> impl Iterator<Item = &'a Instruction> {
    operands
        .iter()
        .map(|operand| extract!(operand, Operand::IdRef))
        .filter_map(move |&id| find_result_id(module, id))
}

pub fn reflect(bytes: &[u8]) {
    let mut loader = Loader::new();
    {
        let p = Parser::new(&bytes, &mut loader);
        p.parse().unwrap();
    }
    let module = loader.module();
    let entry_points: Vec<_> = module
        .entry_points
        .iter()
        .filter_map(|inst| EntryPoint::from_instruction(&module, inst))
        .collect();
    println!("{:#?}", entry_points);
}

#[derive(Debug, Clone)]
pub enum Type {
    F32,
    Vec(Box<Type>, u32),
    Pointer(Box<Type>),
    Struct(Vec<Type>),
    Array(Box<Type>),
}
impl Type {
    pub fn from_instruction(module: &Module, ty_inst: &Instruction) -> Type {
        match ty_inst.class.opcode {
            spirv::Op::TypeFloat => {
                let size = *extract!(&ty_inst.operands[0], Operand::LiteralInt32);
                match size {
                    32 => Type::F32,
                    _ => unimplemented!(),
                }
            }
            spirv::Op::TypePointer => {
                let id = *extract!(&ty_inst.operands[1], Operand::IdRef);
                let inner_ty_inst = find_result_id(module, id).expect("Should have Inner Type");
                Type::Pointer(Box::new(Type::from_instruction(module, inner_ty_inst)))
            }
            spirv::Op::TypeVector => {
                let id = *extract!(&ty_inst.operands[0], Operand::IdRef);
                // FIXME: Possbile other variants here?
                let dim = *extract!(&ty_inst.operands[1], Operand::LiteralInt32);
                let inner_ty_inst = find_result_id(module, id).expect("Should have Inner Type");
                Type::Vec(Box::new(Type::from_instruction(module, inner_ty_inst)), dim)
            }
            spirv::Op::TypeArray => {
                let type_id = *extract!(&ty_inst.operands[0], Operand::IdRef);
                let size_id = *extract!(&ty_inst.operands[1], Operand::IdRef);
                let size_inst =
                    find_result_id(module, size_id).expect("Should have Inner Type");
                println!("{:?}", size_inst);
                let inner_ty_inst =
                    find_result_id(module, type_id).expect("Should have Inner Type");
                Type::Array(Box::new(Type::from_instruction(module, inner_ty_inst)))
            }
            spirv::Op::TypeStruct => {
                let types: Vec<_> = ty_inst
                    .operands
                    .iter()
                    .map(|operand| {
                        let id = *extract!(operand, Operand::IdRef);
                        let inst = find_result_id(module, id).expect("Unable to find type");
                        Type::from_instruction(module, inst)
                    })
                    .collect();
                Type::Struct(types)
            }
            r => unimplemented!("{:?}", r),
        }
    }
}
#[derive(Debug)]
pub struct Variable {
    pub storage_class: spirv::StorageClass,
    pub ty: Type,
}
impl Variable {
    pub fn from_instruction(module: &Module, inst: &Instruction) -> Option<Self> {
        if inst.class.opcode != spirv::Op::Variable {
            return None;
        }
        let result_type = inst.result_type.expect("Variable hould have a type");
        let ty = Type::from_instruction(
            module,
            find_result_id(module, result_type).expect("Unable to find type"),
        );
        let storage_class = *extract!(&inst.operands[0], Operand::StorageClass);
        Some(Variable { ty, storage_class })
    }
}

#[derive(Debug)]
pub struct EntryPoint {
    pub name: String,
    pub model: spirv::ExecutionModel,
    pub variables: Vec<Variable>,
}

impl EntryPoint {
    pub fn from_instruction(module: &Module, inst: &Instruction) -> Option<Self> {
        if inst.class.opcode != spirv::Op::EntryPoint {
            return None;
        }
        let ops = &inst.operands;
        let model = *extract!(&ops[0], Operand::ExecutionModel);
        let name = extract!(&ops[2], Operand::LiteralString).clone();
        let variables: Vec<_> = filter_ids(module, &ops[3..])
            .filter_map(|inst| Variable::from_instruction(module, inst))
            .collect();
        Some(EntryPoint {
            name,
            model,
            variables,
        })
    }
}
