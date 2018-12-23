use std::{
	fs,
	fs::File,
	io,
	io::prelude::*,
	io::Error,
	io::ErrorKind,
	process::Command,
	str,
	usize,
	cell::RefCell,
	mem
};

use crate::library::{Token, Kind, FuncType, Type, FilePos, Function, FunctionSection, Macro};

macro_rules! get_val {
	($e:expr) => ({
		use crate::library::Kind::*;
		use crate::library::Type::*;
		match $e {
			Func(_,_) => String::from("func"),
			GroupOp(ref val, _) => val.to_string(),
			Literal(b) => if b {
				String::from("true")
			} else {
				String::from("false")
			},
			Number(int, fraction) => {
				int.to_string() + "." + &fraction.to_string()
			},
			Op(ref val, _, _, _, _) => val.to_string(),
			Reserved(ref val, _) => val.to_string(),
			Str1(ref val) => "\"".to_string() + val + "\"",
			Str2(ref val) => "'".to_string() + val + "'",
			Type(ref typ, _) => match typ {
				&Array => String::from("array"),
				&Chan => String::from("chan"),
				&Const => String::from("const"),
				&Fraction => String::from("fraction"),
				&Heap => String::from("heap"),
				&List => String::from("list"),
				&Only => String::from("only"),
				&Register => String::from("register"),
				&Stack => String::from("stack"),
				&Unique => String::from("unique"),
				&Volatile => String::from("volatile"),
				&Bool => String::from("bool"),
				&Char => String::from("char"),
				&Int => String::from("int"),
				&Pointer => String::from("pointer"),
				&Unsigned => String::from("unsigned"),
				&Void => String::from("void"),
			},
			Var(ref name, _, _, _, _) => name.to_string()
		}
	});
}

macro_rules! def_builtin_op {
	($a:expr, $b:expr, $name:expr, $typ1:expr, $typ2:expr, $output:expr, $precedence:expr) => (Function {
		structure: vec![
			FunctionSection::Arg($a, vec![vec![$typ1]]),
			FunctionSection::OpID(String::from($name)),
			FunctionSection::Arg($b, vec![vec![$typ2]])
		],
		
		output: if $output == Type::Void {
			vec![vec![]]
		} else {
			vec![vec![$output]]
		},
		
		precedence: $precedence // NOTE: 0 is *lowest* precedence, not highest. Highest precedence is 255.
	})
}

const BUILTIN_FUNCS: usize = 21;

macro_rules! def_builtin_funcs {
	() => (vec![
		// WIP; 'typ' structure needs support for multiple types ('int|fraction' for these operators)
		def_builtin_op!(String::from("a"), String::from("b"), "+", Type::Int, Type::Int, Type::Int, 245),
		def_builtin_op!(String::from("a"), String::from("b"), "-", Type::Int, Type::Int, Type::Int, 245),
		def_builtin_op!(String::from("a"), String::from("b"), "*", Type::Int, Type::Int, Type::Int, 246),
		def_builtin_op!(String::from("a"), String::from("b"), "/", Type::Int, Type::Int, Type::Int, 246),
		def_builtin_op!(String::from("a"), String::from("b"), "%", Type::Int, Type::Int, Type::Int, 246),
		
		// WIP; 'typ' structure needs support for multiple types (all types for these operators)
		def_builtin_op!(String::from("a"), String::from("b"), "==", Type::Int, Type::Int, Type::Bool, 242),
		def_builtin_op!(String::from("a"), String::from("b"), "<", Type::Int, Type::Int, Type::Bool, 243),
		def_builtin_op!(String::from("a"), String::from("b"), ">", Type::Int, Type::Int, Type::Bool, 243),
		
		def_builtin_op!(String::from("a"), String::from("b"), "&&", Type::Bool, Type::Bool, Type::Bool, 238),
		def_builtin_op!(String::from("a"), String::from("b"), "||", Type::Bool, Type::Bool, Type::Bool, 237),
		
		def_builtin_op!(String::from("a"), String::from("b"), "<<", Type::Int, Type::Int, Type::Int, 244),
		def_builtin_op!(String::from("a"), String::from("b"), ">>", Type::Int, Type::Int, Type::Int, 244),
		def_builtin_op!(String::from("a"), String::from("b"), "^", Type::Int, Type::Int, Type::Int, 240),
		
		// WIP; 'macro' types are not yet implemented [EDIT: aren't they now?]
		def_builtin_op!(String::from("a"), String::from("b"), "=", Type::Int, Type::Int, Type::Void, 0),
		
		Function {
			structure: vec![
				FunctionSection::OpID(String::from("!")),
				FunctionSection::Arg(String::from("a"), vec![vec![Type::Int]])
			],
			
			output: vec![vec![Type::Int]],
			
			precedence: 255
		},
		
		Function {
			structure: vec![
				FunctionSection::ID(String::from("let")),
				FunctionSection::Arg(String::from("a"), vec![vec![Type::Int]]), // WIP; No support for any types yet
				FunctionSection::OpID(String::from("=")),
				FunctionSection::Arg(String::from("b"), vec![vec![Type::Int]]) // WIP; No support for any types yet
			],
			
			output: vec![],
			
			precedence: 0
		},
		
		Function {
			structure: vec![
				FunctionSection::ID(String::from("return")),
				FunctionSection::Arg(String::from("a"), vec![vec![Type::Int]]) // WIP; No support for any types yet
			],
			
			output: vec![],
			
			precedence: 0
		},
		
		Function {
			structure: vec![
				FunctionSection::ID(String::from("unsafe")),
				FunctionSection::Arg(String::from("e"), vec![vec![Type::Int]]) // WIP; No support for any types yet
			],
			
			output: vec![vec![Type::Int]], // WIP; No support for any types yet
			
			precedence: 0
		},
		
		Function {
			structure: vec![
				FunctionSection::ID(String::from("Err")),
				FunctionSection::Arg(String::from("e"), vec![vec![Type::Int]])
			],
			
			output: vec![vec![Type::Int]], // WIP; No support for whatever type this actually returns
			
			precedence: 2
		},
		
		Function {
			structure: vec![
				FunctionSection::ID(String::from("println")),
				FunctionSection::Arg(String::from("a"), vec![vec![Type::Int]]) // WIP; No support for strings yet
			],
			
			output: vec![],
			
			precedence: 1
		},
		
		Function {
			structure: vec![
				FunctionSection::ID(String::from("print")),
				FunctionSection::Arg(String::from("a"), vec![vec![Type::Int]]) // WIP; No support for strings yet
			],
			
			output: vec![],
			
			precedence: 1
		}
	])
}

pub fn def_functions() -> Vec<Function> {
	def_builtin_funcs!()
}

fn has_one_arg(structure: &Vec<FunctionSection>) -> bool {
	let mut found_arg = false;
	for section in structure {
		if let FunctionSection::Arg(_,_) = section {
			if found_arg {
				return false;
			} else {
				found_arg = true;
			}
		}
	}
	
	found_arg
}

