#ifndef DEF_INCLUDED
#define DEF_INCLUDED

int lex_parse(FILE *input, char ***keywords, size_t keywords_size, size_t *key, char ***pointers, size_t pointers_size, size_t *pkey, size_t file_size, char specials[]);

char *parse(char **keywords, size_t key, size_t *pos, char specials[]);

#endif