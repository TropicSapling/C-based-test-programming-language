#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <string.h>
#include <stdbool.h>
#include <ctype.h>

#include "def.h"

#define INCR_MEM(size) do { \
	if(pos + (size) > output_size) addSpaceForChars(outputp); \
} while(0)

#define typeToOutput(str) do { \
	typeTo(outputp, str); \
} while(0)

#define RED   "\x1B[31m"
#define GREEN   "\x1B[32m"
#define YELLOW   "\x1B[33m"
#define BLUE   "\x1B[34m"
#define WHITE   "\x1B[37;1m"
#define RESET "\x1B[0m"

char *file;

char types[22][8] = {"bool", "chan", "char", "clang", "const", "fraction", "func", "heap", "int", "list", "noscope", "number", "only", "pointer", "register", "signed", "stack", "static", "unique", "unsigned", "void", "volatile"};
char reserved_keys[19][8] = {"async", "break", "case", "continue", "default", "do", "else", "eval", "export", "foreach", "goto", "if", "import", "in", "repeat", "return", "switch", "type", "while"};
size_t iterators = 0;
size_t bools = 0;

size_t output_size = 256;

size_t lineno = 1;
size_t linecol = 1;

void addSpaceForChars(char **output) {
	output_size *= 2;
	
	char *res = realloc(*output, output_size);
	if(res == NULL) {
		perror("ERROR");
		fprintf(stderr, "ID: %d\n", errno);
		exit(EXIT_FAILURE);
	} else {
		*output = res;
	}
}

static bool isReserved(char arr[][8], char *str, unsigned int len) {
	for(unsigned int i = 0; i < len; i++) {
		if(strcmp(arr[i], str) == 0) return true;
	}
	
	return false;
}

static bool isNumber(char *str) {
	for(unsigned int i = 0; str[i] != '\0'; i++) {
		if(!isdigit(str[i])) return false;
	}
	
	return true;
}

static void typeTo(char **outputp, char *str) {
	for(unsigned int i = 0; str[i] != '\0'; i++) {
		if(pos + 1 > output_size) addSpaceForChars(outputp);
		(*outputp)[pos] = str[i];
		pos++;
	}
}

static void addID(char *str_end, size_t *IDs) {
	char *chars = "abcdefghijklmnopqrstuvwxyz";
	
	str_end[0] = chars[(*IDs / (26 * 26)) % 26];
	str_end[1] = chars[(*IDs / 26) % 26];
	str_end[2] = chars[*IDs % 26];
	str_end[3] = '\0';
	
	(*IDs)++;
}

static unsigned int getListExpStartPos(unsigned int i, char **keywords) {
	unsigned int st_pos = i;
	unsigned short parentheses = 0;
	unsigned short brackets = 0;
	
	while(keywords[st_pos][0] == ')' || keywords[st_pos][0] == ']') {
		if(keywords[st_pos][0] == ')') {
			while(keywords[st_pos][0] != '(' || parentheses > 0 || (keywords[st_pos - 1][0] == ')' && st_pos--)) {
				if(keywords[st_pos][0] == ')') parentheses++;
				if(parentheses && keywords[st_pos][0] == '(') parentheses--;
				
				st_pos--;
			}
		} else if(keywords[st_pos][0] == ']') {
			while(keywords[st_pos][0] != '[' || brackets > 0 || (keywords[st_pos - 1][0] == ']' && st_pos--)) {
				if(keywords[st_pos][0] == ']') brackets++;
				if(brackets && keywords[st_pos][0] == '[') brackets--;
				
				st_pos--;
			}
		}
		
		st_pos--;
	}
	
	return st_pos;
}

static unsigned int getListExpEndPos(char **keywords) {
	unsigned int en_pos = 0;
	unsigned short brackets = 0;
	
	while(keywords[en_pos][0] != ']' || brackets > 0) {
		if(keywords[en_pos][0] == '[') brackets++;
		if(brackets && keywords[en_pos][0] == ']') brackets--;
		
		en_pos++;
	}
	
	en_pos++;
	
	while(keywords[en_pos][0] == ')') en_pos++;
	
	return en_pos;
}