pub fn parse<'a>(tokens: &'a mut Vec<Token>, mut functions: Vec<Function>) -> (Vec<Function>, Vec<Macro>) {
	let mut macros = Vec::new();
	let mut fpos = Vec::new();
	let mut custom_precedence = None;
	let mut i = 0;
	while i < tokens.len() {
		match tokens[i].kind {
			Kind::Var(ref name, _, _, _, _) if name == "#precedence" => {
				i += 1;
				if let Kind::Number(n, _) = tokens[i].kind {
					custom_precedence = Some(n);
					i += 1;
				}
			},
			
			Kind::Func(ref typ, ref body) => {
				let mut func_struct = vec![];
				let mut precedence = 1;
				
				fpos.push(i);
				
				i += 1;
				
				// Parse function structure
				while i < tokens.len() {
					match tokens[i].kind {
						Kind::GroupOp(ref op, _) if op == "{" || op == ";" => break, // End of function structure
						
						Kind::Op(ref op, _, _, _, _) => {
							let mut name = op.to_string();
							
							i += 1;
							while i < tokens.len() {
								match tokens[i].kind {
									Kind::Op(ref op, _, _, _, _) => {
										name += op;
										i += 1;
									},
									
									_ => break
								}
							}
							
							if name == "->" {
								break; // End of function structure
							} else {
								// Function name
								func_struct.push(FunctionSection::OpID(name));
							}
							
							i -= 1;
						},
						
						Kind::Var(ref name, ref typ2, _, _, _) => if typ2[0].len() > 0 {
							// Function arg
							func_struct.push(FunctionSection::Arg(name.to_string(), typ2.clone()));
						} else {
							// Function name
							func_struct.push(FunctionSection::ID(name.to_string()));
						},
						
						_ => ()
					}
					
					i += 1;
				}
				
				// Get function output
				let output = if let Kind::Type(_, ref typ) = tokens[i].kind {
					if precedence != 247 {
						precedence = if has_one_arg(&func_struct) {
							255
						} else {
							2
						};
					}
					
					typ.clone()
				} else {
					Vec::new()
				};
				
				functions.push(Function {
					structure: func_struct,
					output,
					precedence: if let Some(n) = custom_precedence {
						n as u8
					} else {
						precedence
					}
				});
				
				while i < tokens.len() {
					match tokens[i].kind {
						Kind::GroupOp(ref op, _) if op == "{" || op == ";" => break,
						_ => i += 1
					}
				}
				
				body.replace(i); // Save function body index
				
				if typ == &FuncType::Macro {
					let body = i; // Save function body index for macro
					
					let mut ret_points = Vec::new();
					i += 1;
					
					let mut depth = 0;
					while i < tokens.len() {
						match tokens[i].kind {
							Kind::GroupOp(ref op, _) if op == "{" => depth += 1,
							Kind::GroupOp(ref op, _) if op == "}" => if depth > 0 {
								depth -= 1;
							} else {
								break;
							},
							
							Kind::Var(ref name, _, _, _, _) if name == "return" => ret_points.push(i),
							
							_ => ()
						}
						
						i += 1;
					}
					
					macros.push(Macro {func: functions.len() - 1, ret_points, body});
				}
				
				custom_precedence = None;
			},
			
			_ => {
				custom_precedence = None;
				i += 1;
			}
		}
	}
	
	let mut id = BUILTIN_FUNCS;
	for i in fpos {
		match tokens[i].kind {
			Kind::Func(ref mut f, _) => if let FuncType::Func(ref mut f) = f {
				*f = id;
			},
			
			_ => unreachable!()
		}
		
		id += 1;
	}
	
	(functions, macros)
}

fn remove_first(s: &str) -> &str {
    s.chars().next().map(|c| &s[c.len_utf8()..]).unwrap()
}

fn parse_func(tokens: &mut Vec<Token>, functions: &Vec<Function>, blueprint: &Vec<(&FunctionSection, usize)>, all_children: &mut Vec<usize>) {
	if blueprint.len() > 1 {
		let mut last_s = 0;
		let mut parents = &RefCell::new(Vec::new());
		
		for (s, section) in blueprint.iter().enumerate() {
			match section.0 {
				FunctionSection::ID(_) | FunctionSection::OpID(_) => {
					let parent = &tokens[section.1];
					
					let rhs_start = if let Kind::Op(_, ref ops, _, _, _) = parent.kind {
						let ops = ops.borrow();
						
						if ops.len() > 0 {
							ops[ops.len() - 1] + 1
						} else {
							section.1 + 1
						}
					} else {
						section.1 + 1
					};
					
					match parent.kind {
						Kind::Op(_, _, ref children, ref sidekicks, _) | Kind::Var(_, _, ref children, ref sidekicks, _) => {
							if last_s == 0 {
								parents = sidekicks;
							}
							
							let mut i = section.1 - 1;
							let mut c = 0;
							let mut depth = 0;
							while i > 0 && c < s - last_s {
								match tokens[i].kind {
									Kind::GroupOp(ref op, _) if op == "}" => depth += 1,
									Kind::GroupOp(ref op, _) if op == "{" => {
										depth -= 1;
										if depth == 0 && !all_children.contains(&i) {
											children.borrow_mut().push(i);
											all_children.push(i);
											c += 1;
										}
									},
									
									Kind::GroupOp(_,_) => (),
									
									Kind::Op(ref op, _, _, _, _) if depth == 0 => {
										let mut name = op.to_string();
										
										i -= 1;
										while i > 0 {
											match tokens[i].kind {
												Kind::Op(ref op, _, _, _, _) => {
													name += op;
													i -= 1;
												},
												
												_ => break
											}
										}
										i += 1;
										
										name = name.chars().rev().collect();
										
										while !functions.iter().find(|f| {
											let mut m = false;
											for section in &f.structure {
												match section {
													FunctionSection::OpID(ref op) => if op == &name {
														m = true;
														break;
													},
													
													FunctionSection::ID(_) => break,
													
													_ => ()
												}
											}
											
											m
										}).is_some() {
											name = remove_first(&name).to_string();
											i += 1;
										}
										
										if !all_children.contains(&i) {
											children.borrow_mut().push(i);
											all_children.push(i);
											c += 1;
										}
									},
									
									_ => if depth == 0 && !all_children.contains(&i) {
										children.borrow_mut().push(i);
										all_children.push(i);
										c += 1;
									}
								}
								
								i -= 1;
							}
							
							children.borrow_mut().reverse();
							
							let mut s2 = s + 1;
							while s2 < blueprint.len() {
								match blueprint[s2].0 {
									FunctionSection::ID(_) | FunctionSection::OpID(_) => break,
									_ => s2 += 1
								}
							}
							
							if s2 >= blueprint.len() {
								let mut i = rhs_start;
								let mut s = s + 1;
								let mut depth = 0;
								while i < tokens.len() && s < blueprint.len() {
									match tokens[i].kind {
										Kind::GroupOp(ref op, _) if op == "}" => depth -= 1,
										Kind::GroupOp(ref op, _) if op == "{" => {
											if depth == 0 && !all_children.contains(&i) {
												children.borrow_mut().push(i);
												all_children.push(i);
												s += 1;
											}
											
											depth += 1;
										},
										
										Kind::GroupOp(_,_) => (),
										
										Kind::Op(ref op, _, _, _, _) if depth == 0 => {
											if !all_children.contains(&i) {
												children.borrow_mut().push(i);
												all_children.push(i);
												s += 1;
											}
											
											let mut name = op.to_string();
											
											i += 1;
											while i < tokens.len() {
												match tokens[i].kind {
													Kind::Op(ref op, _, _, _, _) => {
														name += op;
														i += 1;
													},
													
													_ => break
												}
											}
											i -= 1;
											
											while !functions.iter().find(|f| {
												let mut m = false;
												for section in &f.structure {
													match section {
														FunctionSection::OpID(ref op) => if op == &name {
															m = true;
															break;
														},
														
														FunctionSection::ID(_) => break,
														
														_ => ()
													}
												}
												
												m
											}).is_some() {
												name.pop();
												i -= 1;
											}
										},
										
										_ => if depth == 0 && !all_children.contains(&i) {
											children.borrow_mut().push(i);
											all_children.push(i);
											s += 1;
										}
									}
									
									i += 1;
								}
							}
						},
						
						_ => unreachable!()
					}
					
					if last_s != 0 {
						parents.borrow_mut().push(section.1);
						all_children.push(section.1);
					}
					
					last_s = s + 1;
				},
				
				_ => ()
			}
		}
	} else {
		match &tokens[blueprint[0].1].kind {
			Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) => {
				children.borrow_mut().push(usize::MAX);
			},
			
			_ => unreachable!()
		}
	}
}

