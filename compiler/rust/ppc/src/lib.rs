#[derive(Clone)]
#[derive(Debug)]
pub struct Token {
	val: String,
	t: &'static str
}

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
		t: "str1"
	};
	
	let mut in_str = false;
	let mut in_str2 = false;
	let mut escaping = false;
	
	let mut num_pos = 0;
	
	for item in tokens {
		if escaping {
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
			string.t = "str1";
			in_str = true;
		} else if item == "'" {
			string.t = "str2";
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
			
			if item.parse::<u64>().is_ok() {
				string.val = item.to_string();
				string.t = "number";
				
				num_pos = 1;
			} else {
				string.val = item.to_string();
				string.t = match item {
					"+" | "-" | "*" | "/" | "%" | "=" | "&" | "|" | "^" | "<" | ">" | "[" | "]" | "(" | ")" | "!" | "~" | "?" | ":" | "." | "," | "@" | ";" | "{" | "}" => "operator",
					"array" | "bool" | "chan" | "char" | "const" | "fraction" | "func" | "heap" | "int" | "list" | "number" | "only" | "pointer" | "register" | "signed" | "stack" | "unique" | "unsigned" | "void" | "volatile" => "type",
					"as" | "async" | "break" | "continue" | "else" | "foreach" | "from" | "goto" | "if" | "in" | "match" | "receive" | "repeat" | "return" | "select" | "send" | "to" | "type" | "until" | "when" | "while" => "reserved",
					"false" | "true" => "literal",
					"\n" | "\r" | "\t" | " " => "whitespace",
					_ => "variable"
				};
				
				res.push(string.clone());
			}
		}
	}
	
	res
}