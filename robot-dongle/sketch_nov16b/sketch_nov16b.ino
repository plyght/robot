#include <Servo.h>

Servo myServo;  // Create a servo object

void setup() {
  myServo.attach(13);  // Attaches the servo to pin 9
}

void loop() {
  myServo.write(40);   // Move servo to 0 degrees
  delay(10000);        // Wait for 1 second
  myServo.write(0);  // Move servo to 90 degrees
  delay(1000);
  myServo.write(40);   // Move servo to 0 degrees
  delay(20000);        // Wait for 1 second
  myServo.write(0);  // Move servo to 90 degrees
  delay(1000);
  myServo.write(40);   // Move servo to 0 degrees
  delay(20000);        // Wait for 1 second
  myServo.write(0);  // Move servo to 90 degrees
  delay(1000);
}