fn get_parse_limit(tokens: &Vec<Token>, i: &mut usize) -> usize {
	let mut depth = 0;
	let mut limit = tokens.len();
	let start = *i;
	while *i < limit {
		match tokens[*i].kind {
			Kind::GroupOp(ref op, _) if op == ";" => if depth == 0 {
				limit = *i;
				break;
			},
			
			Kind::GroupOp(ref op, _) if op == "{" => {
				depth += 1;
			},
			
			Kind::GroupOp(ref op, _) if op == "}" => if depth > 0 {
				depth -= 1;
			} else {
				limit = *i;
				break;
			},
			
			_ => ()
		}
		
		*i += 1;
	}
	
	*i = start;
	
	limit
}

fn update_matches<'a>(matches: &mut Vec<(usize, Vec<(&'a FunctionSection, usize)>, usize)>, functions: &'a Vec<Function>, name: &String, depth: usize, pos: usize, has_children: bool) -> bool {
	let mut new_match = false;
	for (i, f) in functions.iter().enumerate() {
		for (j, section) in f.structure.iter().enumerate() {
			match section {
				FunctionSection::ID(ref s) | FunctionSection::OpID(ref s) if s == name => {
					if !has_children {
						for m in matches.iter_mut().filter(|m| m.0 == i) {
							if m.1.len() == j && pos != m.1[m.1.len() - 1].1 {
								if let Some(_) = m.1.iter().find(|s| match s.0 {
									FunctionSection::Arg(_,_) => false,
									_ => true
								}) {
									if m.2 == depth {
										m.1.push((section, pos));
									}
								} else {
									m.1.push((section, pos));
									m.2 = depth;
								}
							}
						}
						
						if j == 0 {
							matches.push((i, vec![(section, pos)], depth));
						}
					}
					
					new_match = true;
				},
				
				FunctionSection::Arg(_,_) => {
					for m in matches.iter_mut().filter(|m| m.0 == i) {
						if m.1.len() == j && m.2 <= depth && pos != m.1[m.1.len() - 1].1 {
							m.1.push((section, pos));
						}
					}
					
					if j == 0 {
						matches.push((i, vec![(section, pos)], depth));
					}
				},
				
				_ => ()
			}
		}
	}
	
	new_match
}

fn cleanup_matches(matches: &mut Vec<(usize, Vec<(&FunctionSection, usize)>, usize)>, functions: &Vec<Function>) {
	matches.retain(|m| m.1.len() == functions[m.0].structure.len());
	
	let mut i = 0;
	while i < matches.len() {
		let mut found = false;
		for (j, m) in matches.iter().enumerate() {
			if j != i {
				let mut matching = true;
				for section in &matches[i].1 {
					match section.0 {
						FunctionSection::ID(_) | FunctionSection::OpID(_) => if !m.1.contains(&section) {
							matching = false;
							break;
						},
						
						_ => ()
					}
				}
				
				if matching && m.1.len() > matches[i].1.len() {
					found = true;
					break;
				}
			}
		}
		
		if found {
			matches.remove(i);
		} else {
			i += 1;
		}
	}
}

fn cleanup_matches2(matches: &mut Vec<(usize, Vec<(&FunctionSection, usize)>, usize)>, functions: &Vec<Function>, depth: usize) {
	matches.retain(|m| m.2 <= depth || m.1.len() == functions[m.0].structure.len() || match m.1.iter().find(|s| match s.0 {
		FunctionSection::Arg(_,_) => false,
		_ => true
	}) {
		Some(_) => false,
		None => true
	});
}

