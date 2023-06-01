/*****************************************************************************
 *                                                                           *
 *                                p c s p k                                  *
 *                                                                           *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Mit Hilfe dieser Klasse kann man Toene auf dem           *
 *                  PC-Lautsprecher ausgeben.                                *
 *                                                                           *
 * Achtung:         Qemu muss mit dem Parameter -soundhw pcspk aufgerufen    *
 *                  werden. Ansonsten kann man nichts hoeren.                *
 *                                                                           *
 * Autor:           Michael Schoettner, HHU, 22.9.2016                       *
 *****************************************************************************/
#![allow(dead_code)]

use crate::kernel::cpu as cpu;


// Ports
//This port is used to control the Programmable Interval Timer (PIT) chip.
//It is used to configure the timer mode and select the desired channel
const PORT_CTRL:u16  = 0x43;

//This port is used to access the data register of Channel 0 of the PIT. 
//It is used to load the counter value and read the current count.
const PORT_DATA0:u16 = 0x40;

//This port is used to access the data register of Channel 2 of the PIT.
//It is used to load the counter value for generating sound.
const PORT_DATA2:u16 = 0x42;

//This port is used to control the Programmable Peripheral Interface (PPI). 
//It is used to enable or disable the PC speaker.
const PORT_PPI:u16   = 0x61;


// Note, Frequenz
const C0:f32 =    130.81;
const C0X:f32 =   138.59; 
const D0:f32 =    146.83;
const D0X:f32 =   155.56; 
const E0:f32 =    164.81; 
const F0:f32 =    174.61; 
const F0X:f32 =   185.00; 
const G0:f32 =    196.00; 
const G0X:f32 =   207.65; 
const A0:f32 =    220.00; 
const A0X:f32 =   233.08; 
const B0:f32 =    246.94; 

const C1:f32 =    261.63; 
const C1X:f32 =   277.18; 
const D1:f32 =    293.66; 
const D1X:f32 =   311.13; 
const E1:f32 =    329.63; 
const F1:f32 =    349.23; 
const F1X:f32 =   369.99; 
const G1:f32 =    391.00; 
const G1X:f32 =   415.30; 
const A1:f32 =    440.00; 
const A1X:f32 =   466.16; 
const B1:f32 =    493.88; 

const C2:f32 =    523.25; 
const C2X:f32 =   554.37; 
const D2:f32 =    587.33; 
const D2X:f32 =   622.25; 
const E2:f32 =    659.26; 
const F2:f32 =    698.46; 
const F2X:f32 =   739.99; 
const G2:f32 =    783.99; 
const G2X:f32 =   830.61; 
const A2:f32 =    880.00; 
const A2X:f32 =   923.33; 
const B2:f32 =    987.77; 
const C3:f32 =    1046.50; 


/*****************************************************************************
 * Methode:         play                                                     *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Ton abspielen.                                           *
 *                                                                           *
 * Rückgabewerte:   f:   Frequenz des Tons                                   *
 *                  len: Laenge des Tons in ms                               *
 *****************************************************************************/
pub fn play (f: f32, len: u32) {
	let freq: u32 = f as u32;
	let cnt_start: u32  =  1193180 / freq;
    let status: u8;
    
    
    // Zaehler laden
    cpu::outb(PORT_CTRL, 0xb6);			// Zaehler-2 konfigurieren
    cpu::outb(PORT_DATA2, (cnt_start%256) as u8); // Zaehler-2 laden (Lobyte)
    cpu::outb(PORT_DATA2, (cnt_start/256) as u8 );// Zaehler-2 laden (Hibyte)
    
    // Lautsprecher einschalten
    status = cpu::inb(PORT_PPI);	    // Status-Register des PPI auslesen
    cpu::outb(PORT_PPI, status | 3);    // Lautpsrecher Einschalten

    println!("Lautsprecher einschalten");

    // Pause
    delay(len);
    
    // Lautsprecher ausschalten
    off ();
}


/*****************************************************************************
 * Methode:         off                                                      *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Lautsprecher ausschalten.                                *
 *****************************************************************************/
