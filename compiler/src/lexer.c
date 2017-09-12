#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <string.h>
#include <stdbool.h>

#include "def.h"

#define INCR_MEM(size) do { \
	if(*key + (size) - 1 >= keywords_size / (sizeof(char*) + 1) && addSpaceForKeys(keywords, &keywords_size) == NULL) { \
		return 1; \
	} \
} while(0)

bool inStr = false;
bool inStr2 = false;
bool escaping = false;
bool ignoring = false;

char *addSpaceForKeys(char ***keywords, size_t *keywords_size) {
	*keywords_size *= 2;
	
	char *res = realloc(*keywords, *keywords_size);
	if(res == NULL) {
		perror("ERROR");
		fprintf(stderr, "ID: %d\n", errno);
	} else {
		*keywords = (char**) res;
	}
	
	return res;
}

int lex_parse(FILE *input, char ***keywords, size_t keywords_size, size_t *key, size_t file_size, char specials[]) {
	char buf[65536];
	char extra_buf[16] = "\0";
	char *c;
	
	long double progress = 0.0;
	
	while(fgets(buf, 65536, input) != NULL) {
		if(strcmp(buf, "\n") == 0 || strcmp(buf, "\r\n") == 0) {
			continue;
		}
		
		size_t new_size = strlen(buf) + 1;
		char *trimmed_buf = &buf[0];
		while(*trimmed_buf == '\t' || *trimmed_buf == ' ') {
			new_size--;
			trimmed_buf++;
		}
		
		// PREPROCESSING
		if(trimmed_buf[0] == '#' && !inStr) {
			size_t c = 1;
			char skey[8];
			
			while(trimmed_buf[c] != ' ' && trimmed_buf[c] != '\0') {
				skey[c] = trimmed_buf[c];
				c++;
			}
			
			c++;
			progress += c;
			
			if(strcmp(skey, "redef") == 0) {
				for(short s = 0; specials[s] != '\0'; s++) {
					if(trimmed_buf[c] == specials[s]) {
						specials[s] = trimmed_buf[c + 5];
						break;
					}
				}
				
				progress += 5;
			} else if(strcmp(skey, "def") == 0) {
				// WIP
			} else if(strcmp(skey, "ifdef") == 0) {
				// WIP
			} else if(strcmp(skey, "import") == 0) {
				// WIP
			} else if(strcmp(skey, "export") == 0) {
				// WIP
			}
			
			continue;
		}
		
		if(extra_buf != NULL) {
			new_size += strlen(extra_buf);
		}
		
		char *tmp = malloc(new_size);
		if(tmp == NULL) {
			perror("ERROR");
			fprintf(stderr, "ID: %d\n", errno);
		} else {
			c = tmp;
		}
		
		if(extra_buf[0] != '\0') {
			strcpy(c, extra_buf);
			strcat(c, trimmed_buf);
			
			extra_buf[0] = '\0';
		} else {
			strcpy(c, trimmed_buf);
		}
		
		INCR_MEM(2);
		
		(*keywords)[*key] = NULL; // This is used to mark where memory was allocated for 'c'
		(*key)++;
		
		(*keywords)[*key] = c;
		(*key)++;
		
		size_t row_len = 0;
		
		while(row_len < 65521) {
			char *special;
			bool foundSpecial = false;
			
			if(ignoring) {
				*c = '\0';
				c++;
			}
			
			while((ignoring || inStr || inStr2 || *c != ' ') && *c != '\0') {
				if(ignoring) {
					if(*c == '*' && *(c + 1) == '/') {
						ignoring = false;
						c++;
						
						INCR_MEM(1);
						
						(*keywords)[*key] = c + 1;
						(*key)++;
					}
				} else if(!inStr && (*c == specials[0] || *c == specials[1] || *c == specials[2] || *c == specials[3] || *c == specials[4] || *c == specials[5] || *c == specials[6] || *c == specials[7] || *c == specials[8] || *c == specials[9] || *c == specials[10] || *c == specials[11] || *c == specials[12] || *c == specials[13] || *c == specials[14] || *c == specials[15] || *c == specials[16] || *c == specials[17] || *c == specials[18] || *c == specials[19] || *c == specials[20] || *c == specials[21] || *c == specials[22] || *c == specials[23] || *c == specials[24])) {
					special = calloc(2, 1);
					special[0] = *c;
					foundSpecial = true;
					
					break;
				} else if(escaping) {
					escaping = false;
				} else if(!inStr2 && *c == '\'') {
					if(inStr) {
						inStr = false;
						break;
					} else {
						inStr = true;
					}
				} else if(!inStr && *c == '"') {
					if(inStr2) {
						inStr2 = false;
						break;
					} else {
						inStr2 = true;
					}
				} else if(*c == '\\') {
					escaping = true;
				}
				
				c++;
				row_len++;
			}
			
			if(*c == '\0') {
				if(*(c - 1) == '\n') *(c - 1) = '\0';
				if(*(c - 2) == '\r') *(c - 2) = '\0';
				c++;
				break;
			} else {
				if(*c == '/' && *(c + 1) == '/') {
					*c = '\0';
					free(special);
					
					break;
				} else if(*c == '/' && *(c + 1) == '*') {
					ignoring = true;
					*c = '\0';
					free(special);
					
					continue;
				}
			}
			
			*c = '\0';
			
			c++;
			row_len++;
			
			if(row_len < 65521) {
				if(foundSpecial) {
					INCR_MEM(2);
					
					(*keywords)[*key] = NULL; // This is used to mark where memory was allocated for 'special'
					(*key)++;
					
					(*keywords)[*key] = special;
					(*key)++;
				}
				
				INCR_MEM(1);
				
				(*keywords)[*key] = c;
				(*key)++;
			}
		}
		
		if(row_len > 65520) {
			strcpy(extra_buf, c);
		}
		
		progress += row_len;
		printf("Reading file... %.2Lf%%\r", (progress / file_size) * 100);
		fflush(stdout);
	}
	
	return 0;
}