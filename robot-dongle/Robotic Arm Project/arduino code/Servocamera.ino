#include "Servo.h"

Servo servo1; //
Servo servo2; //
Servo servo3; //
Servo servo4; //

int value; //

void setup () {
  
  Serial.begin(9600);
  
  servo1.attach(6); //
  servo2.attach(4); //
  servo3.attach(5); //
  servo4.attach(3); //

  value = 0;
  
  
}

void loop () {
  if (Serial.available()) { 

  Serial.parseInt();

  value = Serial.parseInt();

  if (value == 0)  {
  servo1.write(0);
  servo2.write(180);
  servo3.write(180);
  servo4.write(0);
}
  
  if (value == 1) {
  servo1.write(180); // u Ring
  servo2.write(180); // d middle
  servo3.write(180); // d pinky
  servo4.write(0);   // d ponter
  }
    
    if (value == 2) {
  servo1.write(0);   // d
  servo2.write(0);   // u
  servo3.write(180); // d
  servo4.write(0);   // d
  }

  if (value == 3) {
  servo1.write(0);   // d
  servo2.write(180); // d
  servo3.write(0);   // u
  servo4.write(0);   // d
  }

  if (value == 4) {
  servo1.write(0);   // d Ring
  servo2.write(180); // d middle
  servo3.write(180); // d pinky
  servo4.write(180); // u ponter
  }

  if (value == 5) {
  servo1.write(0);   // d Ring
  servo2.write(0);   // u middle
  servo3.write(180); // d pinky
  servo4.write(180); // u ponter
  }

  if (value == 6) {
  servo1.write(180); // u Ring
  servo2.write(180); // d middle
  servo3.write(180); // d pinky
  servo4.write(180); // u ponter
  }

  if (value == 7) {
  servo1.write(0);   // d Ring
  servo2.write(180); // d middle
  servo3.write(0); // u pinky
  servo4.write(180); // u ponter
  }

  if (value == 8) {
  servo1.write(180);  // u Ring
  servo2.write(0);   // u middle
  servo3.write(180); // d pinky
  servo4.write(0);   // d ponter
  }

  if (value == 9) {
  servo1.write(0);   // d Ring
  servo2.write(0);   // u middle
  servo3.write(0);   // u pinky
  servo4.write(0);   // d ponter
  }

  if (value == 10) {
  servo1.write(180); // u Ring
  servo2.write(180); // d middle
  servo3.write(0);   // u pinky
  servo4.write(0);   // d ponter
  }

  if (value == 11) {
  servo1.write(180); // u Ring
  servo2.write(0);   // u middle
  servo3.write(180); // d pinky
  servo4.write(180);   // u ponter
  }

  if (value == 12) {
  servo1.write(180); // u Ring
  servo2.write(180); // d middle
  servo3.write(0);   // u pinky
  servo4.write(180); // u ponter
  }

  if (value == 13) {
  servo1.write(0);   // d Ring
  servo2.write(0);   // u middle
  servo3.write(0);   // u pinky
  servo4.write(180); // u ponter
  }

  if (value == 14) {
  servo1.write(180); // u Ring
  servo2.write(0);   // u middle
  servo3.write(0);   // u pinky
  servo4.write(0);   // d ponter
  }

  if (value == 15) {
  servo1.write(180); // u Ring
  servo2.write(0);   // u middle
  servo3.write(0);   // u pinky
  servo4.write(180); // u ponter
  }
  
  delay(30);  
 } } 
