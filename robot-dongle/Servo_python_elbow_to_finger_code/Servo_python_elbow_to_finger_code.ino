#include <Servo.h>   // Format - S1:180
                     // Format - S(servo number):(angle)
Servo ServoPinky;
Servo ServoRing;
Servo ServoMiddle;
Servo ServoPointer;

Servo Thumbyone;  // 0-90
Servo Thumbytwo;

Servo Wrist;
Servo Radius;     // 0-270

Servo Elbowyone;  // 0-180
Servo Elbowytwo;  // 180-0

int serialval;

void setup() {
  
  ServoPinky.attach(6);   //Change to 13-10
  ServoRing.attach(4);
  ServoMiddle.attach(5);
  ServoPointer.attach(3);

  Thumbyone.attach(10);   //Change to 10-9
  Thumbytwo.attach(9);
  
  Wrist.attach(8);        //Change to 8-7
  Radius.attach(7);

  Elbowyone.attach(6);    //Change to 6-5
  Elbowytwo.attach(5);

//  xx.attach(4);         //Change to 4-1
//  xx.attach(3);
//  xx.attach(2);
//  xx.attach(1);

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
              Serial.println(" degrees");          //End of Fingers
              break;

            case 5:
              Thumbyone.write(angle);              // 0-90
              Serial.println("Moved Servo 4");
              Serial.print(angle);
              Serial.println(" degrees");
              break;

            case 6:
              Thumbytwo.write(angle);
              Serial.println("Moved Servo 4");
              Serial.print(angle);
              Serial.println(" degrees");          //End of Thumb
              break;

            case 7:
              Wrist.write(angle);
              Serial.println("Moved Servo 4");
              Serial.print(angle);
              Serial.println(" degrees");
              break;

            case 8:
              Radius.write(angle);                  // 0-270
              Serial.println("Moved Servo 4");
              Serial.print(angle);
              Serial.println(" degrees");           //End of Main arm
              break;

            case 9:
              Elbowyone.write(angle);               // 0-180
              Serial.println("Moved Servo 4");
              Serial.print(angle);
              Serial.println(" degrees");
              break;

            case 10:
              Elbowytwo.write(angle);                // 180-0
              Serial.println("Moved Servo 4");
              Serial.print(angle);
              Serial.println(" degrees");            //End of Elbow
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
  
