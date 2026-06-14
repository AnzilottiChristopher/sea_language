#include <stdio.h>

typedef struct Dog Dog;
struct Dog {
    char* name;
};

void Dog_init(Dog *self, char* name) {
    self->name = name;
}
void Dog_bark(Dog *self) {
    printf("Woof!\n");
}
int main() {
    Dog dog;
    Dog_init(&dog, "Rex");
    Dog_bark(&dog);
}
