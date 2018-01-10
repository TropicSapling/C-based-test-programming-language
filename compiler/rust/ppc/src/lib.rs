use std::path::PathBuf;

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Type {
	GroupOp,
	Literal,
	Number,
	Op,
	Reserved,
	Str1,
	Str2,
	Type,
	Var,
	Whitespace
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Token {
	pub val: String,
	pub t: Type,
	pub line: u32
}

pub fn get_io(input: &PathBuf) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
	let mut default_out = (*input).parent().unwrap().to_path_buf();
	default_out.push("rust");
	default_out.push(input.file_name().unwrap());
	default_out.set_extension("rs");
	
	let mut default_out_dir = (*input).parent().unwrap().to_path_buf();
	default_out_dir.push("rust");
	
	let mut default_fin_out = (*input).parent().unwrap().to_path_buf();
	default_fin_out.push("bin");
	default_fin_out.push(input.file_name().unwrap());
	default_fin_out.set_extension("exe"); // TODO: Support for Linux
	
	let mut default_fin_out_dir = (*input).parent().unwrap().to_path_buf();
	default_fin_out_dir.push("bin");
	
	(default_out, default_out_dir, default_fin_out, default_fin_out_dir)
}