pub fn off () {
    let status:u8 ;
    
    
    status = cpu::inb(PORT_PPI);	    // Status-Register des PPI auslesen
    cpu::outb(PORT_PPI, (status>>2)<<2);// Lautpsrecher Einschalten
}


/*****************************************************************************
 * Methode:         read_counter                                             *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Zaehler von PIT Channel 0 auslesen.                      * 
 *                  (wird fuer delay benoetigt).                             *
 *                                                                           *
 * Rückgabewerte:   counter                                                  *
 *****************************************************************************/
fn read_counter() -> u32 {
    let lo:u8;
    let hi:u8;

    //It sends a latch command (0x0) to the control port (PORT_CTRL) to latch the count value.
    cpu::outb(PORT_CTRL, 0x0);	        // Latch Command

   	//It reads the low byte of the count value from the data register (PORT_DATA0).
    lo = cpu::inb(PORT_DATA0);	        // Lobyte des Counters auslesen

    //It reads the high byte of the count value from the data register (PORT_DATA0).
    hi = cpu::inb(PORT_DATA0);	        // Hibyte des Counters auslesen

    //performs a conversion and bitwise operation to construct a 32-bit counter value from two 8-bit values representing the high and low bytes.
    //<< 8 shifts the binary representation of the hi value 8 bits to the left. In other words, it effectively multiplies hi by 2^8 (256) because each left shift by 1 bit corresponds to multiplying the number by 2. This step is performed to place the bits of the hi value in the higher-order bits of the resulting 32-bit value.
    //(hi as u32) << 8) | (lo as u32) performs a bitwise OR (|) operation between the shifted hi value and the lo value. The | operator performs a binary OR operation on each corresponding pair of bits. The result is a new u32 value where the bits from hi occupy the higher-order bits, and the bits from lo occupy the lower-order bits.
    //Finally, the resulting combined 32-bit counter is returned from the function
    return ((hi as u32) << 8) | (lo as u32);
}


/*****************************************************************************
 * Methode:         PCSPK::delay                                             *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Verzoegerung um X ms (in 1ms Schritten; Min. 1ms).       *
 *                  Da der Counter "nur" 16 Bit hat muss man evt. mehrmals   *
 *                  herunterzaehlen.                                         * 
 *                                                                           *
 * Parameter:       time (delay in ms)                                       *
 *****************************************************************************/
 fn delay(mut time: u32) {
    /* 

    let mut counter: u32;
    let mut start: u32;
    let mut end: u32;
    let mut diff: u32;
 
        
    // Zaehler laden
    counter = read_counter();
    
    // Startzeit merken
    start = counter;
    
    // Endzeit berechnen
    end = start - (time * 1193);
    
    // Endzeit erreicht?
    while counter > end {
        // Zaehler lesen
        counter = read_counter();
        
        // Unterschied berechnen
        diff = start - counter;
        
        // Unterschied zu gross?
        if diff > 1193 {
            // Zaehler neu laden
            start = counter;
        }
    }
    */
    let mut counter = u16::MAX as u32 - read_counter();
    let mut now = counter;
    let end = now + time * 1193;

    let mut last: u32;

    while counter < end {
        last = now;
        now = u16::MAX as u32 - read_counter();
        counter += if last > now {
            u16::MAX as u32 - last + now
        } else {
            now - last
        }
    }
    println!("Waited for {} ms", (time * 1193));
}



/*****************************************************************************
 * Methode:         tetris                                                   *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Tetris Sound, Kévin Rapaille, August 2013                *
 *                  https://gist.github.com/XeeX/6220067                     *
 *****************************************************************************/