static size_t parseKey(unsigned int i, char **keywords, char **outputp, unsigned short *status, char *cItem);

static void typeSublistStartPos(unsigned int *sp_pos, char **keywords, char **outputp, unsigned short *stat, char *it_name) {
	if(keywords[*sp_pos][0] == '>') {
		typeToOutput("0;"); // Use default start pos
	} else if(strcmp(keywords[*sp_pos], "when") == 0) {
		typeToOutput("0;while(!(");
		
		// Get sublist start pos condition
		for((*sp_pos)++; !(keywords[*sp_pos][0] == '>' && keywords[*sp_pos + 1][0] == '>' && keywords[*sp_pos + 2][0] == '>'); (*sp_pos)++) {
			*sp_pos = parseKey(*sp_pos, keywords, outputp, stat, it_name);
		}
		
		typeToOutput(")){");
		typeToOutput(it_name);
		
		// Create while loop
		typeToOutput("++;}");
	} else {
		for(; !(keywords[*sp_pos][0] == '>' && keywords[*sp_pos + 1][0] == '>' && keywords[*sp_pos + 2][0] == '>'); (*sp_pos)++) {
			*sp_pos = parseKey(*sp_pos, keywords, outputp, stat, it_name);
		}
		
		INCR_MEM(1);
		(*outputp)[pos] = ';';
		pos++;
	}
	
	*sp_pos += 3;
}

static unsigned int typeSublistEndPos(char **keywords, char **outputp, unsigned short *stat, char *it_name, unsigned int i_pos) {
	if(keywords[i_pos][0] == ']') { // Use default
//		typeToOutput(list_length); // TODO: Define 'list_length'
	} else {
		bool isNormal = false;
		
		if(strcmp(keywords[i_pos], "until") == 0) {
			i_pos++;
		} else {
			typeToOutput("!(");
			typeToOutput(it_name);
			typeToOutput("<(");
			
			isNormal = true;
		}
		
		unsigned short brackets = 0;
		for(; keywords[i_pos][0] != ']' || brackets > 0; i_pos++) {
			i_pos = parseKey(i_pos, keywords, outputp, stat, it_name);
			
			if(keywords[i_pos][0] == '[') brackets++;
			if(brackets && keywords[i_pos][0] == ']') brackets--;
		}
		
		if(isNormal) typeToOutput("))");
	}
	
	return i_pos;
}