fn get_highest<'a>(matches: &'a Vec<(usize, Vec<(&'a FunctionSection, usize)>, usize)>, functions: &Vec<Function>) -> Option<&'a (usize, Vec<(&'a FunctionSection, usize)>, usize)> {
	if matches.len() > 0 {
		let mut top = &matches[0];
		for m in matches {
			if m.2 > top.2 || (m.2 == top.2 && functions[m.0].precedence >= functions[top.0].precedence) {
				top = m;
			}
		}
		
		Some(top)
	} else {
		None
	}
}

pub fn parse_statement(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &Vec<Macro>, all_children: &mut Vec<usize>, i: &mut usize, debugging: bool) -> Option<usize> {
	let start = *i;
	let limit = get_parse_limit(tokens, i);
	let mut parsed = Vec::new();
	let mut lowest = None;
	
	loop {
		let mut matches = Vec::new();
		let mut depth = 0;
		let mut depth2 = 0;
		*i = start;
		while *i < limit {
			match tokens[*i].kind.clone() {
				Kind::GroupOp(ref op, _) if op == "(" && depth2 == 0 => depth += 1,
				Kind::GroupOp(ref op, _) if op == ")" && depth2 == 0 => if depth > 0 {
					depth -= 1;
					cleanup_matches2(&mut matches, functions, depth + depth2);
				} else {
					panic!("{}:{} Excess ending parenthesis", tokens[*i].pos.line, tokens[*i].pos.col);
				},
				
				Kind::GroupOp(ref op, _) if op == "{" => {
					if !all_children.contains(i) {
						update_matches(&mut matches, functions, &String::new(), depth + depth2, *i, true);
					}
					
					if depth2 == 0 && !parsed.contains(i) {
						parsed.push(*i);
						parse2(tokens, functions, macros, all_children, i, debugging);
					} else {
						depth2 += 1;
					}
				},
				
				Kind::GroupOp(ref op, _) if op == "}" => {
					depth2 -= 1;
				},
				
				Kind::Op(ref op, _, ref children, _, _) if depth2 == 0 => {
					let start = *i;
					let mut name = op.to_string();
					
					if let Kind::Op(_, ref ops, _, _, _) = tokens[*i].kind {
						if !all_children.contains(&start) {
							if ops.borrow().len() > 0 {
								for &s in ops.borrow().iter() {
									name += match tokens[s].kind {
										Kind::Op(ref op, _, _, _, _) => {
											*i = s;
											op
										},
										
										_ => unreachable!()
									};
								}
								
								update_matches(&mut matches, functions, &name, depth + depth2, start, children.borrow().len() > 0);
							} else {
								*i += 1;
								while *i < limit {
									match tokens[*i].kind {
										Kind::Op(ref op, _, _, _, _) => name += op,
										_ => break
									}
									
									*i += 1;
								}
								*i -= 1;
								
								while name.len() > 0 && !update_matches(&mut matches, functions, &name, depth + depth2, start, children.borrow().len() > 0) {
									name.pop();
									*i -= 1;
								}
								
								let mut j = start + 1;
								while j < tokens.len() && j < *i + 1 {
									ops.borrow_mut().push(j);
									j += 1;
								}
							}
						} else {
							if ops.borrow().len() > 0 {
								for &s in ops.borrow().iter() {
									match tokens[s].kind {
										Kind::Op(_, _, _, _, _) => {
											*i = s;
										},
										
										_ => unreachable!()
									};
								}
							}
						}
					}
				},
				
				Kind::Var(ref name, _, ref children, _, _) if depth2 == 0 && !all_children.contains(i) => {
					update_matches(&mut matches, functions, name, depth + depth2, *i, children.borrow().len() > 0 );
				},
				
				_ => if depth2 == 0 && !all_children.contains(i) {
					update_matches(&mut matches, functions, &String::new(), depth + depth2, *i, false);
				}
			}
			
			*i += 1;
		}
		
		if depth > 0 {
			panic!("Unclosed parenthesis on line {}", tokens[start].pos.line);
		}
		
		cleanup_matches(&mut matches, functions);
		
		match get_highest(&matches, functions) {
			Some(m) => {
				for section in &m.1 {
					match section.0 {
						FunctionSection::ID(_) | FunctionSection::OpID(_) => {
							lowest = Some(section.1);
							
							for (i, mac) in macros.iter().enumerate() {
								if mac.func == m.0 {
									match tokens[section.1].kind {
										Kind::Var(_, _, _, _, ref macro_id) | Kind::Op(_, _, _, _, ref macro_id) => {
											macro_id.replace(Some(i));
										},
										
										_ => unreachable!()
									}
									
									break;
								}
							}
							
							break;
						},
						
						_ => ()
					}
				}
				
				parse_func(tokens, functions, &m.1, all_children);
				
				// DEBUG BELOW
				if debugging {
					match tokens[lowest.unwrap()].kind {
						Kind::Op(_, _, ref children, ref sidekicks, _) | Kind::Var(_, _, ref children, ref sidekicks, _) => {
							print!("\x1b[0m\x1b[1m\x1b[38;5;11m");
							
							for section in &m.1 {
								match section.0 {
									FunctionSection::ID(ref name) | FunctionSection::OpID(ref name) => print!(" {}", name),
									FunctionSection::Arg(ref arg, _) => print!(" <{}>", arg)
								}
							}
							
							print!(":\x1b[0m (");
							for child in children.borrow().iter() {
								if *child != usize::MAX {
									print!("\x1b[0m\x1b[1m\x1b[38;5;10m{}\x1b[0m[{}]", get_val!(tokens[*child].kind), *child);
									match tokens[*child].kind {
										Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) | Kind::GroupOp(_, ref children) if children.borrow().len() > 0 => {
											print!(": (");
											for child in children.borrow().iter() {
												if *child != usize::MAX {
													print!("\x1b[0m\x1b[1m\x1b[38;5;10m{}\x1b[0m[{}], ", get_val!(tokens[*child].kind), *child);
												}
											}
											print!(")");
										},
										
										_ => ()
									}
									print!(", ");
								}
							}
							print!(")");
							
							if sidekicks.borrow().len() > 0 {
								print!(", {{");
								
								for s in sidekicks.borrow().iter() {
									match tokens[*s].kind {
										Kind::Op(ref name, _, ref children, _, _) | Kind::Var(ref name, _, ref children, _, _) | Kind::GroupOp(ref name, ref children) => {
											print!("\x1b[0m\x1b[1m\x1b[38;5;10m{}\x1b[0m[{}]: (", name, s);
											for child in children.borrow().iter() {
												if *child != usize::MAX {
													print!("\x1b[0m\x1b[1m\x1b[38;5;10m{}\x1b[0m[{}]", get_val!(tokens[*child].kind), *child);
													match tokens[*child].kind {
														Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) | Kind::GroupOp(_, ref children) if children.borrow().len() > 0 => {
															print!(": (");
															for child in children.borrow().iter() {
																if *child != usize::MAX {
																	print!("\x1b[0m\x1b[1m\x1b[38;5;10m{}\x1b[0m[{}], ", get_val!(tokens[*child].kind), *child);
																}
															}
															print!(")");
														},
														
														_ => ()
													}
													print!(", ");
												}
											}
											print!("), ");
										},
										
										_ => unreachable!()
									}
								}
								
								println!("}}");
							} else {
								println!("");
							}
						},
						
						_ => unreachable!()
					}
				}
			},
			
			None => break
		}
	}
	
	lowest
}

/* fn parse_type_decl<'a>(tokens: &mut Vec<Token>, functions: &Vec<Function>, i: &mut usize, parent: usize) {
	let start = *i + 1;
	
	{
//		let mut body = tokens[*i].children.borrow_mut();
		let mut body = match tokens[*i].kind {
			Kind::Var(_, _, ref mut children) => children,
			_ => unreachable!()
		};
		*i += 1;
		
		while *i < tokens.len() {
			match tokens[*i].kind {
				Kind::Op(ref op) => if op == "=" {
//					tokens[parent].children.borrow_mut().push(start - 1);
					children.push(start - 1);
					break;
				} else {
					*i = start - 1;
					return;
				},
				_ => *i += 1
			}
		}
		
		if *i >= tokens.len() {
			panic!("Unexpected EOF");
		}
		
		body.push(*i);
	}
	
	*i = start;
	parse_statement(tokens, functions, i);
} */

pub fn parse2(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &Vec<Macro>, all_children: &mut Vec<usize>, i: &mut usize, debugging: bool) {
	match tokens[*i].kind.clone() {
		Kind::GroupOp(ref op, _) if op == "{" => {
			let parent = *i;
			let mut nests = 0;
			*i += 1;
			
			while *i < tokens.len() {
				let start = *i;
				
				if let Kind::GroupOp(ref op, _) = tokens[*i].kind.clone() {
					if op == "{" {
						nests += 1;
						
						if let Kind::GroupOp(_, ref children) = tokens[parent].kind {
							children.borrow_mut().push(*i);
						}
						
						parse2(tokens, functions, macros, all_children, i, debugging);
						
						*i += 1;
						continue;
					}
				}
				
				match tokens[*i].kind.clone() {
					Kind::GroupOp(ref op, _) if op == "}" => if nests > 0 {
						nests -= 1;
					} else {
						break;
					},
					
					_ => match tokens[*i].kind.clone() {
//						Kind::Type(_) => parse_type_decl(tokens, functions, i, parent),
						
						Kind::GroupOp(ref op, _) if op == ";" => {
							if let Kind::GroupOp(_, ref children) = tokens[parent].kind {
								children.borrow_mut().push(*i);
							}
							
							*i += 1;
						},
						
						_ => if let Some(token) = parse_statement(tokens, functions, macros, all_children, i, debugging) {
							if let Kind::GroupOp(_, ref children) = tokens[parent].kind {
								children.borrow_mut().push(token);
							}
						} else {
							if let Kind::GroupOp(_, ref children) = tokens[parent].kind {
								children.borrow_mut().push(start); // Should this really be pushing start instead of *i?
							}
						}
					}
				}
			}
		},
		
		_ => ()
	}
}

/* pub fn parse3(tokens: &mut Vec<Token>, macros: &mut Vec<Macro>, functions: &mut Vec<Function>, i: &mut usize, depth: &mut usize, rows: &mut Vec<usize>) -> Result<(), Error> {
	match tokens[*i].kind.clone() {
//		Kind::Var(ref name, _) => return parse_macro_func(tokens, macros, functions, i, name, 1, *depth, rows, &mut false),
		
		Kind::Op(ref op, _) if op != ":" => { // 'op != ":"' part is tmp, used to allow Rust-style importing
			let mut name = op.to_string();
			let start = *i;
			
			get_op_name(tokens, functions, i, &mut name);
			
			let end = *i;
			*i = start;
			
			let mut found = false;
//			let res = parse_macro_func(tokens, macros, functions, i, &name, name.len(), *depth, rows, &mut found);
			
			if !found {
				*i = end;
			}
			
//			return res;
		},
		
		Kind::GroupOp(ref op, _) if op == "{" => {
			*depth += 1;
			if *depth + 1 > rows.len() {
				rows.push(0);
			} else {
				rows[*depth] += 1;
			}
		},
		
		Kind::GroupOp(ref op, _) if op == "}" => if *depth > 0 {
			*depth -= 1;
		} else {
			panic!("{}:{} Excess ending bracket", tokens[*i].pos.line, tokens[*i].pos.col);
		},
		
		_ => ()
	}
	
	Ok(())
} */

fn insert_macro(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &mut Vec<Macro>, i: &mut usize, pars: &Vec<FunctionSection>, args: &Vec<usize>, children: &RefCell<Vec<usize>>, sof: usize, ret_points: &mut usize) -> Result<(), Error> {
	// Get new children positions
	let mut new_children = Vec::new();
	for child in children.borrow().iter() {
		new_children.push(tokens.len());
		insert_macro2(tokens, functions, macros, &mut child.clone(), pars, args, sof, ret_points)?;
	}
	
	match tokens[*i].kind {
		Kind::GroupOp(_, ref children) | Kind::Reserved(_, ref children) | Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) => {
			// Replace old positions with new
			children.replace(new_children);
		},
		
		_ => ()
	}
	
	Ok(())
}

