#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <string.h>
#include <sys/stat.h>

#include "def.h"

int main(int argc, char *argv[]) {
	
	//////////////// PREPARE FOR LEXING ////////////////
	
	if(argc < 2 || argc > 3) {
		puts("Invalid usage. Please specify an input file as the first argument and an output file as the second argument.");
		return 1;
	}
	
	FILE *input = fopen(argv[1], "r"); // Will be "r+" if automatic compiled GC gets added in the future and/or if needed to fix 'read-dir' bug
	
	if(input == NULL) {
		perror("ERROR");
		fprintf(stderr, "ID: %d\n", errno);
		return 1;
	}
	
	///////////////// PREPROCESS INPUT /////////////////
	
	char specials[] = ";,[]{}()?><=+-*/%!&|^~@\\.:";
	
	size_t processed_input_size = 256;
	char *processed_input = malloc(processed_input_size);
	preprocess(&input, &processed_input, processed_input_size, argv, NULL, NULL, NULL);
	
	fclose(input);
	
	puts("[DEBUG] Read and preprocessed file.");
	
	/////////////////// START LEXING ///////////////////
	
	size_t keywords_size = sizeof(char*) * 32;
	size_t pointers_size = sizeof(char*) * 32;
	char **keywords = malloc(keywords_size);
	char **pointers = malloc(pointers_size);
	size_t key = 0;
	size_t pkey = 0;
	
	lex_parse(processed_input, &keywords, keywords_size, &key, &pointers, pointers_size, &pkey, specials);
	
	puts("[DEBUG] Lex-parsed input.");
	
	/////////////////// START PARSING //////////////////
	
	size_t pos = 0;
	char *parsed_output = parse(keywords, key, &pos, specials);
	
	free(processed_input);
	
	puts("[DEBUG] Parsed input.");
	
	//////////////// PREPARE FOR OUTPUT ////////////////
	
	FILE *output;
	
	if(argc < 3) {
		size_t file_length = strlen(argv[1]);
		char filename[file_length];
		strcpy(filename, argv[1]);
		
		size_t c = file_length - 1;
		while(filename[c] != '.') {
			c--;
		}
		
		memset(&filename[c + 1], 'c', 1);
		memset(&filename[c + 2], '\0', 1);
		
		while(filename[c] != '/' && c > 0) {
			c--;
		}
		
		char finalname[strlen(filename) + 4];
		char success;
		
		if(c == 0) {
			success = mkdir("bin", 0777);
			
			strcpy(finalname, "bin/");
			strcat(finalname, filename);
		} else {
			strncpy(finalname, filename, c + 1);
			finalname[c + 1] = '\0';
			strcat(finalname, "bin/");
			success = mkdir(finalname, 0777);
			strcat(finalname, &filename[c + 1]);
		}
		
		if(success != 0 && errno != 17) {
			perror("ERROR");
			fprintf(stderr, "ID: %d\n", errno);
			
			return 1;
		}
		
		output = fopen(finalname, "w");
	} else {
		output = fopen(argv[2], "w");
	}
	
	/////////////////// PRINT OUTPUT ///////////////////
	
	fprintf(output, "int main(int argc,char *argv[]){");
	
	for(size_t i = 0; i < pos; i++) {
		if(parsed_output[i] == '$') continue; // TMP; makes it possible to use C functions without the need of 'clang'
		fprintf(output, "%c", parsed_output[i]);
		
		printf("[DEBUG] Printing output... %.2Lf%%\r", (((long double) i + 1) / key) * 100);
	}
	
	fprintf(output, "}");
	
	fclose(output);
	puts("[DEBUG] Printing output... 100.00%");
	
	/////////////////// FREE MEMORY ////////////////////
	
	free(parsed_output);
	
	for(size_t i = 0; i < pkey; i++) {
		free(pointers[i]);
	}
	
	free(keywords);
	free(pointers);
	
	return 0;
}