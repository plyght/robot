import cv2
from cvzone.HandTrackingModule import HandDetector
from cvzone.SerialModule import SerialObject

cap = cv2.VideoCapture(0)

detector = HandDetector(detectionCon=0.8)
arduino = SerialObject("COM6")

while True:
    success, image = cap.read()
    hands, bboxInfo = detector.findHands(image)
    if len(hands)==1:
        print(detector.fingersUp(hands[0]))
        if detector.fingersUp(hands[0]) == [0, 1, 0, 0, 0]: #pointer
            arduino.sendData([4])
        else:
            if detector.fingersUp(hands[0]) == [0, 0, 1, 0, 0]: #middle
                arduino.sendData([2])
            else:
                if detector.fingersUp(hands[0]) == [0, 0, 0, 1, 0]: #ring
                    arduino.sendData([1])
                else:
                    if detector.fingersUp(hands[0]) == [0, 0, 0, 0, 1]: #pinky
                        arduino.sendData([3])
                    else:
                        if detector.fingersUp(hands[0]) == [0, 1, 1, 0, 0]:
                            arduino.sendData([5])
                        else:
                             if detector.fingersUp(hands[0]) == [0, 1, 0, 1, 0]:
                                 arduino.sendData([6])
                             else:
                                  if detector.fingersUp(hands[0]) == [0, 1, 0, 0, 1]: #new segment next
                                      arduino.sendData([7])
                                  else:
                                      if detector.fingersUp(hands[0]) == [0, 0, 1, 1, 0]:
                                          arduino.sendData([8])
                                      else:
                                          if detector.fingersUp(hands[0]) == [0, 0, 1, 0, 1]: #new segment next
                                              arduino.sendData([9])
                                          else:
                                              if detector.fingersUp(hands[0]) == [0, 0, 0, 1, 1]: #new segment next
                                                  arduino.sendData([10])
                                              else:
                                                  if detector.fingersUp(hands[0]) == [0, 1, 1, 1, 0]:
                                                      arduino.sendData([11])
                                                  else:
                                                      if detector.fingersUp(hands[0]) == [0, 1, 0, 1, 1]:
                                                          arduino.sendData([12])
                                                      else:
                                                          if detector.fingersUp(hands[0]) == [0, 1, 1, 0, 1]:  #new segment next
                                                              arduino.sendData([13])
                                                          else:
                                                              if detector.fingersUp(hands[0]) == [0, 0, 1, 1, 1]: #new segment next
                                                                  arduino.sendData([14])
                                                              else:
                                                                  if detector.fingersUp(hands[0]) == [0, 1, 1, 1, 1]:
                                                                      arduino.sendData([15])
                                                                  else:
                                                                   arduino.sendData([0])
    cv2.imshow('image',image)
    cv2.waitKey(1)