fn insert_macro2(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &mut Vec<Macro>, i: &mut usize, pars: &Vec<FunctionSection>, args: &Vec<usize>, sof: usize, ret_points: &mut usize) -> Result<(), Error> {
	let token = tokens[*i].clone();
	match token.kind.clone() {
		Kind::GroupOp(_, ref children) | Kind::Reserved(_, ref children) | Kind::Op(_, _, ref children, _, _) => {
			// Nothing to replace; just add token and its children directly
			
			tokens.push(token);
			
			let mut i = tokens.len() - 1;
			insert_macro(tokens, functions, macros, &mut i, pars, args, children, sof, ret_points)?;
		},
		
		Kind::Var(ref name, _, _, _, _) if *ret_points > 0 && name == "return" => {
			tokens.push(token.clone());
			if let Kind::Var(_, _, ref children, _, _) = tokens[tokens.len() - 1].kind {
				children.replace(vec![tokens.len()]);
			}
			
			tokens.push(Token {
				kind: Kind::Var(String::from("Err"), Vec::new(), RefCell::new(vec![tokens.len() + 1]), RefCell::new(Vec::new()), RefCell::new(None)),
				pos: token.pos.clone()
			});
			
			tokens.push(Token {
				kind: Kind::Number(*ret_points - 1, 0),
				pos: token.pos.clone()
			});
			
			if let Kind::Var(_, _, ref children, _, _) = tokens[tokens.len() - 2].kind {
				children.replace(vec![tokens.len() - 1]);
			}
			
			*ret_points += 1;
		},
		
		Kind::Var(ref name, _, ref children, _, _) => {
			let mut matching = false;
			let mut p = 0;
			for par in pars {
				if let FunctionSection::Arg(ref par_name, _) = par {
					if name == par_name {
						// Found variable to replace with input code; insert replacement
						
						matching = true;
						
						tokens.push(tokens[args[p]].clone());
						parse3_tok(tokens, functions, macros, &mut (tokens.len() - 1), sof)?;
						
						break;
					}
					
					p += 1;
				}
			}
			
			if !matching {
				// Variable should not be replaced; just insert the variable and its children directly instead
				
				tokens.push(token);
				insert_macro(tokens, functions, macros, &mut (tokens.len() - 1), pars, args, children, sof, ret_points)?;
			}
		},
		
		_ => tokens.push(token) // No children; just add token directly
	}
	
	Ok(())
}

fn run_macro(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &mut Vec<Macro>, m: usize, sof: usize, func: usize, input: &Vec<usize>, returning: bool) -> Result<usize, Error> {
	// Get init function pos
	let mut init_id = 0;
	for (f, function) in functions.iter().enumerate() {
		let mut is_init = false;
		for section in function.structure.iter() {
			match section {
				FunctionSection::ID(ref name) | FunctionSection::OpID(ref name) => {
					if name == "init" {
						is_init = true;
					} else {
						is_init = false;
						break;
					}
				},
				
				_ => ()
			}
		}
		
		if is_init {
			init_id = f;
		}
	}
	
	//////// COMPILE MACRO ////////
	
	let mut j = tokens.len();
	
	tokens.push(Token {
		kind: Kind::Func(FuncType::Func(init_id), RefCell::new(j + 2)),
		pos: FilePos {line: 0, col: 0}
	});
	
	tokens.push(Token {
		kind: Kind::Var(String::from("init"), Vec::new(), RefCell::new(Vec::new()), RefCell::new(Vec::new()), RefCell::new(None)),
		pos: FilePos {line: 0, col: 0}
	});
	
	let body = tokens[macros[m].body].clone();
	tokens.push(body.clone());
	
	if let Kind::GroupOp(_, ref children) = body.kind.clone() {
		insert_macro(tokens, functions, macros, &mut (tokens.len() - 1), &functions[func].structure, &input, children, sof, &mut 1)?;
	}
	
	tokens.push(Token {
		kind: Kind::GroupOp(String::from("}"), RefCell::new(Vec::new())),
		pos: FilePos {line: 0, col: 0}
	});
	
	let mut out_contents = String::new();
	while j < tokens.len() {
		out_contents = compile(&tokens, &functions, &mut j, out_contents);
		j += 1;
	}
	
	if out_contents == "fn main(){return Err(0);}" {
		return Ok(0); // No macro code to run except useless return; save some time by skipping file creation & running
	}
	
	out_contents.insert_str(9, "->Result<(),usize>");
	
	if !returning {
		out_contents.insert_str(out_contents.len() - 1, "Ok(())");
	}
	
	//////// CREATE RUST OUTPUT ////////
	
	fs::create_dir_all("macros")?;
	
	let mut out_file = File::create("macros\\macro.rs")?;
	out_file.write_all(out_contents.as_bytes())?;
	
	Command::new("rustfmt").arg("macros\\macro.rs").output().expect("failed to format Rust code");
	
	//////// CREATE BINARY OUTPUT ////////
	
	let mut error = false;
	
	let out = Command::new("rustc")
			.args(&["--color", "always", "-A", "unused_parens", "-A", "unused_must_use", "-A", "unused_unsafe", "-A", "unreachable_code", "-A", "unused_mut", "--out-dir", "macros", "macros\\macro.rs"])
			.output()
			.expect("failed to compile Rust code");
	
	if out.stdout.len() > 0 {
		println!("{}", str::from_utf8(&out.stdout).unwrap());
	}
	
	if out.stderr.len() > 0 {
		println!("{}", str::from_utf8(&out.stderr).unwrap());
		
		if !out.stderr.starts_with(b"\x1b[0m\x1b[1m\x1b[38;5;11mwarning") {
			error = true;
		}
	}
	
	//////// RUN COMPILED BINARY ////////
	
	let mut ret_point = 0;
	
	if !error {
		let out = if cfg!(target_os = "windows") {
			Command::new("macros\\macro.exe")
				.output()
				.expect("failed to execute process")
		} else {
			Command::new("./macros/macro.exe")
				.output()
				.expect("failed to execute process")
		};
		
		if out.stdout.len() > 0 {
			println!("{}", str::from_utf8(&out.stdout).unwrap());
			io::stdout().flush()?;
		}
		
		if out.stderr.len() > 0 {
			if out.stderr.starts_with(b"Error: ") {
				// Replace macro function call with results
				
				let point = str::from_utf8(&out.stderr).unwrap()[7..out.stderr.len() - 1].parse::<usize>();
				
				if let Ok(point) = point {
					ret_point = point;
				}
			} else {
				println!("{}", str::from_utf8(&out.stderr).unwrap());
			}
		}
	}
	
	//////// DELETE CREATED FILES ////////
	
	fs::remove_file("macros\\macro.rs")?;
	
	if !error {
		fs::remove_file("macros\\macro.exe")?;
		fs::remove_file("macros\\macro.pdb")?;
	} else {
		return Err(Error::new(ErrorKind::Other, "compilation of macro failed"));
	}
	
//	fs::remove_dir("macros")?; // Doesn't work (on Windows) for some reason?
	
	Ok(ret_point)
}

