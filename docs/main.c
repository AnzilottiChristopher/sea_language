#include <stdio.h>

#include "./animal.h"
int main() {
    Animal animal;
    Animal_init(&animal, "Rex");
    Animal_speak(&animal);
}
