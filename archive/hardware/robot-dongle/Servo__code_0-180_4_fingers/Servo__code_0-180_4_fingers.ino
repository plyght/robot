#include <Servo.h>

Servo ServoPinky;
Servo ServoRing;
Servo ServoMiddle;
Servo ServoPointer;

int serialval;

void setup() {
  ServoPinky.attach(6); //Change to 13-10
  ServoRing.attach(4);
  ServoMiddle.attach(5);
  ServoPointer.attach(3);

  Serial.begin(9600);

}
void loop() {

if (Serial.available() > 0) {
  
    String command = Serial.readStringUntil('\\n');
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
              Serial.print("Moved Servo 1 to ");
              Serial.print(angle);
              Serial.println(" degrees");
              break;
            case 2:
              ServoRing.write(angle);
              Serial.print("Moved Servo 2 to ");
              Serial.print(angle);
              Serial.println(" degrees");
              break;
            
            case 3:
              ServoMiddle.write(angle);
              Serial.println("Moved Servo 3");
              Serial.print(angle);
              Serial.println(" degrees");
              break;

            case 4:
              ServoPointer.write(angle);
              Serial.println("Moved Servo 4");
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
  