fn expand_macro(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &mut Vec<Macro>, i: &mut usize, m: usize, args: &RefCell<Vec<usize>>, sidekicks: &RefCell<Vec<usize>>, sof: usize) -> Result<(), Error> {
	// Get input
	let mut input = args.borrow().clone();
	for &sidekick in sidekicks.borrow().iter() {
		match tokens[sidekick].kind {
			Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) => {
				for child in children.borrow().iter() {
					input.push(*child);
				}
			},
			
			_ => unreachable!()
		}
	}
	
	let func = macros[m].func;
	let returning = macros[m].ret_points.len() > 0;
	let ret_point = run_macro(tokens, functions, macros, m, sof, func, &input, returning)?;
	
	if returning {
		// Macros returning code
		
		if let Kind::Var(_, _, ref children, _, _) = tokens[macros[m].ret_points[ret_point]].kind.clone() {
			let ret_child = children.borrow()[0];
			match tokens[ret_child].kind.clone() {
				Kind::GroupOp(_, ref children) | Kind::Reserved(_, ref children) | Kind::Op(_, _, ref children, _, _) => {
					// Nothing to replace; just insert token and its children directly
					
					// Replace macro call with return point
					let ret_child = tokens[ret_child].clone();
					mem::replace(&mut tokens[*i], ret_child);
					
					insert_macro(tokens, functions, macros, i, &functions[func].structure, &input, children, sof, &mut 0)?;
				},
				
				Kind::Var(ref name, _, ref children, _, _) => {
					let mut matching = false;
					let mut p = 0;
					for par in &functions[func].structure {
						if let FunctionSection::Arg(ref par_name, _) = par {
							if name == par_name {
								// Found variable to replace with input code; replace
								
								matching = true;
								
								let arg = tokens[input[p]].clone();
								mem::replace(&mut tokens[*i], arg);
								
								let mut i = *i;
								parse3_tok(tokens, functions, macros, &mut i, sof)?;
								
								break;
							}
							
							p += 1;
						}
					}
					
					if !matching {
						// Replace macro call with return point
						let ret_child = tokens[ret_child].clone();
						mem::replace(&mut tokens[*i], ret_child);
						
						insert_macro(tokens, functions, macros, i, &functions[func].structure, &input, children, sof, &mut 0)?;
					}
				},
				
				_ => {
					// No children; replace macro call with return point
					let ret_child = tokens[ret_child].clone();
					mem::replace(&mut tokens[*i], ret_child);
				}
			}
		}
	} else {
		// Macros not returning any code
		
		let pos = tokens[*i].pos.clone();
		mem::replace(&mut tokens[*i], Token {
			kind: Kind::GroupOp(String::from(")"), RefCell::new(Vec::new())),
			pos: pos.clone()
		});
	}
	
	*i += 1;
	
	Ok(())
}

fn parse3_tok(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &mut Vec<Macro>, i: &mut usize, sof: usize) -> Result<(), Error> {
	match tokens[*i].kind.clone() {
		Kind::GroupOp(ref op, _) => if op != ";" {
			parse3_body(tokens, functions, macros, i, sof)?;
		},
		
		Kind::Var(_, _, ref children, ref sidekicks, ref macro_id) => if let Some(id) = *macro_id.borrow() {
			// Found macro; expand
			expand_macro(tokens, functions, macros, i, id, children, sidekicks, sof)?;
		} else {
			// Not a macro; go through children looking for macros there instead
			
			for child in children.borrow().iter() {
				*i = *child;
				if *i != usize::MAX {
					parse3_tok(tokens, functions, macros, i, sof)?;
				}
			}
			
			for sidekick in sidekicks.borrow().iter() {
				*i = *sidekick;
				parse3_tok(tokens, functions, macros, i, sof)?;
			}
		},
		
		Kind::Op(_, ref ops, ref children, ref sidekicks, ref macro_id) => if let Some(id) = *macro_id.borrow() {
			// Found macro; expand
			
			expand_macro(tokens, functions, macros, i, id, children, sidekicks, sof)?;
			
			let ops = ops.borrow();
			if ops.len() > 0 {
				*i = ops[ops.len() - 1];
			}
		} else {
			// Not a macro; go through children looking for macros there instead
			
			for child in children.borrow().iter() {
				*i = *child;
				if *i != usize::MAX {
					parse3_tok(tokens, functions, macros, i, sof)?;
				}
			}
			
			for sidekick in sidekicks.borrow().iter() {
				*i = *sidekick;
				parse3_tok(tokens, functions, macros, i, sof)?;
			}
			
			let ops = ops.borrow();
			if ops.len() > 0 {
				*i = ops[ops.len() - 1];
			}
		},
		
		_ => ()
	}
	
	Ok(())
}

fn parse3_body(tokens: &mut Vec<Token>, functions: &Vec<Function>, macros: &mut Vec<Macro>, i: &mut usize, sof: usize) -> Result<(), Error> {
	if let Kind::GroupOp(_, ref statements) = tokens[*i].kind.clone() {
		// Parse each statement in body
		for statement in statements.borrow().iter() {
			*i = *statement;
			parse3_tok(tokens, functions, macros, i, sof)?;
		}
	}
	
	Ok(())
}

pub fn parse3(tokens: &mut Vec<Token>, macros: &mut Vec<Macro>, functions: &Vec<Function>, i: &mut usize, sof: usize) -> Result<(), Error> {
	match tokens[*i].kind.clone() {
		Kind::Func(_, ref body) => {
			*i = *body.borrow();
			parse3_body(tokens, functions, macros, i, sof)
		},
		
		_ => Ok(())
	}
}

