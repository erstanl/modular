#include <stdint.h>
#include <SPI.h>
#include "config.h"

extern "C" {
    #include "envelope.h"
}

void setup() {
    pinMode(GATE_IN_PIN, INPUT);
    pinMode(RETRIG_IN_PIN, INPUT);
    pinMode(BUTTON_PIN, INPUT_PULLUP);
    pinMode(DAC_CS_PIN, OUTPUT);
    digitalWrite(DAC_CS_PIN, HIGH);

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    #if GATE_PASSTHROUGH_ENABLED
    pinMode(GATE_OUT_PIN, OUTPUT);
    #endif

    #if LED_MODE_INDICATOR_ENABLED
    pinMode(LED_MODE_INDICATOR_PIN, OUTPUT);
    digitalWrite(LED_MODE_INDICATOR_PIN, HIGH);
    #endif

    attachInterrupt(digitalPinToInterrupt(GATE_IN_PIN), handleGateChange, CHANGE);
    attachInterrupt(digitalPinToInterrupt(RETRIG_IN_PIN), handleRetrigChange, CHANGE);
    enableInterrupt(BUTTON_PIN);

    pinMode(CV_PIN_A, INPUT);
    pinMode(CV_PIN_D, INPUT);
    pinMode(CV_PIN_S, INPUT);
    pinMode(CV_PIN_R, INPUT);

    for (int i = 0; i < 4; i++) {
        pinMode(LED_PINS[i], OUTPUT);
    }
    digitalWrite(LED_PINS[DEFAULT_MODE], HIGH);

    Serial.begin(9600);
}

void loop() {
    uint32_t currentTime = micros();
    float value = update(currentTime);
    Serial.println(value);
    MCP4922_write(DAC_CS_PIN, 0, value);
    MCP4922_write(DAC_CS_PIN, 1, 1 - value);
}

#if BUTTON_PIN >= 0 && BUTTON_PIN <= 7
#define PIN_VEC PCINT2_vect
#elif BUTTON_PIN >= 8 && BUTTON_PIN <= 13
#define PIN_VEC PCINT0_vect
#else
#define PIN_VEC PCINT1_vect
#endif
ISR(PIN_VEC) {
    handleButtonPress();
}

void handleGateChange() {
    static uint16_t lastValue = digitalRead(GATE_IN_PIN);
    uint16_t currentValue = digitalRead(GATE_IN_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        #if GATE_PASSTHROUGH_ENABLED
        digitalWrite(GATE_OUT_PIN, true);
        #endif
        gate(true);
    } else if (lastValue == LOW && currentValue == HIGH) {
        #if GATE_PASSTHROUGH_ENABLED
        digitalWrite(GATE_OUT_PIN, false);
        #endif
        gate(false);
    }
    lastValue = currentValue;
}

void handleRetrigChange() {
    static uint16_t lastValue = digitalRead(RETRIG_IN_PIN);
    uint16_t currentValue = digitalRead(RETRIG_IN_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        ping();
    }
    lastValue = currentValue;
}

inline void handleButtonPress() {
    static uint16_t lastValue = digitalRead(BUTTON_PIN);
    uint16_t currentValue = digitalRead(BUTTON_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        cycleModes();
    }
    lastValue = currentValue;
}

inline void enableInterrupt(byte pin) {
    *digitalPinToPCMSK(pin) |= bit (digitalPinToPCMSKbit(pin));  // enable pin
    PCIFR  |= bit (digitalPinToPCICRbit(pin)); // clear any outstanding interrupt
    PCICR  |= bit (digitalPinToPCICRbit(pin)); // enable interrupt for the group
}

/*
 * Writes a given value to a MCP4922 DAC chip to be output as
 * a voltage.
 *
 * cs_pin - which Arduino pin to use as the CHIP SELECT pin
 *     (should be connected to the CS pin of the DAC)
 * dac - 0 or 1 - Which of the MCP4922's internal DAC channels
 *     to output to (see MCP4922 datasheet for pinout diagram)
 * value - {0..1} - The value to output as a fraction of the
 *     DAC's max/reference voltage. Converted to a 12-bit int.
 */
void MCP4922_write(int cs_pin, byte dac, float value) {
    uint16_t value12 = (uint16_t) (value * 4095);
    byte low = value12 & 0xff;
    byte high = (value12 >> 8) & 0x0f;
    dac = (dac & 1) << 7;
    digitalWrite(cs_pin, LOW);
    SPI.transfer(dac | 0x30 | high);
    SPI.transfer(low);
    digitalWrite(cs_pin, HIGH);
}