static size_t parseKey(unsigned int i, char **keywords, char **outputp, unsigned short *status, char *cItem) {
	if(strcmp(keywords[i], "false") == 0) {
		INCR_MEM(1);
		
		(*outputp)[pos] = '0';
		pos++;
	} else if(strcmp(keywords[i], "true") == 0) {
		INCR_MEM(1);
		
		(*outputp)[pos] = '1';
		pos++;
	} else if(strcmp(keywords[i], "unique") == 0) {
		typeToOutput("restrict ");
	} else if(strcmp(keywords[i], "pointer") == 0) {
		INCR_MEM(1);
		
		(*outputp)[pos] = '*';
		pos++;
		
		if(i + 2 < key && keywords[i + 1][0] == '*') {
			i += 2;
			
			for(unsigned short s = 0; s < keywords[i][0] - 49; s++) {
				INCR_MEM(1);
				
				(*outputp)[pos] = '*';
				pos++;
			}
		}
	} else if(keywords[i][0] == '-' && keywords[i + 1][0] == '>') {
		// POINTER CREATION
		
		if(!(keywords[i - 1][0] == '=' && strstr(specials, keywords[i - 2]) == NULL)) { // Assignment
			INCR_MEM(1);
			
			(*outputp)[pos] = '=';
			pos++;
		}
		
		if(keywords[i + 2][0] != '{' && keywords[i + 2][0] != '\'' && keywords[i + 2][0] != '"') {
			INCR_MEM(1);
			
			(*outputp)[pos] = '&';
			pos++;
		}
		
		i++;
	} else if(keywords[i][0] == '@') {
		// POINTER ACCESS
		
		INCR_MEM(1);
		
		(*outputp)[pos] = '*';
		pos++;
	} else if(keywords[i][0] == '\'') {
		// STRINGS (without null termination)
		
		if(keywords[i][2] == '\0' || (keywords[i][1] == '\\' && keywords[i][2] == '0' && keywords[i][3] == '\0')) {
			INCR_MEM(3);
			
			(*outputp)[pos] = '\'';
			pos++;
			
			(*outputp)[pos] = keywords[i][1];
			pos++;
			if(keywords[i][2] != '\0') {
				INCR_MEM(1);
				
				(*outputp)[pos] = keywords[i][2];
				pos++;
			}
			
			(*outputp)[pos] = '\'';
			pos++;
		} else {
			INCR_MEM(2);
			
			(*outputp)[pos] = '{';
			pos++;
			
			for(unsigned int c = 1; keywords[i][c] != '\0'; c++) {
				INCR_MEM(3);
				
				(*outputp)[pos] = '\'';
				pos++;
				
				(*outputp)[pos] = keywords[i][c];
				pos++;
				if(keywords[i][c] == '\\' && keywords[i][c + 1] == '0') {
					INCR_MEM(1);
					
					(*outputp)[pos] = keywords[i][c + 1];
					pos++;
					c++;
				}
				
				(*outputp)[pos] = '\'';
				pos++;
				if(keywords[i][c + 1] != '\0') {
					INCR_MEM(1);
					(*outputp)[pos] = ',';
					pos++;
				}
			}
			
			(*outputp)[pos] = '}';
			pos++;
		}
	} else if(keywords[i][0] == '"') {
		// STRINGS (with null termination)
		
		for(unsigned int c = 0; keywords[i][c] != '\0'; c++) {
			INCR_MEM(1);
			(*outputp)[pos] = keywords[i][c];
			pos++;
		}
		
		INCR_MEM(1);
		(*outputp)[pos] = '"';
		pos++;
	} else if(strcmp(keywords[i], "clang") == 0) {
		// INLINE C
		
		puts("----------------------------------------------------------------");
		printf(WHITE "%s:%zu:%zu: " RESET YELLOW "Warning:" RESET " '" WHITE "clang" RESET "' is not implemented yet.\n", file, lineno, linecol);
		printf("	...%s %s %s " YELLOW "clang" RESET " %s %s %s...\n", keywords[i - 3], keywords[i - 2], keywords[i - 1], keywords[i + 1], keywords[i + 2], keywords[i + 3]);
		puts("----------------------------------------------------------------");
		
		for(unsigned int j = 1; j < 9; j++) {
			unsigned int k = 0;
			for(; k < 22; k++) {
				if(strcmp(keywords[i + j], types[k]) == 0) {
					break;
				}
			}
			
			if(k == 22) {
				i = i + j + 2;
				break;
			}
		}
		
		// WIP
	} else if(strcmp(keywords[i], "__args") == 0) {
		typeToOutput("argv");
	} else if(strcmp(keywords[i], "__argc") == 0) {
		typeToOutput("argc");
	} else if(strcmp(keywords[i], "__line") == 0) {
		typeToOutput("__LINE__");
	} else if(strcmp(keywords[i], "__path") == 0) {
		typeToOutput("__PATH__");
	} else if(strcmp(keywords[i], "__item") == 0) {
		if(cItem == NULL) {
			puts("----------------------------------------------------------------");
			printf(WHITE "%s:%zu:%zu: " RESET RED "Error:" RESET " Invalid placement of '" WHITE "__item" RESET "'.\n", file, lineno, linecol);
			printf("	...%s %s %s " RED "__item" RESET " %s %s %s...\n", keywords[i - 3], keywords[i - 2], keywords[i - 1], keywords[i + 1], keywords[i + 2], keywords[i + 3]);
			puts("----------------------------------------------------------------");
			
			exit(EXIT_FAILURE);
		} else {
			typeToOutput(cItem);
		}
	} else if(i + 1 < key && keywords[i + 1][0] == '[') {
		// LISTS
		
		bool foundSublist = false;
		
		unsigned int i_pos = 2;
		for(; keywords[i + i_pos][0] != ']'; i_pos++) {
			if(keywords[i + i_pos][0] == '>' && keywords[i + i_pos + 1][0] == '>' && keywords[i + i_pos + 2][0] == '>') {
				unsigned short stat = 0;
				
				if(*status != 1) {
					unsigned int sop_pos = i + i_pos + getListExpEndPos(&keywords[i + i_pos]);
					
					if(keywords[sop_pos][0] == '=' && strstr(specials, keywords[sop_pos - 1]) == NULL) {
						// WIP
						break;
					} else if(keywords[sop_pos - 1][0] == '+' && keywords[i - 1][0] == '=') {
						// WIP
						break;
					} else if(keywords[sop_pos - 1][0] == '*' && keywords[i - 1][0] == '=') {
						// WIP
						break;
					} else if(keywords[sop_pos - 1][0] == '-' && keywords[i - 1][0] == '=') {
						// WIP
						break;
					} else if(keywords[sop_pos - 1][0] == '/' && keywords[i - 1][0] == '=') {
						// WIP
						break;
					} else if(keywords[sop_pos][0] == '>' || keywords[sop_pos][0] == '<' || keywords[sop_pos][0] == '=' || keywords[sop_pos][0] == '!' || keywords[sop_pos][0] == '&' || keywords[sop_pos][0] == '|') {
						// keywords[sop_pos] is a comparison operator
						
						foundSublist = true;
						
						while(pos >= 0 && (*outputp)[pos - 1] != ';' && (*outputp)[pos - 1] != '{' && (*outputp)[pos - 1] != '}') { // NEEDS FIXING
							pos--;
						}
						
						INCR_MEM(5);
						
						// Create first iterator
						typeToOutput("size_t ");
						
						char it_name[11] = "ppl_it_";
						addID(it_name + 7, &iterators);
						
						typeToOutput(it_name);
						(*outputp)[pos] = '=';
						pos++;
						
						// Type first sublist start pos
						unsigned int sp_pos = i + 2;
						typeSublistStartPos(&sp_pos, keywords, outputp, &stat, it_name);
						
						// Create second iterator
						typeToOutput("size_t ");
						
						char it2_name[11] = "ppl_it_";
						addID(it2_name + 7, &iterators);
						
						typeToOutput(it2_name);
						(*outputp)[pos] = '=';
						pos++;
						
						// Get second sublist expression pos
						unsigned int sp2_pos = sop_pos;
						while(keywords[sp2_pos][0] == '>' || keywords[sp2_pos][0] == '<' || keywords[sp2_pos][0] == '=' || keywords[sp2_pos][0] == '!' || keywords[sp2_pos][0] == '&' || keywords[sp2_pos][0] == '|') {
							sp2_pos++;
						}
						
						// Get second sublist start pos
						unsigned short parentheses = 0;
						while(keywords[sp2_pos][0] != '[' || parentheses > 0) {
							if(keywords[sp2_pos][0] == '(') parentheses++;
							if(parentheses && keywords[sp2_pos][0] == ')') parentheses--;
							
							sp2_pos++;
						}
						sp2_pos++;
						
						// Type second sublist start pos
						typeSublistStartPos(&sp2_pos, keywords, outputp, &stat, it2_name);
						
						i_pos += 3;
						
						// Create condition bool
						char cond_bool[17] = "ppl_condBool_";
						addID(cond_bool + 13, &bools);
						
						typeToOutput("int ");
						typeToOutput(cond_bool);
						typeToOutput("=1;while(1){if(!(");
						
						unsigned int listExpStart_pos = getListExpStartPos(i, keywords);
						unsigned int listExpStart_pos2 = listExpStart_pos;
						
						// Type first sublist expression
						stat = 1;
						for(; listExpStart_pos < i + 1; listExpStart_pos++) {
							listExpStart_pos = parseKey(listExpStart_pos, keywords, outputp, &stat, cItem);
						}
						
						stat = 0;
						
						(*outputp)[pos] = '[';
						pos++;
						typeToOutput(it_name);
						(*outputp)[pos] = ']';
						pos++;
						
						// Type comparison operator
						for(; keywords[sop_pos][0] == '>' || keywords[sop_pos][0] == '<' || keywords[sop_pos][0] == '=' || keywords[sop_pos][0] == '!' || keywords[sop_pos][0] == '&' || keywords[sop_pos][0] == '|'; sop_pos++) {
							typeToOutput(keywords[sop_pos]);
						}
						
						// Type second sublist expression
						while(keywords[sop_pos][0] != '[' || parentheses > 0) {
							if(keywords[sop_pos][0] == '(') parentheses++;
							if(parentheses && keywords[sop_pos][0] == ')') parentheses--;
							
							sop_pos = parseKey(sop_pos, keywords, outputp, &stat, cItem);
							
							sop_pos++;
						}
						
						(*outputp)[pos] = '[';
						pos++;
						typeToOutput(it2_name);
						typeToOutput("])){");
						typeToOutput(cond_bool);
						typeToOutput("=0;break;}");
						
						// Increment
						typeToOutput(it_name);
						typeToOutput("++;");
						typeToOutput(it2_name);
						typeToOutput("++;if(");
						
						// Check if comparison is done
						typeSublistEndPos(keywords, outputp, &stat, it_name, i + i_pos);
						typeToOutput("&&");
						typeSublistEndPos(keywords, outputp, &stat, it2_name, sp2_pos);
						typeToOutput("){break;}else if(");
						typeSublistEndPos(keywords, outputp, &stat, it_name, i + i_pos);
						typeToOutput("||");
						sp2_pos = typeSublistEndPos(keywords, outputp, &stat, it2_name, sp2_pos);
						typeToOutput("){");
						typeToOutput(cond_bool);
						typeToOutput("=0;break;}}");
						
						unsigned int stBef_pos = listExpStart_pos2;
						while(keywords[stBef_pos][0] != ';' && keywords[stBef_pos][0] != '{' && keywords[stBef_pos][0] != '}') {
							stBef_pos--;
						}
						stBef_pos++;
						
						// Type statement before comparison
						for(; stBef_pos < listExpStart_pos2; stBef_pos++) {
							stBef_pos = parseKey(stBef_pos, keywords, outputp, &stat, cItem);
						}
						
						// Include comparison results
						typeToOutput(cond_bool);
						
						i = sp2_pos;
						break;
					}
				}
			} else if(keywords[i + i_pos][0] == '<' && keywords[i + i_pos + 1][0] == '<' && keywords[i + i_pos + 2][0] == '<') {
				break; // TMP, WIP
			}
		}
		
		if(!foundSublist) {
			typeToOutput(keywords[i]);
//			typeToOutput("_ppl");
		}
	} else {
		if(strcmp(keywords[i], "#include") == 0) { // TMP; makes it possible to include C functions without the need of 'import clib'
			INCR_MEM(2);
			
			if(pos < 1 || (*outputp)[pos - 1] != '\n') {
				INCR_MEM(1);
				(*outputp)[pos] = '\n';
				pos++;
			}
			
			typeToOutput(keywords[i]);
			i++;
			
			(*outputp)[pos] = ' ';
			pos++;
			
			while(keywords[i][0] != ';') {
				typeToOutput(keywords[i]);
				i++;
			}
			
			(*outputp)[pos] = '\n';
			pos++;
		} else {
			typeToOutput(keywords[i]);
		}
		
		if(keywords[i][0] != '"' && keywords[i][0] != '\'' && !isNumber(keywords[i]) && !isReserved(types, keywords[i], 22) && !isReserved(reserved_keys, keywords[i], 19) && strstr(specials, keywords[i]) == NULL) {
//			typeToOutput("_ppl");
		}
		
		if(strstr(specials, keywords[i]) == NULL && strstr(specials, keywords[i + 1]) == NULL) {
			INCR_MEM(1);
			
			(*outputp)[pos] = ' ';
			pos++;
		}
	}
	
	if(keywords[i][0] == ';') {
		lineno++;
		linecol = 1;
	} else {
		linecol++;
	}
	
	return i;
}

char *parse(char **keywords, char *filename) {
	char *output = malloc(output_size);
	file = filename;
	
	unsigned short status = 0;
	
	for(size_t i = 0; i < key; i++) {
		i = parseKey(i, keywords, &output, &status, NULL);
	}
	
	if(pos + 1 > output_size) addSpaceForChars(&output);
	output[pos] = '\0';
	
	return output;
}