fn compile_type(typ: &Vec<Vec<Type>>) -> String {
	use crate::library::Type::*;
	
	let mut output = String::new();
	let mut unsigned = false;
	
	for t in &typ[0] { // TMP until I've worked out how to handle multiple types
		match t {
			Array => (), // WIP
			Bool => output += "bool",
			Chan => (), // WIP
			Char => output += "char",
			Const => (),
			Fraction => (), // WIP
			Heap => (), // WIP
			Int => if unsigned {
				output += "usize";
			} else {
				output += "isize";
			},
			List => (), // WIP
			Only => (), // WIP
			Pointer => output += "&", // NOTE: Needs changing (for example pointer*2)
			Register => (), // WIP
			Stack => (), // WIP
			Unique => (), // WIP
			Unsigned => unsigned = true,
			Void => (), // NOTE: Needs changing to 'output += "()"' once Void is not used for none-existing parameters (use None instead)
			Volatile => (), // WIP
		}
	}
	
	output
}

fn compile_func(function: &Function, mut output: String) -> String {
	let mut is_init = false;
	for section in function.structure.iter() {
		match section {
			FunctionSection::ID(ref name) | FunctionSection::OpID(ref name) => {
				if name == "init" {
					is_init = true;
				} else {
					is_init = false;
					break;
				}
			},
			
			_ => ()
		}
	}
	
	if is_init {
		output += "main";
	} else {
		let mut s = String::new();
		for section in function.structure.iter() {
			match section {
				FunctionSection::ID(ref name) | FunctionSection::OpID(ref name) => {
					for c in name.chars() {
						let ch = c.to_string();
						s += match c {
							'+' => "plus",
							'-' => "minus",
							'*' => "times",
							'/' => "div",
							'%' => "mod",
							'=' => "eq",
							'&' => "and",
							'|' => "or",
							'^' => "xor",
							'<' => "larrow",
							'>' => "rarrow",
							'!' => "not",
							'~' => "binnot",
							'?' => "quest",
							':' => "colon",
							'.' => "dot",
							',' => "comma",
							'@' => "at",
							_ => &ch
						};
					}
					
					s += "_";
				},
				
				_ => ()
			}
		}
		
		output += &s[..s.len() - 1];
		output += "_ppl";
	}
		
	output += "(";
	
	let mut not_first_arg = false;
	for section in function.structure.iter() {
		match section {
			FunctionSection::Arg(ref name, ref typ) => {
				if not_first_arg {
					output += ",";
				}
				
				output += name;
				output += "_ppl";
				output += ":";
				output += &compile_type(typ);
				
				not_first_arg = true;
			},
			
			_ => ()
		}
	}
	
	output += ")";
	
	if function.output.len() > 0 {
		output += "->";
		output += &compile_type(&function.output);
	}
	
	output
}

fn type_full_name(tokens: &Vec<Token>, output: String, sidekicks: &RefCell<Vec<usize>>, name: &str) -> (String, String) {
	if sidekicks.borrow().len() > 0 {
		let mut s = name.to_string() + "_";
		
		for sidekick in sidekicks.borrow().iter() {
			match tokens[*sidekick].kind {
				Kind::Op(ref op, ref ops, _, _, _) => {
					s += match op.as_ref() {
						"+" => "plus",
						"-" => "minus",
						"*" => "times",
						"/" => "div",
						"%" => "mod",
						"=" => "eq",
						"&" => "and",
						"|" => "or",
						"^" => "xor",
						"<" => "larrow",
						">" => "rarrow",
						"!" => "not",
						"~" => "binnot",
						"?" => "quest",
						":" => "colon",
						"." => "dot",
						"," => "comma",
						"@" => "at",
						_ => op
					};
					
					for op in ops.borrow().iter() {
						if let Kind::Op(ref op, _, _, _, _) = tokens[*op].kind {
							s += match op.as_ref() {
								"+" => "plus",
								"-" => "minus",
								"*" => "times",
								"/" => "div",
								"%" => "mod",
								"=" => "eq",
								"&" => "and",
								"|" => "or",
								"^" => "xor",
								"<" => "larrow",
								">" => "rarrow",
								"!" => "not",
								"~" => "binnot",
								"?" => "quest",
								":" => "colon",
								"." => "dot",
								"," => "comma",
								"@" => "at",
								_ => op
							};
						}
					}
					
					s += "_";
				},
				
				Kind::Var(ref name, _, _, _, _) => {
					s += name;
					s += "_";
				},
				
				_ => unreachable!()
			}
		}
		
		(output, s[..s.len() - 1].to_string() + "_ppl")
	} else if name == "print" {
		(output, String::from("print!"))
	} else if name == "println" {
		(output, String::from("println!"))
	} else if name == "__uninit__" {
		(output, String::from("std::mem::uninitialized()"))
	} else {
		(output, name.to_string() + "_ppl")
	}
}

fn type_func_call(tokens: &Vec<Token>, mut output: String, i: &mut usize, children: &RefCell<Vec<usize>>, sidekicks: &RefCell<Vec<usize>>, name: &str) -> String {
	let (children, sidekicks) = (children.borrow(), sidekicks.borrow());
	
	if children.len() > 0 || sidekicks.iter().find(|&&s| match tokens[s].kind {
		Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) => if children.borrow().len() > 0 {true} else {false},
		_ => unreachable!()
	}).is_some() {
		if sidekicks.len() > 0 || (name != "unsafe" && name != "return") {
			output += "(";
		}
		
		if sidekicks.len() == 0 && (name == "print" || name == "println") {
			output += "\"{}\",";
		}
		
		let mut has_children = false;
		
		if children.len() > 0 && children[0] != usize::MAX {
			for (c, child) in children.iter().enumerate() {
				*i = *child;
				output = compile_tok(tokens, i, output, name == "unsafe");
				
				if c + 1 < children.len() {
					output += ",";
				}
			}
			
			has_children = true;
		}
		
		for (s, &sidekick) in sidekicks.iter().enumerate() {
			match tokens[sidekick].kind {
				Kind::Op(_, _, ref children, _, _) | Kind::Var(_, _, ref children, _, _) => if children.borrow().len() > 0 {
					if s > 0 || has_children {
						output += ",";
					}
					
					for (c, child) in children.borrow().iter().enumerate() {
						*i = *child;
						output = compile_tok(tokens, i, output, name == "unsafe");
						
						if c + 1 < children.borrow().len() {
							output += ",";
						}
					}
				},
				
				_ => unreachable!()
			}
		}
		
		if sidekicks.len() > 0 || (name != "unsafe" && name != "return") {
			output += ")";
		}
	}
	
	output
}

