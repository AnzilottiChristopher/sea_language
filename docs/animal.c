#include <stdio.h>

typedef struct Animal Animal;
struct Animal {
    char* name;
};

void Animal_init(Animal *self, char* name) {
    self->name = name;
}
void Animal_speak(Animal *self) {
    printf("...\n");
}
