#ifndef ANIMAL_H
#define ANIMAL_H

typedef struct Animal Animal;
struct Animal {
    char* name;
};

void Animal_init(Animal *self, char* name);
void Animal_speak(Animal *self);

#endif // ANIMAL_H
