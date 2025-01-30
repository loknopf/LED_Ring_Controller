use std::{f32::consts::E, thread, time::Duration, vec};
use clap::Parser;
use rgb_ring::{convert_string_to_hex, LEDRing, LEDRingCliArgs, LEDRingCommands, LEDRingError};
use rppal::spi::{self, Bus, Error, Spi};

fn main() -> Result<(), LEDRingError>{
    let vec = vec![255,0,0];
    let speed:u32 = 6400000;
    let mut bus = Spi::new(Bus::Spi0, rppal::spi::SlaveSelect::Ss0, speed, rppal::spi::Mode::Mode0);

    // let buff = get_spi_data(&vec);
    // println!("{:?}", buff);
    // bus.write(&buff);

    // let mut led_ring = LEDRing::new(&mut bus, 8, 0.1);
    // println!("Created struct");
    // for i in 0..8{
    //     let _ = led_ring.set_led_values_rgb(i, &vec).unwrap();
    // }
    // match led_ring.show(){
    //     Ok(_) => (),
    //     Err(e) => panic!("{}", format!("Error when sending LED data, error: {}", e)),
    // }

    // thread::sleep(Duration::from_secs(5));

    // for i in 0..8{
    //     let _ = led_ring.set_led_values_rgb(i, &vec![0,0,0]).unwrap();
    // }
    // match led_ring.show(){
    //     Ok(_) => (),
    //     Err(e) => panic!("{}", format!("Error when sending LED data, error: {}", e)),
    // }

    // bus.write(&get_spi_data(&vec![0,0,0]));
    
    // Ok(())

    let args = LEDRingCliArgs::parse();
    let num_led = args.leds;
    if let Some(config) = args.config{
        todo!();
    }

    let brightness = args.brightness;

    //Creating the LEDRing controller
    let mut controller: LEDRing<'_>;
    match bus{
        Ok(ref mut bus) => controller = LEDRing::new(bus, num_led, brightness),
        Err(e) => return Err(LEDRingError::IoError(e)),
    }

    match args.command{
        LEDRingCommands::SetSingleRGB { index, r, g, b } => set_single_rgb(r, g, b, index, &mut controller),
        LEDRingCommands::SetSingleHEX { index, hex } => set_single_hex(hex, index, &mut controller),
        LEDRingCommands::SetMultipleRGB { indices, r, g, b } => set_multiple_rgb(indices, r, g, b, &mut controller),
        LEDRingCommands::SetMultipleHEX { indices, hex } => set_multiple_hex(indices, hex, &mut controller),
    }

}

fn set_single_rgb(red: u8, 
                  green: u8, 
                  blue: u8, 
                  index:u8, 
                  controller: &mut LEDRing) -> Result<(), LEDRingError>{
    controller.set_led_values_rgb(index, &vec![red, green, blue])?;
    controller.show()
}

fn set_single_hex(hex: String, index:u8, controller: &mut LEDRing) -> Result<(), LEDRingError>{
    match convert_string_to_hex(hex) {
        Ok(value) => controller.set_led_values_hex(index, &value),
        Err(e) => return Err(e) 
    }?;
    controller.show()
}

fn set_multiple_rgb(indices: Vec<u8>, 
                    red: u8, 
                    green: u8, 
                    blue: u8, 
                    controller: &mut LEDRing) -> Result<(), LEDRingError>{
    let vec = vec![red, green, blue];
    for idx in indices{
        controller.set_led_values_rgb(idx, &vec)?;
    }
    controller.show()
}

fn set_multiple_hex(indices: Vec<u8>, hex: String, controller: &mut LEDRing) -> Result<(), LEDRingError>{
    let hex_value = convert_string_to_hex(hex)?;
    for idx in indices{
        controller.set_led_values_hex(idx, &hex_value)?;
    }

    controller.show()
}

fn get_spi_data(vec: &Vec<u8>) -> Vec<u8>{
    let mut tx = vec![0; vec.len()*8];
    for ibit in 0..8  {
        let mut i = 0;
        for value in vec{
            let current_bit = (value >> ibit) & 1;
            //0x78 represents 1111000 and 0x80 is 10000000 
            //when the bit is high we get 5 bits high and 3 low, 
            //if the bit is low we get 1 high and 7 low
            let encoded_bit = current_bit * 0x78 + 0x80;
            tx[7-ibit + i*8] = encoded_bit;
            i+=1;
        }
    }
    return tx
}