fn compile_tok(tokens: &Vec<Token>, i: &mut usize, mut output: String, is_exceptional: bool) -> String {
	match tokens[*i].kind {
		Kind::GroupOp(ref op, _) => if op == ";" {
			output += ";";
		} else {
			output = compile_body(tokens, i, output, !is_exceptional);
		},
		
		Kind::Literal(b) => if b {
			output += "true";
		} else {
			output += "false";
		},
		
		Kind::Number(int, fraction) => {
			output += &int.to_string();
			if fraction != 0 {
				output += ".";
				output += &fraction.to_string();
			}
		},
		
		Kind::Str1(ref s) => {
			output += "\"";
			output += s;
			output += "\"";
		},
		
		Kind::Str2(ref s) => {
			if s.len() == 1 || (s.len() == 2 && s.chars().next().unwrap() == '\\') { // Just a character, not an actual string
				output += "'";
				output += s;
				output += "'";
			} else {
				panic!("{}:{} P+ style strings are not supported yet", tokens[*i].pos.line, tokens[*i].pos.col);
			}
		},
		
		Kind::Var(ref name, _, ref children, ref sidekicks, _) => {
			let new_output;
			match type_full_name(tokens, output, sidekicks, &name) {
				(updated_output, new_output2) => {
					output = updated_output;
					new_output = new_output2;
				}
			}
			
			match new_output[..new_output.len() - 4].as_ref() {
				"let_eq" => match tokens[sidekicks.borrow()[0]].kind {
					Kind::Op(_, _, ref children, _, _) => {
						output += "let mut ";
						
						*i = children.borrow()[0];
						output = compile_tok(tokens, i, output, false);
						
						output += "=";
						
						*i = children.borrow()[1];
						output = compile_tok(tokens, i, output, false);
					},
					
					_ => unreachable!()
				},
				
				"return" => {
					output += "return ";
					output = type_func_call(tokens, output, i, children, sidekicks, &name);
				},
				
				"unsafe" => {
					output += "unsafe ";
					output = type_func_call(tokens, output, i, children, sidekicks, &name);
				},
				
				"Err" => {
					output += "Err";
					output = type_func_call(tokens, output, i, children, sidekicks, &name);
				},
				
				_ => {
					output += &new_output;
					output = type_func_call(tokens, output, i, children, sidekicks, &name);
				}
			}
		},
		
		Kind::Op(ref op, ref ops, ref children, ref sidekicks, _) => {
			let mut name = match op.as_ref() {
				"+" => "plus",
				"-" => "minus",
				"*" => "times",
				"/" => "div",
				"%" => "mod",
				"=" => "eq",
				"&" => "and",
				"|" => "or",
				"^" => "xor",
				"<" => "larrow",
				">" => "rarrow",
				"!" => "not",
				"~" => "binnot",
				"?" => "quest",
				":" => "colon",
				"." => "dot",
				"," => "comma",
				"@" => "at",
				_ => op
			}.to_string();
			
			for opid in ops.borrow().iter() {
				if let Kind::Op(ref op, _, _, _, _) = tokens[*opid].kind {
					name += match op.as_ref() {
						"+" => "plus",
						"-" => "minus",
						"*" => "times",
						"/" => "div",
						"%" => "mod",
						"=" => "eq",
						"&" => "and",
						"|" => "or",
						"^" => "xor",
						"<" => "larrow",
						">" => "rarrow",
						"!" => "not",
						"~" => "binnot",
						"?" => "quest",
						":" => "colon",
						"." => "dot",
						"," => "comma",
						"@" => "at",
						_ => op
					};
					
					*i = *opid;
				}
			}
			
			let new_output;
			match type_full_name(tokens, output, sidekicks, &name) {
				(updated_output, new_output2) => {
					output = updated_output;
					new_output = new_output2;
				}
			}
			
			match new_output[..new_output.len() - 4].as_ref() {
				"plus" | "minus" | "times" | "div" | "mod" | "eqeq" | "noteq" | "and" | "andand" | "or" | "oror" | "xor" | "larrow" | "larrowlarrow" | "rarrow" | "rarrowrarrow" | "larroweq" | "rarroweq" => {
					*i = children.borrow()[0];
					output = compile_tok(tokens, i, output, false);
					
					output += match new_output[..new_output.len() - 4].as_ref() {
						"plus" => "+",
						"minus" => "-",
						"times" => "*",
						"div" => "/",
						"mod" => "%",
						"eq" => "=",
						"eqeq" => "==",
						"noteq" => "!=",
						"pluseq" => "+=",
						"and" => "&",
						"andand" => "&&",
						"or" => "|",
						"oror" => "||",
						"xor" => "^",
						"larrow" => "<",
						"larrowlarrow" => "<<",
						"rarrow" => ">",
						"rarrowrarrow" => ">>",
						"larroweq" => "<=",
						"rarroweq" => ">=",
						_ => unreachable!()
					};
					
					*i = children.borrow()[1];
					output = compile_tok(tokens, i, output, false);
				},
				
				"eq" | "pluseq" | "minuseq" | "timeseq" | "diveq" | "modeq" | "larrowlarroweq" | "rarrowrarroweq" | "xoreq" => {
					output += "{";
					
					*i = children.borrow()[0];
					output = compile_tok(tokens, i, output, false);
					
					output += match new_output[..new_output.len() - 4].as_ref() {
						"eq" => "=",
						"pluseq" => "+=",
						"minuseq" => "-=",
						"timeseq" => "*=",
						"diveq" => "/=",
						"modeq" => "%=",
						"larrowlarroweq" => "<<=",
						"rarrowrarroweq" => ">>=",
						"xoreq" => "^=",
						_ => unreachable!()
					};
					
					*i = children.borrow()[1];
					output = compile_tok(tokens, i, output, false);
					
					output += ";true}";
				},
				
				"not" => {
					output += "!(";
					
					*i = children.borrow()[0];
					output = compile_tok(tokens, i, output, false);
					
					output += ")";
				},
				
				_ => {
					output += &new_output;
					output = type_func_call(tokens, output, i, children, sidekicks, &name);
				}
			}
		},
		
		_ => ()
	}
	
	output
}

fn compile_body(tokens: &Vec<Token>, i: &mut usize, mut output: String, in_expr: bool) -> String {
	if in_expr {
		output += "(";
	}
	
	output += "{";
	
	if let Kind::GroupOp(_, ref statements) = tokens[*i].kind {
		for statement in statements.borrow().iter() {
			output = compile_tok(tokens, &mut statement.clone(), output, false);
		}
	}
	
	*i += 1;
	
	output += "}";
	
	if in_expr {
		output += ")";
	}
	
	output
}

pub fn compile(tokens: &Vec<Token>, functions: &Vec<Function>, i: &mut usize, mut output: String) -> String {
	match tokens[*i].kind {
		Kind::Func(ref f, ref body) => if let FuncType::Func(f) = *f {
			output += "fn ";
			
			output = compile_func(&functions[f], output);
			
			*i = *body.borrow();
			output = compile_body(tokens, i, output, false);
		},
		
		Kind::Reserved(ref keyword, _) if keyword == "import" => {
			// Using Rust-style importing for now
			output += "use ";
			*i += 1;
			
			let mut success = false;
			while *i < tokens.len() {
				match tokens[*i].kind {
					Kind::Reserved(ref keyword, _) if keyword == "as" => {
						output += " as ";
						*i += 1;
					},
					
					Kind::GroupOp(ref op, _) if op == ";" => {
						output += ";";
						success = true;
						break;
					},
					
					_ => {
						output += &get_val!(tokens[*i].kind); // Will probably be changed
						*i += 1;
					}
				}
			}
			
			if !success {
				panic!("Unexpected EOF");
			}
		},
		
		Kind::Var(ref name, _, _, _, _) if name == "#" => {
			while *i < tokens.len() {
				match tokens[*i].kind {
					Kind::GroupOp(ref op, _) if op == "]" => {
						output += "]";
						break;
					},
					
					_ => {
						output += &get_val!(tokens[*i].kind); // Will probably be changed
						*i += 1;
					}
				}
			}
		},
		
		_ => ()
	}
	
	output
}