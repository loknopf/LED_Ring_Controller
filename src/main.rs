use std::{thread, time::Duration};
use rgb_ring::LEDRing;
use rppal::spi::{self, Bus, Error, Spi};

fn main() -> Result<(), Error>{
    let vec = vec![255,0,0];
    let speed:u32 = 6400000;
    let mut bus = Spi::new(Bus::Spi0, rppal::spi::SlaveSelect::Ss0, speed, rppal::spi::Mode::Mode0).unwrap();
    
    // let buff = get_spi_data(&vec);
    // println!("{:?}", buff);
    // bus.write(&buff);

    let mut led_ring = LEDRing::new(&mut bus, 8, 0.1);
    println!("Created struct");
    for i in 0..8{
        let _ = led_ring.set_led_values_rgb(i, &vec).unwrap();
    }
    match led_ring.show(){
        Ok(_) => (),
        Err(e) => panic!("{}", format!("Error when sending LED data, error: {}", e)),
    }

    thread::sleep(Duration::from_secs(5));

    for i in 0..8{
        let _ = led_ring.set_led_values_rgb(i, &vec![0,0,0]).unwrap();
    }
    match led_ring.show(){
        Ok(_) => (),
        Err(e) => panic!("{}", format!("Error when sending LED data, error: {}", e)),
    }

    bus.write(&get_spi_data(&vec![0,0,0]));
    
    Ok(())

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