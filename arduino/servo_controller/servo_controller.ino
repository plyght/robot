#include <Servo.h>

Servo ServoPinky;
Servo ServoRing;
Servo ServoMiddle;
Servo ServoPointer;

void setup() {
  Serial.begin(9600);
  Serial.println("Arduino Servo Controller Ready");
  
  // Attach servos but don't move them - they'll stay at their last position
  // This prevents resetting all servos to 90 when serial port opens
  ServoPinky.attach(6);
  ServoRing.attach(4);
  ServoMiddle.attach(5);
  ServoPointer.attach(3);
  
  // Don't write 90 - let servos stay where they were
  // If you want to initialize to a specific position, do it explicitly
}

void loop() {
  if (Serial.available() > 0) {
    String command = Serial.readStringUntil('\n');
    command.trim();
    if (command.startsWith("S")) {
      int separatorIndex = command.indexOf(':');
      if (separatorIndex != -1) {
        int servoId = command.substring(1, separatorIndex).toInt();
        int angle = command.substring(separatorIndex + 1).toInt();
        if (angle >= 0 && angle <= 180) {
          switch (servoId) {
            case 1:
              ServoPinky.write(angle);
              Serial.print("Moved Servo 1 (Pinky) to ");
              Serial.print(angle);
              Serial.println(" degrees");
              break;
            case 2:
              ServoRing.write(angle);
              Serial.print("Moved Servo 2 (Ring) to ");
              Serial.print(angle);
              Serial.println(" degrees");
              break;
            case 3:
              ServoMiddle.write(angle);
              Serial.print("Moved Servo 3 (Middle) to ");
              Serial.print(angle);
              Serial.println(" degrees");
              break;
            case 4:
              ServoPointer.write(angle);
              Serial.print("Moved Servo 4 (Pointer/Index) to ");
              Serial.print(angle);
              Serial.println(" degrees");
              break;
            default:
              Serial.println("Unknown Servo ID");
          }
        } else {
          Serial.println("Invalid angle (0-180)");
        }
      }
    }
  }
}