pub fn tetris () {
    play(658.0, 125);
    play(1320.0, 500);
    play(990.0, 250);
    play(1056.0, 250);
    play(1188.0, 250);
    play(1320.0, 125);
    play(1188.0, 125);
    play(1056.0, 250);
    play(990.0, 250);
    play(880.0, 500);
    play(880.0, 250);
    play(1056.0, 250);
    play(1320.0, 500);
    play(1188.0, 250);
    play(1056.0, 250);
    play(990.0, 750);
    play(1056.0, 250);
    play(1188.0, 500);
    play(1320.0, 500);
    play(1056.0, 500);
    play(880.0, 500);
    play(880.0, 500);
    delay(250);
    play(1188.0, 500);
    play(1408.0, 250);
    play(1760.0, 500);
    play(1584.0, 250);
    play(1408.0, 250);
    play(1320.0, 750);
    play(1056.0, 250);
    play(1320.0, 500);
    play(1188.0, 250);
    play(1056.0, 250);
    play(990.0, 500);
    play(990.0, 250);
    play(1056.0, 250);
    play(1188.0, 500);
    play(1320.0, 500);
    play(1056.0, 500);
    play(880.0, 500);
    play(880.0, 500);
    delay(500);
    play(1320.0, 500);
    play(990.0, 250);
    play(1056.0, 250);
    play(1188.0, 250);
    play(1320.0, 125);
    play(1188.0, 125);
    play(1056.0, 250);
    play(990.0, 250);
    play(880.0, 500);
    play(880.0, 250);
    play(1056.0, 250);
    play(1320.0, 500);
    play(1188.0, 250);
    play(1056.0, 250);
    play(990.0, 750);
    play(1056.0, 250);
    play(1188.0, 500);
    play(1320.0, 500);
    play(1056.0, 500);
    play(880.0, 500);
    play(880.0, 500);
    delay(250);
    play(1188.0, 500);
    play(1408.0, 250);
    play(1760.0, 500);
    play(1584.0, 250);
    play(1408.0, 250);
    play(1320.0, 750);
    play(1056.0, 250);
    play(1320.0, 500);
    play(1188.0, 250);
    play(1056.0, 250);
    play(990.0, 500);
    play(990.0, 250);
    play(1056.0, 250);
    play(1188.0, 500);
    play(1320.0, 500);
    play(1056.0, 500);
    play(880.0, 500);
    play(880.0, 500);
    delay(500);
    play(660.0, 1000);
    play(528.0, 1000);
    play(594.0, 1000);
    play(495.0, 1000);
    play(528.0, 1000);
    play(440.0, 1000);
    play(419.0, 1000);
    play(495.0, 1000);
    play(660.0, 1000);
    play(528.0, 1000);
    play(594.0, 1000);
    play(495.0, 1000);
    play(528.0, 500);
    play(660.0, 500);
    play(880.0, 1000);
    play(838.0, 2000);
    play(660.0, 1000);
    play(528.0, 1000);
    play(594.0, 1000);
    play(495.0, 1000);
    play(528.0, 1000);
    play(440.0, 1000);
    play(419.0, 1000);
    play(495.0, 1000);
    play(660.0, 1000);
    play(528.0, 1000);
    play(594.0, 1000);
    play(495.0, 1000);
    play(528.0, 500);
    play(660.0, 500);
    play(880.0, 1000);
    play(838.0, 2000);
    off ();
}


/*****************************************************************************
 * Methode:         aerodynamic                                              *
 *---------------------------------------------------------------------------*
 * Beschreibung:    Clint, Part of Daft Punk’s Aerodynamic                   *
 *                  https://www.kirrus.co.uk/2010/09/linux-beep-music/       *
 *****************************************************************************/
pub fn aerodynamic() {
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(370.0, 122);
    play(493.9, 122);
    play(370.0, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(587.3, 122);
    play(415.3, 122);
    play(493.9, 122);
    play(415.3, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(784.0, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(493.9, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(440.0, 122);
    play(659.3, 122);
    play(440.0, 122);
    play(554.4, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(740.0, 122);
    play(987.8, 122);
    play(740.0, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1174.7, 122);
    play(830.6, 122);
    play(987.8, 122);
    play(830.6, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1568.0, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(987.8, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    play(1318.5, 122);
    play(880.0, 122);
    play(1108.7, 122);
    play(880.0, 122);
    off ();
}
