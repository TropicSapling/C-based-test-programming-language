use std::cell::RefCell;
use lib::{Token, Type, Type2, FilePos};

fn is_var(c: char) -> bool {
	c == '_' || c == '$' || c.is_alphanumeric()
}

pub fn lex<'a>(contents: &'a String) -> Vec<&'a str> {
	let mut result = Vec::new();
	let mut last = 0;
	for (index, matched) in contents.match_indices(|c: char| !is_var(c)) {
		if last != index {
			result.push(&contents[last..index]);
		}
		
		result.push(matched);
		
		last = index + matched.len();
	}
	
	if last < contents.len() {
		result.push(&contents[last..]);
	}
	
	result
}

pub fn lex2(tokens: Vec<&str>) -> Vec<Token> {
	let mut res: Vec<Token> = Vec::new();
	let mut string = Token {
		val: String::from(""),
		t: Type::Str1,
		t2: Type2::Void,
		pos: FilePos {line: 1, col: 1},
		children: RefCell::new(vec![])
	};
	
	let mut in_str = false;
	let mut in_str2 = false;
	let mut escaping = false;
	let mut ignoring = false;
	let mut ignoring2 = false;
	let mut possible_comment = false;
	
	let mut num_pos = 0;
	let mut line = 1;
	let mut col = 1;
	
	for item in tokens {
		if ignoring {
			if item == "\n" {
				res.push(Token {val: item.to_string(), t: Type::Whitespace, t2: Type2::Void, pos: FilePos {line, col}, children: RefCell::new(vec![])});
				
				line += 1;
				col = 0;
				ignoring = false;
			}
			
			if item == "\r" {
				res.push(Token {val: item.to_string(), t: Type::Whitespace, t2: Type2::Void, pos: FilePos {line, col}, children: RefCell::new(vec![])});
			}
		} else if ignoring2 {
			if possible_comment {
				if item == "/" {
					ignoring2 = false;
				}
				
				possible_comment = false;
			}
			
			if item == "*" {
				possible_comment = true;
			}
			
			if item == "\n" {
				line += 1;
				col = 0;
			}
		} else {
			if possible_comment {
				if item == "/" {
					ignoring = true;
					possible_comment = false;
					
					continue;
				} else if item == "*" {
					ignoring2 = true;
					possible_comment = false;
					
					continue;
				} else {
					possible_comment = false;
					
					string.val = String::from("/");
					string.t = Type::Op;
					string.pos = FilePos {line, col};
					
					res.push(string.clone());
					string.val = String::from("");
				}
			}
			
			if escaping {
				if item == "0" || item == "n" { // Null and newlines
					string.val += "\\";
				}
				
				string.val += item;
				string.pos = FilePos {line, col};
				
				escaping = false;
			} else if in_str {
				if item == "\"" {
					res.push(string.clone());
					string.val = String::from("");
					in_str = false;
				} else if item == "\\" {
					escaping = true;
				} else {
					string.val += item;
				}
			} else if in_str2 {
				if item == "'" {
					res.push(string.clone());
					string.val = String::from("");
					in_str2 = false;
				} else if item == "\\" {
					escaping = true;
				} else {
					string.val += item;
				}
			} else if item == "\"" {
				string.t = Type::Str1;
				string.pos = FilePos {line, col};
				in_str = true;
			} else if item == "'" {
				string.t = Type::Str2;
				string.pos = FilePos {line, col};
				in_str2 = true;
			} else {
				if num_pos > 0 && (item == "." || num_pos == 2) {
					string.val += item;
					if num_pos == 2 {
						res.push(string.clone());
						string.val = String::from("");
						
						num_pos = 0;
					} else {
						num_pos = 2;
					}
					
					continue;
				} else if num_pos == 1 {
					res.push(string.clone());
					string.val = String::from("");
					
					num_pos = 0;
				}
				
				if item == "/" {
					possible_comment = true;
				} else if item.parse::<u64>().is_ok() {
					string.val = item.to_string();
					string.t = Type::Number;
					string.pos = FilePos {line, col};
					
					num_pos = 1;
				} else {
					string.val = item.to_string();
					string.t = match item {
						"+" | "-" | "*" | "/" | "%" | "=" | "&" | "|" | "^" | "<" | ">" | "!" | "~" | "?" | ":" | "." | "," | "@" | ";" => Type::Op,
						"{" | "}" | "[" | "]" | "(" | ")" => Type::GroupOp,
						"array" | "bool" | "chan" | "char" | "const" | "fraction" | "func" | "heap" | "int" | "list" | "number" | "only" | "pointer" | "register" | "signed" | "stack" | "unique" | "unsigned" | "void" | "volatile" => Type::Type,
						"as" | "async" | "break" | "continue" | "else" | "export" | "foreach" | "from" | "goto" | "if" | "import" | "in" | "match" | "receive" | "repeat" | "return" | "select" | "send" | "to" | "type" | "until" | "when" | "while" => Type::Reserved,
						"false" | "true" => Type::Literal,
						"\n" => {
							line += 1;
							col = 0;
							Type::Whitespace
						},
						"\r" | "\t" | " " => Type::Whitespace,
						_ => Type::Var
					};
					if string.t == Type::Type {
						string.t2 = match item {
							"array" => Type2::Array,
							"bool" => Type2::Bool,
							"chan" => Type2::Chan,
							"char" => Type2::Char,
							"const" => Type2::Const,
							"fraction" => Type2::Fraction,
							"func" => Type2::Func,
							"heap" => Type2::Heap,
							"int" => Type2::Int,
							"list" => Type2::List,
							"only" => Type2::Only,
							"pointer" => Type2::Pointer,
							"register" => Type2::Register,
							"stack" => Type2::Stack,
							"unique" => Type2::Unique,
							"unsigned" => Type2::Unsigned,
							"volatile" => Type2::Volatile,
							_ => Type2::Void,
						};
					}
					string.pos = FilePos {line, col};
					
					res.push(string.clone());
					string.val = String::from("");
					string.t2 = Type2::Void;
				}
			}
		}
		
		col += 1;
	}
	
	res
}